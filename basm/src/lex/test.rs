use expect_test::{expect, Expect};

use super::{Lexer, Line};

// TODO: create a better stringify that assembles lines, errors, and literals
// into the actual order they appear in.
fn print_line(l: &Line, lexer: &Lexer) -> String {
    format!(
        "{:?}\n{}{}{:?}",
        l.kind,
        l.slice_lit(&lexer.literals)
            .iter()
            .map(|(s, l)| format!("\t{s:?} {l:?}\n"))
            .collect::<String>(),
        l.slice_err(&lexer.errors)
            .iter()
            .map(|(s, e)| format!("\t\t{s:?} {e:?}\n"))
            .collect::<String>(),
        l.comment
    )
}

fn check(src: &str, expect: Expect) {
    let mut lexer = super::Lexer::new(src);
    let lines: Vec<_> = std::iter::from_fn(|| lexer.line()).collect();
    let lines = lines
        .iter()
        .map(|l| print_line(l, &lexer))
        .collect::<Vec<_>>()
        .join("\n");
    expect.assert_eq(&lines);
}

#[test]
fn empty() {
    check("", expect![""]);
}
#[test]
fn multi_empty() {
    check(
        "\n\n\n\n",
        expect![[r#"
            Empty
            None
            Empty
            None
            Empty
            None
            Empty
            None"#]],
    );
}
#[test]
fn empty_ws() {
    check(
        "\t\t    \t     \t ",
        expect![[r#"
            Empty
            	(0, 14) Whitespace
            None"#]],
    );
}
#[test]
fn multi_empty_ws() {
    check(
        "\t\t    \t     \t \n\n\t\t    \t     \t \n\n",
        expect![[r#"
            Empty
            	(0, 14) Whitespace
            None
            Empty
            None
            Empty
            	(0, 14) Whitespace
            None
            Empty
            None"#]],
    );
}
#[test]
fn singles() {
    check(
        "one\n two\t\n threeeeee\n four\n",
        expect![[r#"
            Instruction((0, 3))
            None
            Instruction((1, 4))
            	(0, 1) Whitespace
            	(4, 5) Whitespace
            None
            Instruction((1, 10))
            	(0, 1) Whitespace
            None
            Instruction((1, 5))
            	(0, 1) Whitespace
            None"#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            None
            Instruction((1, 4))
            	(0, 1) Whitespace
            	(4, 5) Whitespace
            	(5, 13) Ident
            	(13, 14) Whitespace
            None
            Instruction((1, 10))
            	(0, 1) Whitespace
            	(10, 11) Whitespace
            	(11, 15) Ident
            None
            Instruction((1, 5))
            	(0, 1) Whitespace
            	(5, 6) Whitespace
            	(6, 10) Ident
            None"#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Instruction((1, 4))
            	(0, 1) Whitespace
            	(4, 5) Whitespace
            	(5, 13) Ident
            	(14, 15) Whitespace
            	(15, 19) Ident
            None"#]],
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
            Empty
            	(0, 9) Whitespace
            Some((9, 24))
            Empty
            Some((0, 17))
            Empty
            	(0, 1) Whitespace
            Some((1, 11))"#]],
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
            Instruction((0, 3))
            	(3, 12) Whitespace
            Some((12, 27))
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            Some((7, 24))
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(8, 9) Whitespace
            	(9, 12) Ident
            	(12, 13) Whitespace
            Some((13, 23))"#]],
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
            Section((0, 7), (8, 12))
            	(7, 8) Whitespace
            	(12, 13) Whitespace
            	(13, 16) Ident
            None
            Section((0, 7), (8, 11))
            	(7, 8) Whitespace
            None
            Section((0, 7), (8, 12))
            	(7, 8) Whitespace
            None"#]],
    );
}
#[test]
fn decimal() {
    check(
        "\
1, 1293, 9384093, 1231234",
        expect![[r#"
            Empty
            	(0, 1) Decimal
            	(2, 3) Whitespace
            	(3, 7) Decimal
            	(8, 9) Whitespace
            	(9, 16) Decimal
            	(17, 18) Whitespace
            	(18, 25) Decimal
            None"#]],
    );
}
#[test]
fn variable() {
    check(
        "\
msg db \"ONE TWO THREE\", 12309, 12",
        expect![[r#"
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 6) Ident
            	(6, 7) Whitespace
            	(7, 22) String
            	(23, 24) Whitespace
            	(24, 29) Decimal
            	(30, 31) Whitespace
            	(31, 33) Decimal
            None"#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 6) Ident
            	(6, 7) Whitespace
            	(7, 22) String
            	(22, 23) Whitespace
            	(23, 28) Decimal
            	(28, 29) Whitespace
            	(29, 31) Decimal
            		(22, 24) MissingComma
            		(28, 30) MissingComma
            None"#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            Section((0, 7), (10, 14))
            	(7, 10) Whitespace
            None
            Instruction((4, 11))
            	(0, 4) Whitespace
            	(11, 12) Whitespace
            	(12, 14) Ident
            	(14, 15) Whitespace
            	(15, 29) String
            	(30, 31) Whitespace
            	(31, 33) Decimal
            	(33, 35) Whitespace
            Some((35, 64))
            Empty
            None
            Section((0, 7), (10, 14))
            	(7, 10) Whitespace
            None
            Instruction((4, 10))
            	(0, 4) Whitespace
            	(10, 14) Whitespace
            	(14, 20) Ident
            None
            Empty
            None
            Label((0, 6))
            	(7, 8) Whitespace
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 32) Whitespace
            Some((32, 55))
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 32) Whitespace
            Some((32, 57))
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(18, 19) Whitespace
            	(19, 26) Ident
            	(26, 32) Whitespace
            Some((32, 61))
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(18, 19) Whitespace
            	(19, 21) Decimal
            	(21, 32) Whitespace
            Some((32, 49))
            Instruction((4, 11))
            	(0, 4) Whitespace
            	(11, 32) Whitespace
            Some((32, 73))
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(18, 19) Whitespace
            	(19, 21) Decimal
            	(21, 32) Whitespace
            Some((32, 54))
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(18, 19) Whitespace
            	(19, 22) Ident
            	(22, 32) Whitespace
            Some((32, 45))
            Instruction((4, 11))
            	(0, 4) Whitespace
            	(11, 32) Whitespace
            Some((32, 65))"#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            Section((0, 7), (8, 12))
            	(7, 8) Whitespace
            None
            Instruction((4, 15))
            	(0, 4) Whitespace
            	(15, 16) Whitespace
            	(16, 18) Ident
            	(18, 19) Whitespace
            	(19, 34) String
            	(35, 36) Whitespace
            	(36, 38) Decimal
            	(39, 40) Whitespace
            	(40, 41) Decimal
            None
            Instruction((4, 12))
            	(0, 4) Whitespace
            	(12, 13) Whitespace
            	(13, 15) Ident
            	(15, 16) Whitespace
            	(16, 27) String
            	(28, 29) Whitespace
            	(29, 31) Decimal
            	(32, 33) Whitespace
            	(33, 34) Decimal
            None
            Instruction((4, 13))
            	(0, 4) Whitespace
            	(13, 14) Whitespace
            	(14, 16) Ident
            	(16, 17) Whitespace
            	(17, 49) String
            	(50, 51) Whitespace
            	(51, 53) Decimal
            	(54, 55) Whitespace
            	(55, 56) Decimal
            None
            Empty
            None
            Section((0, 7), (8, 12))
            	(7, 8) Whitespace
            None
            Instruction((4, 10))
            	(0, 4) Whitespace
            	(10, 11) Whitespace
            	(11, 17) Ident
            None
            Empty
            None
            Label((0, 6))
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 24) Ident
            None
            Instruction((4, 8))
            	(0, 4) Whitespace
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Empty
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 21) Ident
            None
            Instruction((4, 8))
            	(0, 4) Whitespace
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Empty
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 22) Ident
            None
            Instruction((4, 8))
            	(0, 4) Whitespace
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Empty
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 15) Decimal
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Instruction((4, 11))
            	(0, 4) Whitespace
            None
            Label((0, 5))
            None
            Instruction((4, 8))
            	(0, 4) Whitespace
            	(8, 9) Whitespace
            	(9, 12) Ident
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Label((0, 10))
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 10) Ident
            	(11, 12) Whitespace
            	(12, 17) Deref
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 10) Ident
            	(11, 12) Whitespace
            	(12, 13) Decimal
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 18) Ident
            None
            Empty
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 16) Ident
            None
            Instruction((4, 11))
            	(0, 4) Whitespace
            None
            Empty
            None
            Instruction((4, 7))
            	(0, 4) Whitespace
            None"#]],
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
            Empty
            	(0, 6) Decimal
            	(7, 8) Whitespace
            	(8, 16) Decimal
            	(17, 18) Whitespace
            	(18, 28) Decimal
            	(29, 30) Whitespace
            	(30, 55) Decimal
            None
            Empty
            	(0, 20) Hex
            	(21, 22) Whitespace
            	(22, 34) Binary
            	(35, 36) Whitespace
            	(36, 47) Octal
            None"#]],
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
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(5, 6) Whitespace
            	(6, 14) Ident
            		(4, 5) UnclosedDeref
            None
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(5, 6) Whitespace
            	(6, 12) Ident
            	(12, 13) Whitespace
            	(13, 18) Decimal
            		(4, 19) MuddyDeref
            None
            Instruction((0, 5))
            	(5, 6) Whitespace
            	(7, 9) Whitespace
            	(9, 14) Decimal
            	(14, 15) Whitespace
            	(15, 21) Ident
            		(6, 22) MuddyDeref
            None"#]],
    );
}

#[test]
fn float() {
    check(
        "\
0.123.123.123
",
        expect![[r#"
            Empty
            	(0, 13) Decimal
            None"#]],
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
            Empty
            	(0, 3) Decimal
            	(3, 5) Whitespace
            	(5, 9) Ident
            None
            Empty
            	(0, 3) Decimal
            	(4, 5) Whitespace
            	(5, 9) Ident
            None"#]],
    );
}

#[test]
fn other_comment() {
    check(
        "\
; one two three
    ; one two three
",
        expect![[r#"
            Empty
            Some((0, 15))
            Empty
            	(0, 4) Whitespace
            Some((4, 19))"#]],
    );
}

#[test]
fn colon_other() {
    check(
        "\
label: : :
var var var: : :
var var \"var\", 10, 0: : :
section section: : :
: : : ;empty!
",
        expect![[r#"
            Label((0, 5))
            	(6, 7) Whitespace
            	(7, 8) Other
            	(8, 9) Whitespace
            	(9, 10) Other
            None
            Label((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(12, 13) Whitespace
            	(13, 14) Other
            	(14, 15) Whitespace
            	(15, 16) Other
            None
            Label((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 13) String
            	(14, 15) Whitespace
            	(15, 17) Decimal
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(21, 22) Whitespace
            	(22, 23) Other
            	(23, 24) Whitespace
            	(24, 25) Other
            None
            Section((0, 7), (8, 15))
            	(7, 8) Whitespace
            	(15, 16) Other
            	(16, 17) Whitespace
            	(17, 18) Other
            	(18, 19) Whitespace
            	(19, 20) Other
            None
            Empty
            	(0, 1) Other
            	(1, 2) Whitespace
            	(2, 3) Other
            	(3, 4) Whitespace
            	(4, 5) Other
            	(5, 6) Whitespace
            Some((6, 13))"#]],
    );
}

#[test]
fn rand_other() {
    check(
        "\
label: `~]]./\\
var var var `~]]./\\
var var \"var\", 10, 0`~]]./\\
section section`~]]./\\
 `~]]./\\;empty!
",
        expect![[r#"
            Label((0, 5))
            	(6, 7) Whitespace
            	(7, 14) Other
            None
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Whitespace
            	(12, 19) Other
            None
            Instruction((0, 3))
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 13) String
            	(14, 15) Whitespace
            	(15, 17) Decimal
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 27) Other
            None
            Section((0, 7), (8, 15))
            	(7, 8) Whitespace
            	(15, 22) Other
            None
            Empty
            	(0, 1) Whitespace
            	(1, 8) Other
            Some((8, 15))"#]],
    );
}
