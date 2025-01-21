use expect_test::{expect, Expect};

use super::Parser;

fn check(src: &str, expect: Expect) {
    let (basm, errors) = Parser::new(src).parse();
    let output = format!(
        "{}{}{}{}",
        basm.sections
            .into_iter()
            .map(|s| format!("{s:?}\n"))
            .collect::<String>(),
        basm.labels
            .into_iter()
            .map(|(s, a)| format!("{s:?}: {a}\n"))
            .collect::<String>(),
        basm.lines
            .into_iter()
            .map(|l| format!("{l:?}\n"))
            .collect::<String>(),
        errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<String>()
    );
    expect.assert_eq(&output);
}

#[test]
fn empty() {
    check(
        "",
        expect![[r#"
            NoOp
        "#]],
    );
}
#[test]
fn multi_empty() {
    check(
        "\n\n\n\n",
        expect![[r#"
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
        "#]],
    );
}
#[test]
fn empty_ws() {
    check(
        "\t\t    \t     \t ",
        expect![[r#"
            NoOp
        "#]],
    );
}
#[test]
fn multi_empty_ws() {
    check(
        "\t\t    \t     \t \n\n\t\t    \t     \t \n\n",
        expect![[r#"
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
        "#]],
    );
}
#[test]
fn singles() {
    check(
        "one\n two\t\n threeeeee\n four\n",
        expect![[r#"
            Instruction { values: [Ident((0, 3))] }
            Instruction { values: [Ident((5, 8))] }
            Instruction { values: [Ident((11, 20))] }
            Instruction { values: [Ident((22, 26))] }
            NoOp
        "#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            Instruction { values: [] }
            Instruction { values: [] }
            Instruction { values: [] }
            Instruction { values: [] }
            NoOp
        "#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            NoOp
            missing comma at 0:14,15
            missing comma at 1:19,20
        "#]],
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
            NoOp
            NoOp
            NoOp
            NoOp
        "#]],
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
            Instruction { values: [Ident((0, 3))] }
            Instruction { values: [Ident((62, 65))] }
            NoOp
            missing comma at 2:3,4
            unexpected input found at 2:7,8. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Comma
        "#]],
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
            NoOp
            NoOp
            NoOp
            unexpected input found at 0:8,9. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
            unexpected input found at 1:5,6. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
            unexpected input found at 2:14,15. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
        "#]],
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
            Section { kind: Data, address: 0 }
            Section { kind: Bss, address: 0 }
            Section { kind: Text, address: 0 }
            NoOp
            unexpected input found at 0:13,16. expected [Whitespace, Eol(false), Eol(true), Eof] but found Ident
        "#]],
    );
}
#[test]
fn decimal() {
    check(
        "\
1, 1293, 9384093, 1231234",
        expect![[r#"
            NoOp
            unexpected input found at 0:0,1. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:1,2. expected [Ident] but found Comma
            unexpected input found at 0:3,7. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:7,8. expected [Ident] but found Comma
            unexpected input found at 0:9,16. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:16,17. expected [Ident] but found Comma
            unexpected input found at 0:18,25. expected [Ident] but found Digit(Decimal)
        "#]],
    );
}
#[test]
fn variable() {
    check(
        "\
msg db \"ONE TWO THREE\", 12309, 12
digit reb 100",
        expect![[r#"
            NoOp
            missing comma at 0:33,34
            missing comma at 1:13,14
        "#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            NoOp
            missing comma at 0:22,23
            unexpected input found at 0:23,28. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:29,31. expected [Ident] but found Digit(Decimal)
        "#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            Section { kind: Data, address: 0 }
            Section { kind: Text, address: 2 }
            NoOp
            NoOp
            Instruction { values: [] }
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            Instruction { values: [Ident((357, 364))] }
            NoOp
            NoOp
            Instruction { values: [Ident((532, 539))] }
            NoOp
            missing comma at 1:33,35
            unexpected input found at 6:6,7. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
            missing comma at 7:20,32
            missing comma at 8:20,32
            missing comma at 9:26,32
            missing comma at 10:21,32
            missing comma at 12:21,32
            missing comma at 13:22,32
        "#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            Section { kind: Data, address: 0 }
            Section { kind: Text, address: 3 }
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            unexpected input found at 1:4, 15. expected [] but found Ident
            unexpected input found at 1:16, 18. expected [] but found Ident
            unexpected input found at 1:19, 34. expected [] but found String
            unexpected input found at 1:34, 35. expected [Ident] but found Comma
            unexpected input found at 1:36, 38. expected [] but found Digit(Decimal)
            unexpected input found at 1:38, 39. expected [Ident] but found Comma
            unexpected input found at 1:40, 41. expected [] but found Digit(Decimal)
            unexpected input found at 2:4, 12. expected [] but found Ident
            unexpected input found at 2:13, 15. expected [] but found Ident
            unexpected input found at 2:16, 27. expected [] but found String
            unexpected input found at 2:27, 28. expected [Ident] but found Comma
            unexpected input found at 2:29, 31. expected [] but found Digit(Decimal)
            unexpected input found at 2:31, 32. expected [Ident] but found Comma
            unexpected input found at 2:33, 34. expected [] but found Digit(Decimal)
            unexpected input found at 3:4, 13. expected [] but found Ident
            unexpected input found at 3:14, 16. expected [] but found Ident
            unexpected input found at 3:17, 49. expected [] but found String
            unexpected input found at 3:49, 50. expected [Ident] but found Comma
            unexpected input found at 3:51, 53. expected [] but found Digit(Decimal)
            unexpected input found at 3:53, 54. expected [Ident] but found Comma
            unexpected input found at 3:55, 56. expected [] but found Digit(Decimal)
            unexpected input found at 6:4, 10. expected [] but found Ident
            unexpected input found at 6:11, 17. expected [] but found Ident
            unexpected input found at 8:6, 7. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
            unexpected input found at 9:4, 7. expected [] but found Ident
            unexpected input found at 9:8, 11. expected [] but found Ident
            unexpected input found at 9:11, 12. expected [Ident] but found Comma
            unexpected input found at 9:13, 24. expected [] but found Ident
            unexpected input found at 10:4, 8. expected [] but found Ident
            unexpected input found at 10:9, 14. expected [] but found Ident
            unexpected input found at 12:4, 7. expected [] but found Ident
            unexpected input found at 12:8, 11. expected [] but found Ident
            unexpected input found at 12:11, 12. expected [Ident] but found Comma
            unexpected input found at 12:13, 21. expected [] but found Ident
            unexpected input found at 13:4, 8. expected [] but found Ident
            unexpected input found at 13:9, 14. expected [] but found Ident
            unexpected input found at 15:4, 7. expected [] but found Ident
            unexpected input found at 15:8, 11. expected [] but found Ident
            unexpected input found at 15:11, 12. expected [Ident] but found Comma
            unexpected input found at 15:13, 22. expected [] but found Ident
            unexpected input found at 16:4, 8. expected [] but found Ident
            unexpected input found at 16:9, 14. expected [] but found Ident
            unexpected input found at 18:4, 7. expected [] but found Ident
            unexpected input found at 18:8, 11. expected [] but found Ident
            unexpected input found at 18:11, 12. expected [Ident] but found Comma
            unexpected input found at 18:13, 15. expected [] but found Digit(Decimal)
            unexpected input found at 19:4, 7. expected [] but found Ident
            unexpected input found at 19:8, 11. expected [] but found Ident
            unexpected input found at 19:11, 12. expected [Ident] but found Comma
            unexpected input found at 19:13, 14. expected [] but found Digit(Decimal)
            unexpected input found at 20:4, 11. expected [] but found Ident
            unexpected input found at 21:0, 5. expected [] but found Ident
            unexpected input found at 21:5, 6. expected [Ident] but found Colon
            unexpected input found at 22:4, 8. expected [] but found Ident
            unexpected input found at 22:9, 12. expected [] but found Ident
            unexpected input found at 23:4, 7. expected [] but found Ident
            unexpected input found at 23:8, 11. expected [] but found Ident
            unexpected input found at 23:11, 12. expected [Ident] but found Comma
            unexpected input found at 23:13, 14. expected [] but found Digit(Decimal)
            unexpected input found at 24:0, 10. expected [] but found Ident
            unexpected input found at 24:10, 11. expected [Ident] but found Colon
            unexpected input found at 25:4, 7. expected [] but found Ident
            unexpected input found at 25:8, 11. expected [] but found Ident
            unexpected input found at 26:4, 7. expected [] but found Ident
            unexpected input found at 26:8, 11. expected [] but found Ident
            unexpected input found at 27:4, 7. expected [] but found Ident
            unexpected input found at 27:8, 10. expected [] but found Ident
            unexpected input found at 27:10, 11. expected [Ident] but found Comma
            unexpected input found at 27:12, 13. expected [] but found OpenBracket
            unexpected input found at 27:16, 17. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found CloseBracket
            unexpected input found at 28:4, 7. expected [] but found Ident
            unexpected input found at 28:8, 10. expected [] but found Ident
            unexpected input found at 28:10, 11. expected [Ident] but found Comma
            unexpected input found at 28:12, 13. expected [] but found Digit(Decimal)
            unexpected input found at 29:4, 7. expected [] but found Ident
            unexpected input found at 29:8, 18. expected [] but found Ident
            unexpected input found at 31:4, 7. expected [] but found Ident
            unexpected input found at 31:8, 11. expected [] but found Ident
            unexpected input found at 31:11, 12. expected [Ident] but found Comma
            unexpected input found at 31:13, 14. expected [] but found Digit(Decimal)
            unexpected input found at 32:4, 7. expected [] but found Ident
            unexpected input found at 32:8, 11. expected [] but found Ident
            unexpected input found at 32:11, 12. expected [Ident] but found Comma
            unexpected input found at 32:13, 14. expected [] but found Digit(Decimal)
            unexpected input found at 33:4, 7. expected [] but found Ident
            unexpected input found at 33:8, 11. expected [] but found Ident
            unexpected input found at 34:4, 7. expected [] but found Ident
            unexpected input found at 34:8, 11. expected [] but found Ident
            unexpected input found at 34:11, 12. expected [Ident] but found Comma
            unexpected input found at 34:13, 16. expected [] but found Ident
            unexpected input found at 35:4, 11. expected [] but found Ident
            unexpected input found at 37:4, 7. expected [] but found Ident
        "#]],
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
            NoOp
            NoOp
            NoOp
            unexpected input found at 0:0,6. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:6,7. expected [Ident] but found Comma
            unexpected input found at 0:8,16. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:16,17. expected [Ident] but found Comma
            unexpected input found at 0:18,28. expected [Ident] but found Digit(Decimal)
            unexpected input found at 0:28,29. expected [Ident] but found Comma
            unexpected input found at 0:30,55. expected [Ident] but found Digit(Decimal)
            unexpected input found at 1:0,3. expected [Ident] but found Digit(Hex)
            unexpected input found at 1:20,21. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Comma
            unexpected input found at 1:22,34. expected [Ident] but found Digit(Binary)
            unexpected input found at 1:34,35. expected [Ident] but found Comma
            unexpected input found at 1:36,47. expected [Ident] but found Digit(Octal)
        "#]],
    );
}

#[test]
fn deref() {
    check(
        "\
one [ yeahhhhh
two [ nooooo 12309]
three [  12309 nooooo]
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Whitespace
            	0:(4, 5)=OpenBracket
            	0:(5, 6)=Whitespace
            	0:(6, 14)=Ident
            0:(14, 15)=Eol(false)
            	15:(15, 18)=Ident
            	15:(18, 19)=Whitespace
            	15:(19, 20)=OpenBracket
            	15:(20, 21)=Whitespace
            	15:(21, 27)=Ident
            	15:(27, 28)=Whitespace
            	15:(28, 33)=Digit(Decimal)
            	15:(33, 34)=CloseBracket
            15:(34, 35)=Eol(false)
            	35:(35, 40)=Ident
            	35:(40, 41)=Whitespace
            	35:(41, 42)=OpenBracket
            	35:(42, 44)=Whitespace
            	35:(44, 49)=Digit(Decimal)
            	35:(49, 50)=Whitespace
            	35:(50, 56)=Ident
            	35:(56, 57)=CloseBracket
            35:(57, 58)=Eol(false)"#]],
    );
}

#[test]
fn float() {
    check(
        "\
0.123.123.123
",
        expect![[r#"
            NoOp
            NoOp
            unexpected input found at 0:0,1. expected [Ident] but found Digit(Decimal)
            unknown input found at 0:1,2
            unexpected input found at 0:2,5. expected [Ident] but found Digit(Decimal)
            unknown input found at 0:5,6
            unexpected input found at 0:6,9. expected [Ident] but found Digit(Decimal)
            unknown input found at 0:9,10
            unexpected input found at 0:10,13. expected [Ident] but found Digit(Decimal)
        "#]],
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
            Instruction { values: [Ident((5, 9))] }
            Instruction { values: [Ident((15, 19))] }
            NoOp
            unexpected input found at 0:0,3. expected [Ident] but found Digit(Decimal)
            unexpected input found at 1:0,3. expected [Ident] but found Digit(Decimal)
            unexpected input found at 1:3,4. expected [Ident] but found Comma
        "#]],
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
            NoOp
            NoOp
            NoOp
        "#]],
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
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            unexpected input found at 0:5,6. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
            unexpected input found at 0:7,8. expected [Ident] but found Colon
            unexpected input found at 0:9,10. expected [Ident] but found Colon
            missing comma at 1:11,12
            unexpected input found at 1:13,14. expected [Ident] but found Colon
            unexpected input found at 1:15,16. expected [Ident] but found Colon
            missing comma at 2:20,21
            unexpected input found at 2:22,23. expected [Ident] but found Colon
            unexpected input found at 2:24,25. expected [Ident] but found Colon
            variable at line 3 had invalid type: Ident((62, 69))
            unexpected input found at 3:15,16. expected [Ident] but found Colon
            unexpected input found at 3:17,18. expected [Ident] but found Colon
            unexpected input found at 3:19,20. expected [Ident] but found Colon
            unexpected input found at 4:0,1. expected [Ident] but found Colon
            unexpected input found at 4:2,3. expected [Ident] but found Colon
            unexpected input found at 4:4,5. expected [Ident] but found Colon
        "#]],
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
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            NoOp
            unexpected input found at 0:5,6. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Colon
            unknown input found at 0:7,9
            unexpected input found at 0:9,10. expected [Ident] but found CloseBracket
            unexpected input found at 0:10,11. expected [Ident] but found CloseBracket
            unknown input found at 0:11,14
            missing comma at 1:11,12
            unknown input found at 1:12,14
            unexpected input found at 1:14,15. expected [Ident] but found CloseBracket
            unexpected input found at 1:15,16. expected [Ident] but found CloseBracket
            unknown input found at 1:16,19
            missing comma at 2:20,22
            unexpected input found at 2:22,23. expected [Ident] but found CloseBracket
            unexpected input found at 2:23,24. expected [Ident] but found CloseBracket
            unknown input found at 2:24,27
            variable at line 3 had invalid type: Ident((71, 78))
            unknown input found at 3:15,17
            unexpected input found at 3:17,18. expected [Ident] but found CloseBracket
            unexpected input found at 3:18,19. expected [Ident] but found CloseBracket
            unknown input found at 3:19,22
            unknown input found at 4:1,3
            unexpected input found at 4:3,4. expected [Ident] but found CloseBracket
            unexpected input found at 4:4,5. expected [Ident] but found CloseBracket
            unknown input found at 4:5,8
        "#]],
    );
}

#[test]
fn empty_not_instruction() {
    check(
        "one, two, three\n two, threeeee, four",
        expect![[r#"
            Instruction { values: [Ident((10, 15))] }
            Instruction { values: [Ident((32, 36))] }
            NoOp
            unexpected input found at 0:3,4. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Comma
            unexpected input found at 0:8,9. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Comma
            unexpected input found at 1:4,5. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Comma
            unexpected input found at 1:14,15. expected [Ident, String, Colon, OpenBracket, Digit(Decimal)] but found Comma
        "#]],
    );
}
