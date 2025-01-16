use expect_test::{expect, Expect};

fn debug_iter<D: std::fmt::Debug>(iter: impl Iterator<Item = D>) -> String {
    iter.map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn check(src: &str, expect: Expect) {
    let mut lexer = super::Lexer::new(src);
    let lins: Vec<_> = std::iter::from_fn(|| lexer.line()).collect();
    let lines = debug_iter(lins.iter().map(|l| (l.kind, l.literals, l.comment)));
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
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)"#]],
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
            (Empty, (0, 0), None)"#]],
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
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)"#]],
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
            (Instruction((0.1, 0.3)), (0, 0), None)
            (Instruction((1.2, 1.4)), (0, 0), None)
            (Instruction((2.3, 1.10)), (0, 0), None)
            (Instruction((3.4, 1.5)), (0, 0), None)"#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            errors:

            literals:
            ((0.1, 4.7), Ident)
            ((1.2, 5.13), Ident)
            ((2.3, 11.15), Ident)
            ((3.4, 6.10), Ident)
            lines:
            (Instruction((0.1, 0.3)), (0, 1), None)
            (Instruction((1.2, 1.4)), (1, 2), None)
            (Instruction((2.3, 1.10)), (2, 3), None)
            (Instruction((3.4, 1.5)), (3, 4), None)"#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            errors:

            literals:
            ((0.1, 4.7), Ident)
            ((0.1, 9.14), Ident)
            ((1.2, 5.13), Ident)
            ((1.2, 15.19), Ident)
            lines:
            (Instruction((0.1, 0.3)), (0, 2), None)
            (Instruction((1.2, 1.4)), (2, 4), None)"#]],
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
            (Empty, (0, 0), Some((0.1, 9.24)))
            (Empty, (0, 0), Some((1.2, 0.17)))
            (Empty, (0, 0), Some((2.3, 1.11)))"#]],
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
            ((1.2, 4.7), Ident)
            ((2.3, 4.7), Ident)
            ((2.3, 9.12), Ident)
            lines:
            (Instruction((0.1, 0.3)), (0, 0), Some((0.1, 12.27)))
            (Instruction((1.2, 0.3)), (0, 1), Some((1.2, 7.24)))
            (Instruction((2.3, 0.3)), (1, 3), Some((2.3, 13.23)))"#]],
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
            ((0.1, 13.16), Ident)
            lines:
            (Section((0.1, 8.12)), (0, 1), None)
            (Section((1.2, 8.11)), (1, 1), None)
            (Section((2.3, 8.12)), (1, 1), None)"#]],
    );
}
#[test]
fn comma_err() {
    check(
        "\
,
,,,, ,,
,one ,two,",
        expect![[r#"
            errors:

            literals:
            ((2.3, 1.4), Ident)
            ((2.3, 6.9), Ident)
            lines:
            (Empty, (0, 0), None)
            (Empty, (0, 0), None)
            (Empty, (0, 2), None)"#]],
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
            ((0.1, 0.1), Decimal)
            ((0.1, 3.7), Decimal)
            ((0.1, 9.16), Decimal)
            ((0.1, 18.25), Decimal)
            lines:
            (Empty, (0, 4), None)"#]],
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
            ((0.1, 7.22), String)
            ((0.1, 24.29), Decimal)
            ((0.1, 31.33), Decimal)
            lines:
            (Variable((0.1, 0.3), (0.1, 4.6)), (0, 3), None)"#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            errors:
            ((0.1, 22.23), MissingComma)
            ((0.1, 28.29), MissingComma)
            literals:
            ((0.1, 7.22), String)
            ((0.1, 23.28), Decimal)
            ((0.1, 29.31), Decimal)
            lines:
            (Variable((0.1, 0.3), (0.1, 4.6)), (0, 3), None)"#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("./test/0-hello-world.asm"),
        expect![[r#"
            errors:

            literals:
            ((1.2, 15.29), String)
            ((1.2, 31.33), Decimal)
            ((4.5, 14.20), Ident)
            ((7.8, 14.17), Ident)
            ((7.8, 19.20), Decimal)
            ((8.9, 14.17), Ident)
            ((8.9, 19.20), Decimal)
            ((9.10, 14.17), Ident)
            ((9.10, 19.26), Ident)
            ((10.11, 14.17), Ident)
            ((10.11, 19.21), Decimal)
            ((12.13, 14.17), Ident)
            ((12.13, 19.21), Decimal)
            ((13.14, 14.17), Ident)
            ((13.14, 19.22), Ident)
            lines:
            (Section((0.1, 10.14)), (0, 0), None)
            (Variable((1.2, 4.11), (1.2, 12.14)), (0, 2), Some((1.2, 35.64)))
            (Empty, (2, 2), None)
            (Section((3.4, 10.14)), (2, 2), None)
            (Instruction((4.5, 4.10)), (2, 3), None)
            (Empty, (3, 3), None)
            (Label((6.7, 0.6)), (3, 3), None)
            (Instruction((7.8, 4.7)), (3, 5), Some((7.8, 32.55)))
            (Instruction((8.9, 4.7)), (5, 7), Some((8.9, 32.57)))
            (Instruction((9.10, 4.7)), (7, 9), Some((9.10, 32.61)))
            (Instruction((10.11, 4.7)), (9, 11), Some((10.11, 32.49)))
            (Instruction((11.12, 4.11)), (11, 11), Some((11.12, 32.73)))
            (Instruction((12.13, 4.7)), (11, 13), Some((12.13, 32.54)))
            (Instruction((13.14, 4.7)), (13, 15), Some((13.14, 32.45)))
            (Instruction((14.15, 4.11)), (15, 15), Some((14.15, 32.65)))"#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("./test/1-print-any.asm"),
        expect![[r#"
            errors:

            literals:
            ((1.2, 19.34), String)
            ((1.2, 36.38), Decimal)
            ((2.3, 16.27), String)
            ((2.3, 29.31), Decimal)
            ((3.4, 17.49), String)
            ((3.4, 51.53), Decimal)
            ((6.7, 11.17), Ident)
            ((9.10, 8.11), Ident)
            ((9.10, 13.24), Ident)
            ((10.11, 9.14), Ident)
            ((12.13, 8.11), Ident)
            ((12.13, 13.21), Ident)
            ((13.14, 9.14), Ident)
            ((15.16, 8.11), Ident)
            ((15.16, 13.22), Ident)
            ((16.17, 9.14), Ident)
            ((18.19, 8.11), Ident)
            ((18.19, 13.15), Decimal)
            ((19.20, 8.11), Ident)
            ((22.23, 9.12), Ident)
            ((23.24, 8.11), Ident)
            ((25.26, 8.11), Ident)
            ((26.27, 8.11), Ident)
            ((27.28, 13.16), Deref)
            ((28.29, 8.10), Ident)
            ((29.30, 8.18), Ident)
            ((31.32, 8.11), Ident)
            ((31.32, 13.14), Decimal)
            ((32.33, 8.11), Ident)
            ((32.33, 13.14), Decimal)
            ((33.34, 8.11), Ident)
            ((34.35, 8.11), Ident)
            ((34.35, 13.16), Ident)
            lines:
            (Section((0.1, 8.12)), (0, 0), None)
            (Variable((1.2, 4.15), (1.2, 16.18)), (0, 2), None)
            (Variable((2.3, 4.12), (2.3, 13.15)), (2, 4), None)
            (Variable((3.4, 4.13), (3.4, 14.16)), (4, 6), None)
            (Empty, (6, 6), None)
            (Section((5.6, 8.12)), (6, 6), None)
            (Instruction((6.7, 4.10)), (6, 7), None)
            (Empty, (7, 7), None)
            (Label((8.9, 0.6)), (7, 7), None)
            (Instruction((9.10, 4.7)), (7, 9), None)
            (Instruction((10.11, 4.8)), (9, 10), None)
            (Empty, (10, 10), None)
            (Instruction((12.13, 4.7)), (10, 12), None)
            (Instruction((13.14, 4.8)), (12, 13), None)
            (Empty, (13, 13), None)
            (Instruction((15.16, 4.7)), (13, 15), None)
            (Instruction((16.17, 4.8)), (15, 16), None)
            (Empty, (16, 16), None)
            (Instruction((18.19, 4.7)), (16, 18), None)
            (Instruction((19.20, 4.7)), (18, 19), None)
            (Instruction((20.21, 4.11)), (19, 19), None)
            (Label((21.22, 0.5)), (19, 19), None)
            (Instruction((22.23, 4.8)), (19, 20), None)
            (Instruction((23.24, 4.7)), (20, 21), None)
            (Label((24.25, 0.10)), (21, 21), None)
            (Instruction((25.26, 4.7)), (21, 22), None)
            (Instruction((26.27, 4.7)), (22, 23), None)
            (Variable((27.28, 4.7), (27.28, 8.10)), (23, 24), None)
            (Instruction((28.29, 4.7)), (24, 25), None)
            (Instruction((29.30, 4.7)), (25, 26), None)
            (Empty, (26, 26), None)
            (Instruction((31.32, 4.7)), (26, 28), None)
            (Instruction((32.33, 4.7)), (28, 30), None)
            (Instruction((33.34, 4.7)), (30, 31), None)
            (Instruction((34.35, 4.7)), (31, 33), None)
            (Instruction((35.36, 4.11)), (33, 33), None)
            (Empty, (33, 33), None)
            (Instruction((37.38, 4.7)), (33, 33), None)"#]],
    );
}
