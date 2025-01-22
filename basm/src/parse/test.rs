use expect_test::{expect, Expect};

use super::Parser;

fn check(src: &str, expect: Expect) {
    use std::fmt::Write;
    let (basm, errors) = Parser::base(src).parse();
    let mut output = std::string::String::with_capacity(src.len());
    let mut section = 0;
    let mut label = 0;
    let mut labels = basm
        .labels
        .into_iter()
        .map(|(k, ad)| (basm.si.resolve(k).unwrap(), ad))
        .collect::<Vec<_>>();
    labels.sort_by_key(|&(s, ad)| (ad, s));
    output.push_str("output:\n");
    (0..basm.lines.len())
        .try_for_each(|i| {
            if section < basm.sections.len() && basm.sections[section].address as usize == i {
                writeln!(output, "{:?}", basm.sections[section])?;
                section += 1
            }
            if label < labels.len() && labels[label].1 as usize == i {
                writeln!(output, "{}: {}", labels[label].0, labels[label].1)?;
                label += 1
            }
            writeln!(output, "{:?}", basm.lines[i])
        })
        .unwrap();
    while (section as usize) < basm.sections.len() {
        writeln!(output, "{:?}", basm.sections[section]).unwrap();
        section += 1
    }
    while (label as usize) < labels.len() {
        writeln!(output, "{:?}", labels[label]).unwrap();
        label += 1
    }
    errors
        .iter()
        .try_for_each(|e| write!(output, "{e}"))
        .unwrap();
    expect.assert_eq(&output);
}

#[test]
fn empty() {
    check(
        "",
        expect![[r#"
        output:
    "#]],
    );
}
#[test]
fn multi_empty() {
    check(
        "\n\n\n\n",
        expect![[r#"
            output:
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
        output:
    "#]],
    );
}
#[test]
fn multi_empty_ws() {
    check(
        "\t\t    \t     \t \n\n\t\t    \t     \t \n\n",
        expect![[r#"
            output:
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
            output:
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
            output:
            Instruction { ins: (0, 3), values: [Ident((4, 7))] }
            Instruction { ins: (9, 12), values: [Ident((13, 21))] }
            Instruction { ins: (24, 33), values: [Ident((34, 38))] }
            Instruction { ins: (40, 44), values: [Ident((45, 49))] }
        "#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            output:
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
            output:
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
            output:
            Instruction { ins: (0, 3), values: [] }
            Instruction { ins: (28, 31), values: [Ident((32, 35))] }
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
            output:
            ("l1", 0)
            ("l2", 0)
            ("l3", 0)
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
            output:
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
            output:
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
            output:
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
            output:
            unexpected input found at 0:23,28. expected Comma but found Digit(Decimal)
        "#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            output:
            Section { kind: Data, address: 0 }
            Variable { name: (19, 26), type: (27, 29), values: [String("Hello, World"), Digit(Decimal, 10)] }
            NoOp
            Section { kind: Text, address: 2 }
            Instruction { ins: (100, 106), values: [Ident((110, 116))] }
            NoOp
            _start: 4
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
            output:
            Section { kind: Data, address: 0 }
            Variable { name: (17, 28), type: (29, 31), values: [String("Hello, World!"), Digit(Decimal, 10), Digit(Decimal, 0)] }
            Variable { name: (59, 67), type: (68, 70), values: [String("What's up"), Digit(Decimal, 10), Digit(Decimal, 0)] }
            Variable { name: (94, 103), type: (104, 106), values: [String("this is a longer line of text."), Digit(Decimal, 10), Digit(Decimal, 0)] }
            NoOp
            Section { kind: Text, address: 4 }
            Instruction { ins: (165, 171), values: [Ident((172, 178))] }
            NoOp
            _start: 6
            Instruction { ins: (192, 195), values: [Ident((196, 199)), Ident((201, 212))] }
            Instruction { ins: (217, 221), values: [Ident((222, 227))] }
            NoOp
            Instruction { ins: (233, 236), values: [Ident((237, 240)), Ident((242, 250))] }
            Instruction { ins: (255, 259), values: [Ident((260, 265))] }
            NoOp
            Instruction { ins: (271, 274), values: [Ident((275, 278)), Ident((280, 289))] }
            Instruction { ins: (294, 298), values: [Ident((299, 304))] }
            NoOp
            Instruction { ins: (310, 313), values: [Ident((314, 317)), Digit(Decimal, 60)] }
            Instruction { ins: (326, 329), values: [Ident((330, 333)), Digit(Decimal, 0)] }
            Instruction { ins: (341, 348), values: [] }
            print: 18
            Instruction { ins: (360, 364), values: [Ident((365, 368))] }
            Instruction { ins: (373, 376), values: [Ident((377, 380)), Digit(Decimal, 0)] }
            print_loop: 20
            Instruction { ins: (400, 403), values: [Ident((404, 407))] }
            Instruction { ins: (412, 415), values: [Ident((416, 419))] }
            Instruction { ins: (424, 427), values: [Ident((428, 430)), Deref((433, 436))] }
            Instruction { ins: (442, 445), values: [Ident((446, 448)), Digit(Decimal, 0)] }
            Instruction { ins: (456, 459), values: [Ident((460, 470))] }
            NoOp
            Instruction { ins: (476, 479), values: [Ident((480, 483)), Digit(Decimal, 1)] }
            Instruction { ins: (491, 494), values: [Ident((495, 498)), Digit(Decimal, 1)] }
            Instruction { ins: (506, 509), values: [Ident((510, 513))] }
            Instruction { ins: (518, 521), values: [Ident((522, 525)), Ident((527, 530))] }
            Instruction { ins: (535, 542), values: [] }
            NoOp
            Instruction { ins: (548, 551), values: [] }
        "#]],
    );
}

#[test]
fn print_int() {
    check(
        include_str!("../../../test-sample/4-print-int.asm"),
        expect![[r#"
            output:
            Section { kind: Bss, address: 0 }
            Variable { name: (16, 28), type: (36, 40), values: [Digit(Decimal, 100)] }
            Variable { name: (75, 91), type: (95, 99), values: [Digit(Decimal, 8)] }
            NoOp
            NoOp
            Section { kind: Data, address: 4 }
            NoOp
            Section { kind: Text, address: 5 }
            Instruction { ins: (238, 244), values: [Ident((245, 251))] }
            NoOp
            _start: 7
            Instruction { ins: (265, 268), values: [Ident((269, 272)), Digit(Decimal, 1337)] }
            Instruction { ins: (283, 287), values: [Ident((288, 293))] }
            NoOp
            Instruction { ins: (299, 302), values: [Ident((303, 306)), Digit(Decimal, 60)] }
            Instruction { ins: (315, 318), values: [Ident((319, 322)), Digit(Decimal, 0)] }
            Instruction { ins: (330, 337), values: [] }
            NoOp
            print: 14
            Instruction { ins: (350, 353), values: [Ident((354, 357)), Ident((359, 371))] }
            Instruction { ins: (376, 379), values: [Ident((380, 383)), Digit(Decimal, 10)] }
            Instruction { ins: (392, 395), values: [Deref((397, 400)), Ident((403, 406))] }
            Instruction { ins: (411, 414), values: [Ident((415, 418))] }
            Instruction { ins: (423, 426), values: [Deref((428, 444)), Ident((447, 450))] }
            int_buffer: 19
            Instruction { ins: (467, 470), values: [Ident((471, 474)), Digit(Decimal, 0)] }
            Instruction { ins: (482, 485), values: [Ident((486, 489)), Digit(Decimal, 10)] }
            Instruction { ins: (498, 501), values: [Ident((502, 505))] }
            Instruction { ins: (510, 514), values: [Ident((515, 518))] }
            Instruction { ins: (523, 526), values: [Ident((527, 530)), Digit(Decimal, 48)] }
            NoOp
            Instruction { ins: (584, 587), values: [Ident((588, 591)), Deref((594, 610))] }
            Instruction { ins: (616, 619), values: [Deref((621, 624)), Ident((627, 629))] }
            Instruction { ins: (634, 637), values: [Ident((638, 641))] }
            Instruction { ins: (646, 649), values: [Deref((651, 667)), Ident((670, 673))] }
            NoOp
            Instruction { ins: (679, 682), values: [Ident((683, 686))] }
            Instruction { ins: (691, 694), values: [Ident((695, 698)), Digit(Decimal, 0)] }
            Instruction { ins: (706, 709), values: [Ident((710, 720))] }
            print_loop: 33
            Instruction { ins: (737, 740), values: [Ident((741, 744)), Deref((747, 763))] }
            NoOp
            Instruction { ins: (770, 773), values: [Ident((774, 777)), Digit(Decimal, 1)] }
            Instruction { ins: (785, 788), values: [Ident((789, 792)), Digit(Decimal, 1)] }
            Instruction { ins: (800, 803), values: [Ident((804, 807)), Ident((809, 812))] }
            Instruction { ins: (817, 820), values: [Ident((821, 824)), Digit(Decimal, 1)] }
            Instruction { ins: (832, 839), values: [] }
            NoOp
            Instruction { ins: (845, 848), values: [Ident((849, 852)), Deref((855, 871))] }
            Instruction { ins: (877, 880), values: [Ident((881, 884))] }
            Instruction { ins: (889, 892), values: [Deref((894, 910)), Ident((913, 916))] }
            NoOp
            Instruction { ins: (922, 925), values: [Ident((926, 929)), Ident((931, 943))] }
            Instruction { ins: (948, 951), values: [Ident((952, 962))] }
            NoOp
            Instruction { ins: (968, 971), values: [] }
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
            output:
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
            output:
            unexpected input end on line 0
            unexpected input found at 1:13,18. expected CloseBracket but found Digit(Decimal)
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
            output:
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
            output:
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
            output:
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
            output:
            unexpected input found at 0:7,8. expected Whitespace but found Colon
            unexpected input found at 1:11,12. expected Comma but found Colon
            unexpected input found at 2:20,21. expected Comma but found Colon
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
            output:
            unexpected input found at 0:7,9. expected Whitespace but found Other
            unexpected input found at 1:12,14. expected Comma but found Other
            unexpected input found at 2:20,22. expected Comma but found Other
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
            output:
            unexpected input found at 0:3,4. expected Ident | Str | Colon | OpenBracket | Digit but found Comma
            unexpected input found at 1:4,5. expected Ident | Str | Colon | OpenBracket | Digit but found Comma
        "#]],
    );
}
