use expect_test::{expect, Expect};

fn debug_iter<D: std::fmt::Debug>(iter: impl Iterator<Item = D>) -> String {
    iter.map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn check(src: &str, expect: Expect) {
    let mut lexer = super::Lexer::new(src);
    let lines = debug_iter(std::iter::from_fn(|| lexer.line()));
    let errors = debug_iter(lexer.errors.iter());
    let literals = debug_iter(lexer.literals.iter());
    expect.assert_eq(&format!(
        "errors:\n{errors}\nliterals:\n{literals}\nlines:\n{lines}",
    ));
}

macro_rules! check {
    ($name:ident, $lit:literal, $expect: expr) => {
        #[test]
        fn $name() {
            check($lit, $expect);
        }
    };
}
check!(
    empty,
    "",
    expect![[r#"
        errors:

        literals:

        lines:
    "#]]
);
check!(
    multi_empty,
    "\n\n\n\n",
    expect![[r#"
        errors:

        literals:

        lines:
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }"#]]
);
check!(
    empty_ws,
    "\t\t    \t     \t ",
    expect![[r#"
        errors:

        literals:

        lines:
        Line { kind: Empty, literals: (0, 0), comment: None }"#]]
);
check!(
    multi_empty_ws,
    "\t\t    \t     \t \n\n\t\t    \t     \t \n\n",
    expect![[r#"
        errors:

        literals:

        lines:
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }"#]]
);
check!(
    singles,
    "one\n two\t\n threeeeee\n four\n",
    expect![[r#"
        errors:

        literals:

        lines:
        Line { kind: Instruction((0.0, 1.3)), literals: (0, 0), comment: None }
        Line { kind: Instruction((1.1, 2.4)), literals: (0, 0), comment: None }
        Line { kind: Instruction((2.1, 3.10)), literals: (0, 0), comment: None }
        Line { kind: Instruction((3.1, 4.5)), literals: (0, 0), comment: None }"#]]
);
check!(
    doubles,
    "one two\n two threeeee\t\n threeeeee four\n four five\n",
    expect![[r#"
        errors:

        literals:

        lines:
        Line { kind: Variable((0.0, 1.3), (0.4, 1.7)), literals: (0, 0), comment: None }
        Line { kind: Variable((1.1, 2.4), (1.5, 2.13)), literals: (0, 0), comment: None }
        Line { kind: Variable((2.1, 3.10), (2.11, 3.15)), literals: (0, 0), comment: None }
        Line { kind: Variable((3.1, 4.5), (3.6, 4.10)), literals: (0, 0), comment: None }"#]]
);
check!(
    triples,
    "one two three\n two threeeee four",
    expect![[r#"
        errors:

        literals:
        ((0.8, 1.13), Ident)
        ((1.14, 2.18), Ident)
        lines:
        Line { kind: Variable((0.0, 1.3), (0.4, 1.7)), literals: (0, 1), comment: None }
        Line { kind: Variable((1.1, 2.4), (1.5, 2.13)), literals: (1, 2), comment: None }"#]]
);
check!(
    empty_comments,
    "\
\t\t  \t    ; one two three
; three four five
\t; five six",
    expect![[r#"
        errors:

        literals:

        lines:
        Line { kind: Empty, literals: (0, 0), comment: Some(9) }
        Line { kind: Empty, literals: (0, 0), comment: Some(0) }
        Line { kind: Empty, literals: (0, 0), comment: Some(1) }"#]]
);
check!(
    comment_etc,
    "\
abc\t\t  \t    ; one two three
cde efg; three four five
ghi ijk klm\t; five six",
    expect![[r#"
        errors:

        literals:
        ((2.8, 3.11), Ident)
        lines:
        Line { kind: Instruction((0.0, 1.3)), literals: (0, 0), comment: Some(12) }
        Line { kind: Variable((1.0, 2.3), (1.4, 2.7)), literals: (0, 0), comment: Some(7) }
        Line { kind: Variable((2.0, 3.3), (2.4, 3.7)), literals: (0, 1), comment: Some(12) }"#]]
);
check!(
    section,
    "\
section data one
section bss
section text",
    expect![[r#"
        errors:

        literals:
        ((0.13, 1.16), Ident)
        lines:
        Line { kind: Section((0.8, 1.12)), literals: (0, 1), comment: None }
        Line { kind: Section((1.8, 2.11)), literals: (1, 1), comment: None }
        Line { kind: Section((2.8, 3.12)), literals: (1, 1), comment: None }"#]]
);
check!(
    comma_err,
    "\
,
,,,, ,,
,one ,two,",
    expect![[r#"
        errors:
        ((0.0, 1.1), UnknownChar(','))
        ((1.0, 2.1), UnknownChar(','))
        ((2.0, 3.1), UnknownChar(','))
        literals:
        ((2.1, 3.4), Ident)
        ((2.6, 3.9), Ident)
        lines:
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 0), comment: None }
        Line { kind: Empty, literals: (0, 2), comment: None }"#]]
);
