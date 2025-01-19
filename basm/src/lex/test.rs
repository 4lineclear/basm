use expect_test::{expect, Expect};

fn debug_iter<D: std::fmt::Debug>(iter: impl Iterator<Item = D>) -> String {
    iter.map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// TODO: create a better stringify that assembles lines, errors, and literals
// into the actual order they appear in.
fn check(src: &str, expect: Expect) {
    let mut lexer = super::Lexer::new(src);
    let lines: Vec<_> = std::iter::from_fn(|| lexer.line()).collect();
    let lines = debug_iter(
        lines
            .iter()
            .map(|l| (l.kind, l.literals, l.errors, l.comment)),
    );
    let errors = debug_iter(lexer.errors.iter());
    let literals = debug_iter(lexer.literals.iter());
    expect.assert_eq(&format!(
        "errors:\n{errors}\nliterals:\n{literals}\nlines:\n{lines}",
    ));
}

#[test]
fn empty() {
    check(
        "",
        expect![[r#"
            errors:

            literals:

            lines:
        "#]],
    );
}
#[test]
fn multi_empty() {
    check(
        "\n\n\n\n",
        expect![[r#"
            errors:

            literals:

            lines:
            (Empty, (0, 0), (0, 0), None)
            (Empty, (0, 0), (0, 0), None)
            (Empty, (0, 0), (0, 0), None)
            (Empty, (0, 0), (0, 0), None)"#]],
    );
}
#[test]
fn empty_ws() {
    check(
        "\t\t    \t     \t ",
        expect![[r#"
            errors:

            literals:

            lines:
            (Empty, (0, 0), (0, 0), None)"#]],
    );
}
#[test]
fn multi_empty_ws() {
    check(
        "\t\t    \t     \t \n\n\t\t    \t     \t \n\n",
        expect![[r#"
            errors:

            literals:

            lines:
            (Empty, (0, 0), (0, 0), None)
            (Empty, (0, 0), (0, 0), None)
            (Empty, (0, 0), (0, 0), None)
            (Empty, (0, 0), (0, 0), None)"#]],
    );
}
#[test]
fn singles() {
    check(
        "one\n two\t\n threeeeee\n four\n",
        expect![[r#"
            errors:

            literals:

            lines:
            (Instruction((0, 3)), (0, 0), (0, 0), None)
            (Instruction((1, 4)), (0, 0), (0, 0), None)
            (Instruction((1, 10)), (0, 0), (0, 0), None)
            (Instruction((1, 5)), (0, 0), (0, 0), None)"#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            errors:

            literals:
            ((4, 7), Ident)
            ((5, 13), Ident)
            ((11, 15), Ident)
            ((6, 10), Ident)
            lines:
            (Instruction((0, 3)), (0, 1), (0, 0), None)
            (Instruction((1, 4)), (1, 2), (0, 0), None)
            (Instruction((1, 10)), (2, 3), (0, 0), None)
            (Instruction((1, 5)), (3, 4), (0, 0), None)"#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            errors:

            literals:
            ((4, 7), Ident)
            ((9, 14), Ident)
            ((5, 13), Ident)
            ((15, 19), Ident)
            lines:
            (Instruction((0, 3)), (0, 2), (0, 0), None)
            (Instruction((1, 4)), (2, 4), (0, 0), None)"#]],
    );
}
#[test]
fn empty_comments() {
    check(
        "\
\t\t  \t    ; one two three
; three four five
\t; five six",
        expect![[r#"
            errors:

            literals:

            lines:
            (Empty, (0, 0), (0, 0), Some((9, 24)))
            (Empty, (0, 0), (0, 0), Some((0, 17)))
            (Empty, (0, 0), (0, 0), Some((1, 11)))"#]],
    );
}
#[test]
fn comment_etc() {
    check(
        "\
abc\t\t  \t    ; one two three
cde efg; three four five
ghi ijk, klm\t; five six",
        expect![[r#"
            errors:

            literals:
            ((4, 7), Ident)
            ((4, 7), Ident)
            ((9, 12), Ident)
            lines:
            (Instruction((0, 3)), (0, 0), (0, 0), Some((12, 27)))
            (Instruction((0, 3)), (0, 1), (0, 0), Some((7, 24)))
            (Instruction((0, 3)), (1, 3), (0, 0), Some((13, 23)))"#]],
    );
}
#[test]
fn section() {
    check(
        "\
section data one
section bss
section text",
        expect![[r#"
            errors:

            literals:
            ((13, 16), Ident)
            lines:
            (Section((0, 7), (8, 12)), (0, 1), (0, 0), None)
            (Section((0, 7), (8, 11)), (1, 1), (0, 0), None)
            (Section((0, 7), (8, 12)), (1, 1), (0, 0), None)"#]],
    );
}
#[test]
fn decimal() {
    check(
        "\
1, 1293, 9384093, 1231234",
        expect![[r#"
            errors:

            literals:
            ((0, 1), Decimal)
            ((3, 7), Decimal)
            ((9, 16), Decimal)
            ((18, 25), Decimal)
            lines:
            (Empty, (0, 4), (0, 0), None)"#]],
    );
}
#[test]
fn variable() {
    check(
        "\
msg db \"ONE TWO THREE\", 12309, 12",
        expect![[r#"
            errors:

            literals:
            ((7, 22), String)
            ((24, 29), Decimal)
            ((31, 33), Decimal)
            lines:
            (Variable((0, 3), (4, 6)), (0, 3), (0, 0), None)"#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            errors:
            ((22, 23), MissingComma)
            ((28, 29), MissingComma)
            literals:
            ((7, 22), String)
            ((23, 28), Decimal)
            ((29, 31), Decimal)
            lines:
            (Variable((0, 3), (4, 6)), (0, 3), (0, 2), None)"#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            errors:

            literals:
            ((15, 29), String)
            ((31, 33), Decimal)
            ((14, 20), Ident)
            ((14, 17), Ident)
            ((19, 20), Decimal)
            ((14, 17), Ident)
            ((19, 20), Decimal)
            ((14, 17), Ident)
            ((19, 26), Ident)
            ((14, 17), Ident)
            ((19, 21), Decimal)
            ((14, 17), Ident)
            ((19, 21), Decimal)
            ((14, 17), Ident)
            ((19, 22), Ident)
            lines:
            (Section((0, 7), (10, 14)), (0, 0), (0, 0), None)
            (Variable((4, 11), (12, 14)), (0, 2), (0, 0), Some((35, 64)))
            (Empty, (2, 2), (0, 0), None)
            (Section((0, 7), (10, 14)), (2, 2), (0, 0), None)
            (Instruction((4, 10)), (2, 3), (0, 0), None)
            (Empty, (3, 3), (0, 0), None)
            (Label((0, 6)), (3, 3), (0, 0), None)
            (Instruction((4, 7)), (3, 5), (0, 0), Some((32, 55)))
            (Instruction((4, 7)), (5, 7), (0, 0), Some((32, 57)))
            (Instruction((4, 7)), (7, 9), (0, 0), Some((32, 61)))
            (Instruction((4, 7)), (9, 11), (0, 0), Some((32, 49)))
            (Instruction((4, 11)), (11, 11), (0, 0), Some((32, 73)))
            (Instruction((4, 7)), (11, 13), (0, 0), Some((32, 54)))
            (Instruction((4, 7)), (13, 15), (0, 0), Some((32, 45)))
            (Instruction((4, 11)), (15, 15), (0, 0), Some((32, 65)))"#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            errors:

            literals:
            ((19, 34), String)
            ((36, 38), Decimal)
            ((40, 41), Decimal)
            ((16, 27), String)
            ((29, 31), Decimal)
            ((33, 34), Decimal)
            ((17, 49), String)
            ((51, 53), Decimal)
            ((55, 56), Decimal)
            ((11, 17), Ident)
            ((8, 11), Ident)
            ((13, 24), Ident)
            ((9, 14), Ident)
            ((8, 11), Ident)
            ((13, 21), Ident)
            ((9, 14), Ident)
            ((8, 11), Ident)
            ((13, 22), Ident)
            ((9, 14), Ident)
            ((8, 11), Ident)
            ((13, 15), Decimal)
            ((8, 11), Ident)
            ((13, 14), Decimal)
            ((9, 12), Ident)
            ((8, 11), Ident)
            ((13, 14), Decimal)
            ((8, 11), Ident)
            ((8, 11), Ident)
            ((8, 10), Ident)
            ((12, 17), Deref)
            ((8, 10), Ident)
            ((12, 13), Decimal)
            ((8, 18), Ident)
            ((8, 11), Ident)
            ((13, 14), Decimal)
            ((8, 11), Ident)
            ((13, 14), Decimal)
            ((8, 11), Ident)
            ((8, 11), Ident)
            ((13, 16), Ident)
            lines:
            (Section((0, 7), (8, 12)), (0, 0), (0, 0), None)
            (Variable((4, 15), (16, 18)), (0, 3), (0, 0), None)
            (Variable((4, 12), (13, 15)), (3, 6), (0, 0), None)
            (Variable((4, 13), (14, 16)), (6, 9), (0, 0), None)
            (Empty, (9, 9), (0, 0), None)
            (Section((0, 7), (8, 12)), (9, 9), (0, 0), None)
            (Instruction((4, 10)), (9, 10), (0, 0), None)
            (Empty, (10, 10), (0, 0), None)
            (Label((0, 6)), (10, 10), (0, 0), None)
            (Instruction((4, 7)), (10, 12), (0, 0), None)
            (Instruction((4, 8)), (12, 13), (0, 0), None)
            (Empty, (13, 13), (0, 0), None)
            (Instruction((4, 7)), (13, 15), (0, 0), None)
            (Instruction((4, 8)), (15, 16), (0, 0), None)
            (Empty, (16, 16), (0, 0), None)
            (Instruction((4, 7)), (16, 18), (0, 0), None)
            (Instruction((4, 8)), (18, 19), (0, 0), None)
            (Empty, (19, 19), (0, 0), None)
            (Instruction((4, 7)), (19, 21), (0, 0), None)
            (Instruction((4, 7)), (21, 23), (0, 0), None)
            (Instruction((4, 11)), (23, 23), (0, 0), None)
            (Label((0, 5)), (23, 23), (0, 0), None)
            (Instruction((4, 8)), (23, 24), (0, 0), None)
            (Instruction((4, 7)), (24, 26), (0, 0), None)
            (Label((0, 10)), (26, 26), (0, 0), None)
            (Instruction((4, 7)), (26, 27), (0, 0), None)
            (Instruction((4, 7)), (27, 28), (0, 0), None)
            (Instruction((4, 7)), (28, 30), (0, 0), None)
            (Instruction((4, 7)), (30, 32), (0, 0), None)
            (Instruction((4, 7)), (32, 33), (0, 0), None)
            (Empty, (33, 33), (0, 0), None)
            (Instruction((4, 7)), (33, 35), (0, 0), None)
            (Instruction((4, 7)), (35, 37), (0, 0), None)
            (Instruction((4, 7)), (37, 38), (0, 0), None)
            (Instruction((4, 7)), (38, 40), (0, 0), None)
            (Instruction((4, 11)), (40, 40), (0, 0), None)
            (Empty, (40, 40), (0, 0), None)
            (Instruction((4, 7)), (40, 40), (0, 0), None)"#]],
    );
}

#[test]
fn numerics() {
    check(
        "\
190238, 10928321, 0904832041, 3924092840238491019283210
0x1saklj90238SLKDJSD, 0b10101_1001, 0o172537162
",
        expect![[r#"
            errors:

            literals:
            ((0, 6), Decimal)
            ((8, 16), Decimal)
            ((18, 28), Decimal)
            ((30, 55), Decimal)
            ((0, 20), Hex)
            ((22, 34), Binary)
            ((36, 47), Octal)
            lines:
            (Empty, (0, 4), (0, 0), None)
            (Empty, (4, 7), (0, 0), None)"#]],
    );
}

#[test]
fn deref_err() {
    check(
        "\
one [ yeahhhhh
two [ nooooo 12309]
three [  12309 nooooo]
",
        expect![[r#"
            errors:
            ((4, 5), UnclosedDeref)
            ((4, 19), MuddyDeref)
            ((14, 15), MissingComma)
            ((6, 22), MuddyDeref)
            literals:
            ((6, 14), Ident)
            ((13, 18), Decimal)
            ((9, 14), Decimal)
            ((15, 21), Ident)
            lines:
            (Instruction((0, 3)), (0, 1), (0, 1), None)
            (Variable((0, 3), (6, 12)), (1, 2), (1, 2), None)
            (Instruction((0, 5)), (2, 4), (2, 4), None)"#]],
    );
}

#[test]
fn float() {
    check(
        "\
0.123.123.123
",
        expect![[r#"
            errors:

            literals:
            ((0, 13), Decimal)
            lines:
            (Empty, (0, 1), (0, 0), None)"#]],
    );
}

/// out of order
#[test]
fn ooo() {
    check(
        "\
123  yeah
123, yeah
",
        expect![[r#"
            errors:
            ((3, 5), MissingComma)
            literals:
            ((0, 3), Decimal)
            ((5, 9), Ident)
            ((0, 3), Decimal)
            ((5, 9), Ident)
            lines:
            (Empty, (0, 2), (0, 1), None)
            (Empty, (2, 4), (1, 1), None)"#]],
    );
}
#[test]
fn yeah() {
    check(
        "\
; one two three
    ; one two three
",
        expect![[r#"
            errors:

            literals:

            lines:
            (Empty, (0, 0), (0, 0), Some((0, 15)))
            (Empty, (0, 0), (0, 0), Some((4, 19)))"#]],
    );
}
