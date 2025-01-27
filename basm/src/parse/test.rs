use expect_test::{expect, Expect};
use string_interner::symbol::SymbolU32;

use crate::{Basm, Value};

use super::Parser;

fn vals(basm: &Basm, values: &[Value]) -> String {
    let mut out = String::new();
    for value in values {
        if !out.is_empty() {
            out.push_str(", ");
        } else {
            out.push(' ');
        }
        match value {
            Value::Deref(sy) => {
                out.push('[');
                out.push_str(basm.si.resolve(*sy).unwrap());
                out.push(']');
            }
            Value::Ident(sy) => {
                out.push_str(basm.si.resolve(*sy).unwrap());
            }
            Value::String(sy) => {
                out.push('"');
                out.push_str(basm.si.resolve(*sy).unwrap());
                out.push('"');
            }
            Value::Digit(_, n) => out.push_str(&n.to_string()),
        };
    }
    out
}

fn check(src: &str, expect: Expect) {
    use crate::Line::*;
    use std::fmt::Write;
    let (basm, errors) = Parser::base(src).parse();
    let mut output = std::string::String::with_capacity(src.len());
    output.push_str("output:\n");
    let sy = |sy: &SymbolU32| basm.si.resolve(*sy).unwrap();
    basm.lines
        .iter()
        .try_for_each(|line| match line {
            NoOp => writeln!(output, "NoOp: "),
            Global { name } => writeln!(output, "Global: {}", sy(name)),
            Label { name } => writeln!(output, "Label: {}", sy(name)),
            Instruction { ins, values } => {
                writeln!(output, "Instruction: {}{}", sy(ins), vals(&basm, values))
            }
            Variable {
                name,
                r#type,
                values,
            } => writeln!(
                output,
                "Variable: {} {}{}",
                sy(name),
                sy(r#type),
                vals(&basm, values)
            ),
        })
        // writeln!(output, "{line:?}")
        .unwrap();
    errors
        .iter()
        .try_for_each(|e| write!(output, "{e}"))
        .unwrap();
    expect.assert_eq(&output);
}

#[test]
fn empty() {
    check("", expect![[r#"
        output:
    "#]]);
}
#[test]
fn multi_empty() {
    check(
        "\n\n\n\n",
        expect![[r#"
            output:
            NoOp: 
            NoOp: 
            NoOp: 
            NoOp: 
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
            NoOp: 
            NoOp: 
            NoOp: 
            NoOp: 
        "#]],
    );
}
#[test]
fn singles() {
    check(
        "one\n two\t\n threeeeee\n four\n",
        expect![[r#"
            output:
            Instruction: one
            Instruction: two
            Instruction: threeeeee
            Instruction: four
        "#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            output:
            Instruction: one two
            Instruction: two threeeee
            Instruction: threeeeee four
            Instruction: four five
        "#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            output:
            Instruction: one two, three
            Instruction: two threeeee, four
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
            NoOp: 
            NoOp: 
            NoOp: 
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
            Instruction: abc
            Instruction: cde efg
            Instruction: ghi ijk, klm
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
            Label: l1
            Label: l2
            Label: l3
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
            NoOp: 
            unexpected input found at: 0:0:1. expected Ident | Eol | Eof but got Digit(Decimal)
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
            Variable: msg db "ONE TWO THREE", 12309, 12
            Variable: digit reb 100
        "#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            output:
            NoOp: 
            unexpected input found at: 0:23:28. expected Comma but got Digit(Decimal)
        "#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            output:
            NoOp: 
            NoOp: 
            Variable: message str "Hello, World", 10
            Global: _start
            NoOp: 
            Label: _start
            Instruction: mov rax, 1
            Instruction: mov rdi, 1
            Instruction: mov rsi, message
            Instruction: mov rdx, 13
            Instruction: syscall
            Instruction: mov rax, 60
            Instruction: xor rdi, rdi
            Instruction: syscall
        "#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            output:
            Variable: hello_world str "Hello, World!", 10, 0
            Variable: whats_up str "What's up", 10, 0
            Variable: long_text str "this is a longer line of text.", 10, 0
            NoOp: 
            Global: _start
            NoOp: 
            Label: _start
            Instruction: mov rax, hello_world
            Instruction: call print
            NoOp: 
            Instruction: mov rax, whats_up
            Instruction: call print
            NoOp: 
            Instruction: mov rax, long_text
            Instruction: call print
            NoOp: 
            Instruction: mov rax, 60
            Instruction: mov rdi, 0
            Instruction: syscall
            Label: print
            Instruction: push rax
            Instruction: mov rbx, 0
            Label: print_loop
            Instruction: inc rax
            Instruction: inc rbx
            Instruction: mov cl, [rax]
            Instruction: cmp cl, 0
            Instruction: jne print_loop
            NoOp: 
            Instruction: mov rax, 1
            Instruction: mov rdi, 1
            Instruction: pop rsi
            Instruction: mov rdx, rbx
            Instruction: syscall
            NoOp: 
            Instruction: ret
        "#]],
    );
}

#[test]
fn print_int() {
    check(
        include_str!("../../../test-sample/4-print-int.asm"),
        expect![[r#"
            output:
            Variable: digit_buffer resb 100
            Variable: digit_buffer_pos resb 8
            NoOp: 
            NoOp: 
            NoOp: 
            Global: _start
            NoOp: 
            Label: _start
            Instruction: mov rax, 1337
            Instruction: call print
            NoOp: 
            Instruction: mov rax, 60
            Instruction: mov rdi, 0
            Instruction: syscall
            NoOp: 
            Label: print
            Instruction: mov rcx, digit_buffer
            Instruction: mov rbx, 10
            Instruction: mov [rcx], rbx
            Instruction: inc rcx
            Instruction: mov [digit_buffer_pos], rcx
            Label: int_buffer
            Instruction: mov rdx, 0
            Instruction: mov rbx, 10
            Instruction: div rbx
            Instruction: push rax
            Instruction: add rdx, 48
            NoOp: 
            Instruction: mov rcx, [digit_buffer_pos]
            Instruction: mov [rcx], dl
            Instruction: inc rcx
            Instruction: mov [digit_buffer_pos], rcx
            NoOp: 
            Instruction: pop rax
            Instruction: cmp rax, 0
            Instruction: jne int_buffer
            Label: print_loop
            Instruction: mov rcx, [digit_buffer_pos]
            NoOp: 
            Instruction: mov rax, 1
            Instruction: mov rdi, 1
            Instruction: mov rsi, rcx
            Instruction: mov rdx, 1
            Instruction: syscall
            NoOp: 
            Instruction: mov rcx, [digit_buffer_pos]
            Instruction: dec rcx
            Instruction: mov [digit_buffer_pos], rcx
            NoOp: 
            Instruction: cmp rcx, digit_buffer
            Instruction: jge print_loop
            NoOp: 
            Instruction: ret
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
            NoOp: 
            NoOp: 
            unexpected input found at: 0:0:6. expected Ident | Eol | Eof but got Digit(Decimal)
            unexpected input found at: 1:0:3. expected Ident | Eol | Eof but got Digit(Hex)
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
            NoOp: 
            NoOp: 
            NoOp: 
            input ended early at: 0:14:15
            unexpected input found at: 1:13:18. expected CloseBracket but got Digit(Decimal)
            unexpected input found at: 2:9:14. expected Ident but got Digit(Decimal)
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
            NoOp: 
            unexpected input found at: 0:0:1. expected Ident | Eol | Eof but got Digit(Decimal)
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
            NoOp: 
            NoOp: 
            unexpected input found at: 0:0:3. expected Ident | Eol | Eof but got Digit(Decimal)
            unexpected input found at: 1:0:3. expected Ident | Eol | Eof but got Digit(Decimal)
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
            NoOp: 
            NoOp: 
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
: : : ;empty!
",
        expect![[r#"
            output:
            NoOp: 
            NoOp: 
            NoOp: 
            NoOp: 
            unexpected input found at: 0:7:8. expected Whitespace but got Colon
            unexpected input found at: 1:11:12. expected Comma but got Colon
            unexpected input found at: 2:20:21. expected Comma but got Colon
            unexpected input found at: 3:0:1. expected Ident | Eol | Eof but got Colon
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
 `~]]./\\;empty!
",
        expect![[r#"
            output:
            NoOp: 
            NoOp: 
            NoOp: 
            NoOp: 
            unexpected input found at: 0:7:9. expected Whitespace but got Other
            unexpected input found at: 1:12:14. expected Comma but got Other
            unexpected input found at: 2:20:22. expected Comma but got Other
            unexpected input found at: 3:1:3. expected Ident | Eol | Eof but got Other
        "#]],
    );
}

#[test]
fn empty_not_instruction() {
    check(
        "one, two, three\n two, threeeee, four",
        expect![[r#"
            output:
            NoOp: 
            NoOp: 
            unexpected input found at: 0:3:4. expected Ident | Str | Colon | OpenBracket | Digit but got Comma
            unexpected input found at: 1:4:5. expected Ident | Str | Colon | OpenBracket | Digit but got Comma
        "#]],
    );
}
