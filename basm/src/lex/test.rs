use expect_test::{expect, Expect};

use super::{LexOutput, Line};

// TODO: create a better stringify that assembles lines, errors, and literals
// into the actual order they appear in.
fn print_line<S: AsRef<str>>(l: &Line, lexer: &LexOutput<S>) -> String {
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
    let lexer = super::LexOutput::lex_all(src);
    let lines = lexer
        .lines
        .iter()
        .map(|l| print_line(&l.line, &lexer))
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
            Instruction
            	(0, 3) Ident
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 4) Ident
            	(4, 5) Whitespace
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 10) Ident
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 5) Ident
            None"#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 4) Ident
            	(4, 5) Whitespace
            	(5, 13) Ident
            	(13, 14) Whitespace
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 10) Ident
            	(10, 11) Whitespace
            	(11, 15) Ident
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 5) Ident
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
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Comma
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Instruction
            	(0, 1) Whitespace
            	(1, 4) Ident
            	(4, 5) Whitespace
            	(5, 13) Ident
            	(13, 14) Comma
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
            Instruction
            	(0, 3) Ident
            	(3, 12) Whitespace
            Some((12, 27))
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            Some((7, 24))
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Comma
            	(8, 9) Whitespace
            	(9, 12) Ident
            	(12, 13) Whitespace
            Some((13, 23))"#]],
    );
}
#[test]
fn label() {
    check(
        "\
l1      :
\t\tl2\t:
\t\t    l3      :",
        expect![[r#"
            Label
            	(0, 2) Ident
            	(2, 8) Whitespace
            	(8, 9) Colon
            None
            Label
            	(0, 2) Whitespace
            	(2, 4) Ident
            	(4, 5) Whitespace
            	(5, 6) Colon
            None
            Label
            	(0, 6) Whitespace
            	(6, 8) Ident
            	(8, 14) Whitespace
            	(14, 15) Colon
            None"#]],
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
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 12) Ident
            	(12, 13) Whitespace
            	(13, 16) Ident
            None
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 12) Ident
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
            	(1, 2) Comma
            	(2, 3) Whitespace
            	(3, 7) Decimal
            	(7, 8) Comma
            	(8, 9) Whitespace
            	(9, 16) Decimal
            	(16, 17) Comma
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
            Variable
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 6) Ident
            	(6, 7) Whitespace
            	(7, 22) String
            	(22, 23) Comma
            	(23, 24) Whitespace
            	(24, 29) Decimal
            	(29, 30) Comma
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
            Variable
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 6) Ident
            	(6, 7) Whitespace
            	(7, 22) String
            	(22, 23) Whitespace
            	(23, 28) Decimal
            	(28, 29) Whitespace
            	(29, 31) Decimal
            		(22, 23) MissingComma
            		(28, 29) MissingComma
            None"#]],
    );
}
// msg db "ONE TWO THREE" 12309 12
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            Section
            	(0, 7) Ident
            	(7, 10) Whitespace
            	(10, 14) Ident
            None
            Variable
            	(0, 4) Whitespace
            	(4, 11) Ident
            	(11, 12) Whitespace
            	(12, 14) Ident
            	(14, 15) Whitespace
            	(15, 29) String
            	(29, 30) Comma
            	(30, 31) Whitespace
            	(31, 33) Decimal
            	(33, 35) Whitespace
            Some((35, 64))
            Empty
            None
            Section
            	(0, 7) Ident
            	(7, 10) Whitespace
            	(10, 14) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 10) Ident
            	(10, 14) Whitespace
            	(14, 20) Ident
            None
            Empty
            None
            Label
            	(0, 6) Ident
            	(6, 7) Colon
            	(7, 8) Whitespace
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 32) Whitespace
            Some((32, 55))
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 32) Whitespace
            Some((32, 57))
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 26) Ident
            	(26, 32) Whitespace
            Some((32, 61))
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 21) Decimal
            	(21, 32) Whitespace
            Some((32, 49))
            Instruction
            	(0, 4) Whitespace
            	(4, 11) Ident
            	(11, 32) Whitespace
            Some((32, 73))
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 21) Decimal
            	(21, 32) Whitespace
            Some((32, 54))
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 14) Whitespace
            	(14, 17) Ident
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 22) Ident
            	(22, 32) Whitespace
            Some((32, 45))
            Instruction
            	(0, 4) Whitespace
            	(4, 11) Ident
            	(11, 32) Whitespace
            Some((32, 65))"#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 12) Ident
            None
            Variable
            	(0, 4) Whitespace
            	(4, 15) Ident
            	(15, 16) Whitespace
            	(16, 18) Ident
            	(18, 19) Whitespace
            	(19, 34) String
            	(34, 35) Comma
            	(35, 36) Whitespace
            	(36, 38) Decimal
            	(38, 39) Comma
            	(39, 40) Whitespace
            	(40, 41) Decimal
            None
            Variable
            	(0, 4) Whitespace
            	(4, 12) Ident
            	(12, 13) Whitespace
            	(13, 15) Ident
            	(15, 16) Whitespace
            	(16, 27) String
            	(27, 28) Comma
            	(28, 29) Whitespace
            	(29, 31) Decimal
            	(31, 32) Comma
            	(32, 33) Whitespace
            	(33, 34) Decimal
            None
            Variable
            	(0, 4) Whitespace
            	(4, 13) Ident
            	(13, 14) Whitespace
            	(14, 16) Ident
            	(16, 17) Whitespace
            	(17, 49) String
            	(49, 50) Comma
            	(50, 51) Whitespace
            	(51, 53) Decimal
            	(53, 54) Comma
            	(54, 55) Whitespace
            	(55, 56) Decimal
            None
            Empty
            None
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 12) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 10) Ident
            	(10, 11) Whitespace
            	(11, 17) Ident
            None
            Empty
            None
            Label
            	(0, 6) Ident
            	(6, 7) Colon
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 24) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 8) Ident
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Empty
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 21) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 8) Ident
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Empty
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 22) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 8) Ident
            	(8, 9) Whitespace
            	(9, 14) Ident
            None
            Empty
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 15) Decimal
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 11) Ident
            None
            Label
            	(0, 5) Ident
            	(5, 6) Colon
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 8) Ident
            	(8, 9) Whitespace
            	(9, 12) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Label
            	(0, 10) Ident
            	(10, 11) Colon
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 10) Ident
            	(10, 11) Comma
            	(11, 12) Whitespace
            	(12, 13) Other
            	(13, 16) Ident
            	(16, 17) Other
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 10) Ident
            	(10, 11) Comma
            	(11, 12) Whitespace
            	(12, 13) Decimal
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 18) Ident
            None
            Empty
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 14) Decimal
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Comma
            	(12, 13) Whitespace
            	(13, 16) Ident
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 11) Ident
            None
            Empty
            None
            Instruction
            	(0, 4) Whitespace
            	(4, 7) Ident
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
            	(6, 7) Comma
            	(7, 8) Whitespace
            	(8, 16) Decimal
            	(16, 17) Comma
            	(17, 18) Whitespace
            	(18, 28) Decimal
            	(28, 29) Comma
            	(29, 30) Whitespace
            	(30, 55) Decimal
            None
            Empty
            	(0, 20) Hex
            	(20, 21) Comma
            	(21, 22) Whitespace
            	(22, 34) Binary
            	(34, 35) Comma
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
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 5) Other
            	(5, 6) Whitespace
            	(6, 14) Ident
            None
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 5) Other
            	(5, 6) Whitespace
            	(6, 12) Ident
            	(12, 13) Whitespace
            	(13, 18) Decimal
            	(18, 19) Other
            None
            Instruction
            	(0, 5) Ident
            	(5, 6) Whitespace
            	(6, 7) Other
            	(7, 9) Whitespace
            	(9, 14) Decimal
            	(14, 15) Whitespace
            	(15, 21) Ident
            	(21, 22) Other
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
            Instruction
            	(0, 3) Decimal
            	(3, 5) Whitespace
            	(5, 9) Ident
            None
            Empty
            	(0, 3) Decimal
            	(3, 4) Comma
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
            Label
            	(0, 5) Ident
            	(5, 6) Colon
            	(6, 7) Whitespace
            	(7, 8) Colon
            	(8, 9) Whitespace
            	(9, 10) Colon
            		(8, 9) MissingComma
            None
            Label
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Colon
            	(12, 13) Whitespace
            	(13, 14) Colon
            	(14, 15) Whitespace
            	(15, 16) Colon
            		(8, 11) MissingComma
            		(12, 13) MissingComma
            		(14, 15) MissingComma
            None
            Variable
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 13) String
            	(13, 14) Comma
            	(14, 15) Whitespace
            	(15, 17) Decimal
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 21) Colon
            	(21, 22) Whitespace
            	(22, 23) Colon
            	(23, 24) Whitespace
            	(24, 25) Colon
            		(19, 20) MissingComma
            		(21, 22) MissingComma
            		(23, 24) MissingComma
            None
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 15) Ident
            	(15, 16) Colon
            	(16, 17) Whitespace
            	(17, 18) Colon
            	(18, 19) Whitespace
            	(19, 20) Colon
            		(16, 17) MissingComma
            		(18, 19) MissingComma
            None
            Empty
            	(0, 1) Colon
            	(1, 2) Whitespace
            	(2, 3) Colon
            	(3, 4) Whitespace
            	(4, 5) Colon
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
            Label
            	(0, 5) Ident
            	(5, 6) Colon
            	(6, 7) Whitespace
            	(7, 14) Other
            None
            Instruction
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 11) Ident
            	(11, 12) Whitespace
            	(12, 19) Other
            None
            Variable
            	(0, 3) Ident
            	(3, 4) Whitespace
            	(4, 7) Ident
            	(7, 8) Whitespace
            	(8, 13) String
            	(13, 14) Comma
            	(14, 15) Whitespace
            	(15, 17) Decimal
            	(17, 18) Comma
            	(18, 19) Whitespace
            	(19, 20) Decimal
            	(20, 27) Other
            None
            Section
            	(0, 7) Ident
            	(7, 8) Whitespace
            	(8, 15) Ident
            	(15, 22) Other
            None
            Empty
            	(0, 1) Whitespace
            	(1, 8) Other
            Some((8, 15))"#]],
    );
}

#[test]
fn empty_not_instruction() {
    check(
        "one, two, three\n two, threeeee, four",
        expect![[r#"
            Empty
            	(0, 3) Ident
            	(3, 4) Comma
            	(4, 5) Whitespace
            	(5, 8) Ident
            	(8, 9) Comma
            	(9, 10) Whitespace
            	(10, 15) Ident
            None
            Empty
            	(0, 1) Whitespace
            	(1, 4) Ident
            	(4, 5) Comma
            	(5, 6) Whitespace
            	(6, 14) Ident
            	(14, 15) Comma
            	(15, 16) Whitespace
            	(16, 20) Ident
            None"#]],
    );
}
