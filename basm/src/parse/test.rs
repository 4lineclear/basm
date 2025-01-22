use expect_test::{expect, Expect};

use super::Parser;

fn check(src: &str, expect: Expect) {
    let (basm, errors) = Parser::new(src).parse();
    let mut labels = basm.labels.into_iter().collect::<Vec<_>>();
    labels.sort();
    let output = format!(
        "{}{}{}{}",
        basm.sections
            .into_iter()
            .map(|s| format!("{s:?}\n"))
            .collect::<String>(),
        labels
            .iter()
            .map(|(s, a)| format!("{s}: {a}\n"))
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
        expect![""],
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
        "#]],
    );
}
#[test]
fn empty_ws() {
    check(
        "\t\t    \t     \t ",
        expect![""],
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
        "#]],
    );
}
#[test]
fn singles() {
    check(
        "one\n two\t\n threeeeee\n four\n",
        expect![[r#"
            Instruction { ins: (0, 3), values: [] }
            Instruction { ins: (5, 8), values: [] }
            Instruction { ins: (11, 20), values: [] }
            Instruction { ins: (22, 26), values: [] }
        "#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            Instruction { ins: (0, 3), values: [] }
            Instruction { ins: (9, 12), values: [] }
            Instruction { ins: (24, 33), values: [] }
            Instruction { ins: (40, 44), values: [] }
        "#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            Instruction { ins: (0, 3), values: [Ident((4, 7)), Ident((9, 14))] }
            Instruction { ins: (16, 19), values: [Ident((20, 28)), Ident((30, 34))] }
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
            Instruction { ins: (0, 3), values: [] }
            Instruction { ins: (28, 31), values: [] }
            Instruction { ins: (53, 56), values: [Ident((57, 60)), Ident((62, 65))] }
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
            l1: 0
            l2: 0
            l3: 0
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
            Section { kind: Bss, address: 0 }
            Section { kind: Text, address: 0 }
            unexpected input found at 0:13,16. expected Whitespace but found Ident
        "#]],
    );
}
#[test]
fn decimal() {
    check(
        "\
1, 1293, 9384093, 1231234",
        expect![[r#"
            unexpected input found at 0:0,1. expected Ident | Eol | Eof but found Digit(Decimal)
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
            Variable { name: (0, 3), type: (4, 6), values: [String("ONE TWO THREE"), Digit(Decimal, 12309), Digit(Decimal, 12)] }
            Variable { name: (34, 39), type: (40, 43), values: [Digit(Decimal, 100)] }
        "#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            unexpected input found at 0:23,28. expected Comma | Ident | Str but found Digit(Decimal)
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
            _start: 4
            Variable { name: (19, 26), type: (27, 29), values: [String("Hello, World"), Digit(Decimal, 10)] }
            NoOp
            Instruction { ins: (100, 106), values: [] }
            NoOp
            Instruction { ins: (131, 134), values: [Ident((141, 144)), Digit(Decimal, 1)] }
            Instruction { ins: (187, 190), values: [Ident((197, 200)), Digit(Decimal, 1)] }
            Instruction { ins: (245, 248), values: [Ident((255, 258)), Ident((260, 267))] }
            Instruction { ins: (307, 310), values: [Ident((317, 320)), Digit(Decimal, 13)] }
            Instruction { ins: (357, 364), values: [] }
            Instruction { ins: (431, 434), values: [Ident((441, 444)), Digit(Decimal, 60)] }
            Instruction { ins: (486, 489), values: [Ident((496, 499)), Ident((501, 504))] }
            Instruction { ins: (532, 539), values: [] }
        "#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            Section { kind: Data, address: 0 }
            Section { kind: Text, address: 4 }
            _start: 6
            print: 18
            print_loop: 20
            Variable { name: (17, 28), type: (29, 31), values: [String("Hello, World!"), Digit(Decimal, 10), Digit(Decimal, 0)] }
            Variable { name: (59, 67), type: (68, 70), values: [String("What's up"), Digit(Decimal, 10), Digit(Decimal, 0)] }
            Variable { name: (94, 103), type: (104, 106), values: [String("this is a longer line of text."), Digit(Decimal, 10), Digit(Decimal, 0)] }
            NoOp
            Instruction { ins: (165, 171), values: [] }
            NoOp
            Instruction { ins: (192, 195), values: [Ident((196, 199)), Ident((201, 212))] }
            Instruction { ins: (217, 221), values: [] }
            NoOp
            Instruction { ins: (233, 236), values: [Ident((237, 240)), Ident((242, 250))] }
            Instruction { ins: (255, 259), values: [] }
            NoOp
            Instruction { ins: (271, 274), values: [Ident((275, 278)), Ident((280, 289))] }
            Instruction { ins: (294, 298), values: [] }
            NoOp
            Instruction { ins: (310, 313), values: [Ident((314, 317)), Digit(Decimal, 60)] }
            Instruction { ins: (326, 329), values: [Ident((330, 333)), Digit(Decimal, 0)] }
            Instruction { ins: (341, 348), values: [] }
            Instruction { ins: (360, 364), values: [] }
            Instruction { ins: (373, 376), values: [Ident((377, 380)), Digit(Decimal, 0)] }
            Instruction { ins: (400, 403), values: [] }
            Instruction { ins: (412, 415), values: [] }
            Instruction { ins: (424, 427), values: [Ident((428, 430)), Deref((433, 436))] }
            Instruction { ins: (442, 445), values: [Ident((446, 448)), Digit(Decimal, 0)] }
            Instruction { ins: (456, 459), values: [] }
            NoOp
            Instruction { ins: (476, 479), values: [Ident((480, 483)), Digit(Decimal, 1)] }
            Instruction { ins: (491, 494), values: [Ident((495, 498)), Digit(Decimal, 1)] }
            Instruction { ins: (506, 509), values: [] }
            Instruction { ins: (518, 521), values: [Ident((522, 525)), Ident((527, 530))] }
            Instruction { ins: (535, 542), values: [] }
            NoOp
            Instruction { ins: (548, 551), values: [] }
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
            unexpected input found at 0:0,6. expected Ident | Eol | Eof but found Digit(Decimal)
            unexpected input found at 1:0,3. expected Ident | Eol | Eof but found Digit(Hex)
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
            unexpected input found at 0:4,5. expected Ident | Colon but found OpenBracket
            unexpected input found at 1:4,5. expected Ident | Colon but found OpenBracket
            unexpected input found at 2:6,7. expected Ident | Colon but found OpenBracket
        "#]],
    );
}

#[test]
fn float() {
    check(
        "\
0.123.123.123
",
        expect![[r#"
            unexpected input found at 0:0,1. expected Ident | Eol | Eof but found Digit(Decimal)
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
            unexpected input found at 0:0,3. expected Ident | Eol | Eof but found Digit(Decimal)
            unexpected input found at 1:0,3. expected Ident | Eol | Eof but found Digit(Decimal)
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
            unexpected input found at 0:7,8. expected Whitespace but found Colon
            unexpected input found at 1:11,12. expected Comma | Ident | Str but found Colon
            unexpected input found at 2:20,21. expected Comma | Ident | Str but found Colon
            unexpected input found at 3:8,15. expected 'bss' | 'data' | 'text' but found Ident
            unexpected input found at 4:0,1. expected Ident | Eol | Eof but found Colon
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
            unexpected input found at 0:7,9. expected Whitespace but found Other
            unexpected input found at 1:12,14. expected Comma | Ident | Str but found Other
            unexpected input found at 2:20,22. expected Comma | Ident | Str but found Other
            unexpected input found at 3:8,15. expected 'bss' | 'data' | 'text' but found Ident
            unexpected input found at 4:1,3. expected Ident | Eol | Eof but found Other
        "#]],
    );
}

#[test]
fn empty_not_instruction() {
    check(
        "one, two, three\n two, threeeee, four",
        expect![[r#"
            unexpected input found at 0:3,4. expected Ident | Colon but found Comma
            unexpected input found at 1:4,5. expected Ident | Colon but found Comma
        "#]],
    );
}
