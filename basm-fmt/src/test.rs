use std::fmt::Debug;

use basm::parse::Parser;
use expect_test::{expect, Expect};

use crate::Edit;
fn display_edit(src: &str, edit: Edit) -> String {
    let mut span = edit.span;
    span.from += edit.offset;
    span.to += edit.offset;
    format!(
        "{}:{:?} = '{}' -> '{}'",
        edit.line,
        edit.span,
        span.slice(src).escape_debug(),
        edit.text
    )
}

fn check(src: &str, expect: Expect) {
    let (basm, errors, lex) = Parser::recorded(&src).parse();
    expect.assert_eq(
        &crate::fmt(&basm, &lex, src, &errors, &Default::default())
            .into_iter()
            .map(|e| display_edit(src, e))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[allow(unused)]
fn debug_check(actual: &impl Debug, expect: Expect) {
    expect.assert_debug_eq(actual);
}

#[test]
fn empty() {
    check("", expect![[""]]);
}

#[test]
fn empty_lines() {
    check("\n\n\n", expect![[""]]);
}

#[test]
fn empty_lines_ws() {
    check(
        "  \t  \t \n \t \n \n",
        expect![[r#"
            0:(0, 7) = '  \t  \t ' -> ''
            1:(0, 3) = ' \t ' -> ''
            2:(0, 1) = ' ' -> ''"#]],
    );
}

#[test]
fn pad_instruction_four() {
    check(
        "
mov rax, 12
mov rax, rax
mov rax
",
        expect![[r#"
            1:(0, 0) = '' -> '    '
            2:(0, 0) = '' -> '    '
            3:(0, 0) = '' -> '    '"#]],
    );
}

#[test]
fn pad_instruction_n() {
    check(
        "
            mov rax, 12
    mov rax, rax
            mov rax
",
        expect![[r#"
            1:(0, 12) = '            ' -> '    '
            3:(0, 12) = '            ' -> '    '"#]],
    );
}

#[test]
fn pad_trim() {
    check(
        "
            \tmov rax, 12     
    mov rax, rax                
   \t\t         mov rax         
",
        expect![[r#"
            1:(0, 13) = '            \t' -> '    '
            1:(24, 29) = '     ' -> ''
            2:(16, 32) = '                ' -> ''
            3:(0, 14) = '   \t\t         ' -> '    '
            3:(21, 30) = '         ' -> ''"#]],
    );
}

#[test]
fn deref() {
    check(
        "
    rax [deref]
    rax [deref      ]
    rax [           deref]
    rax [\t \t deref  \t\t   ]
",
        expect![[r#"
            2:(14, 20) = '      ' -> ''
            3:(9, 20) = '           ' -> ''
            4:(9, 13) = '\t \t ' -> ''
            4:(18, 25) = '  \t\t   ' -> ''"#]],
    );
}

#[test]
fn comment_empty() {
    check(
        "
;
    ;   \t\t
    \t\t;   \t\t
            ;;;;    \t
",
        expect![[r#"
            2:(0, 4) = '    ' -> ''
            2:(5, 10) = '   \t\t' -> ''
            3:(0, 6) = '    \t\t' -> ''
            3:(7, 12) = '   \t\t' -> ''
            4:(0, 12) = '            ' -> ''
            4:(13, 13) = '' -> ' '
            4:(16, 21) = '    \t' -> ''"#]],
    );
}

#[test]
fn ins_comment() {
    check(
        "
    mov rax, 12 ; do nothing
    mov rax, 12     ;\t do something
    mov rax, 12 ;\t   do something
    mov rax, 12     ;\t   do something
    mov rax, 12;\t   do something
; do nothing
",
        expect![[r#"
            2:(15, 20) = '     ' -> ' '
            2:(21, 23) = '\t ' -> ' '
            3:(17, 21) = '\t   ' -> ' '
            4:(15, 20) = '     ' -> ' '
            4:(21, 25) = '\t   ' -> ' '
            5:(15, 15) = '' -> ' '
            5:(16, 20) = '\t   ' -> ' '"#]],
    );
}

#[test]
fn hello_world() {
    check(
        include_str!("../../test-sample/0-hello.asm"),
        expect![[r#"
            1:(7, 10) = '   ' -> ' '
            2:(33, 35) = '  ' -> ' '
            4:(7, 10) = '   ' -> ' '
            5:(10, 14) = '    ' -> ' '
            7:(7, 8) = ' ' -> ''
            8:(7, 14) = '       ' -> ' '
            8:(20, 32) = '            ' -> ' '
            9:(7, 14) = '       ' -> ' '
            9:(20, 32) = '            ' -> ' '
            10:(7, 14) = '       ' -> ' '
            10:(26, 32) = '      ' -> ' '
            11:(7, 14) = '       ' -> ' '
            11:(21, 32) = '           ' -> ' '
            12:(11, 32) = '                     ' -> ' '
            13:(7, 14) = '       ' -> ' '
            13:(21, 32) = '           ' -> ' '
            14:(7, 14) = '       ' -> ' '
            14:(22, 32) = '          ' -> ' '
            15:(11, 32) = '                     ' -> ' '"#]],
    );
}

#[test]
fn apply_works() {
    assert_eq!(
        crate::apply_fmt(include_str!("../../test-sample/0-hello.asm")),
        include_str!("../../test-sample/0-hello.basm"),
    );
}
