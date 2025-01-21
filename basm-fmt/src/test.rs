// use std::fmt::Debug;
//
// use basm::lex::LexOutput;
// use expect_test::{expect, Expect};
//
// use crate::Edit;
// fn display_edit(src: &str, edit: Edit) -> String {
//     format!(
//         "{}:{:?} = '{}' -> '{}'",
//         edit.line,
//         edit.span,
//         edit.span.slice(src).escape_debug(),
//         edit.change
//     )
// }
//
// fn check(src: &str, expect: Expect) {
//     let lo = &LexOutput::lex_all(src);
//     expect.assert_eq(
//         &crate::fmt(lo, &Default::default())
//             .map(|e| display_edit(lo.line_src(e.line as usize), e))
//             .collect::<Vec<_>>()
//             .join("\n"),
//     );
// }
//
// #[allow(unused)]
// fn debug_check(actual: &impl Debug, expect: Expect) {
//     expect.assert_debug_eq(actual);
// }
//
// #[test]
// fn empty() {
//     check("", expect![[""]]);
// }
//
// #[test]
// fn empty_lines() {
//     check("\n\n\n", expect![[""]]);
// }
//
// #[test]
// fn ws() {
//     check(
//         " \t    \t",
//         expect![[r#"
//         0:(0, 7) = ' \t    \t' -> ''"#]],
//     );
// }
//
// #[test]
// fn empty_lines_ws() {
//     check(
//         "  \t  \t \n \t \n \n",
//         expect![[r#"
//             0:(0, 7) = '  \t  \t ' -> ''
//             1:(0, 3) = ' \t ' -> ''
//             2:(0, 1) = ' ' -> ''"#]],
//     );
// }
//
// #[test]
// fn empty_lit_ws_start() {
//     check(
//         "
// 123, 123
//     123, 123
// \t0b123, 123
//   \t  0x123 123
// ",
//         expect![[r#"
//             2:(0, 4) = '    ' -> ''
//             3:(0, 1) = '\t' -> ''
//             4:(0, 5) = '  \t  ' -> ''"#]],
//     );
// }
//
// #[test]
// fn empty_lit_ws_end() {
//     check(
//         "
// 123, 123
// 0b123, 123  \t\t\t\t
// 0x123 123               \t
// ",
//         expect![[r#"
//             1:(8, 20) = '            ' -> ''
//             2:(10, 16) = '  \t\t\t\t' -> ''
//             3:(9, 25) = '               \t' -> ''"#]],
//     );
// }
//
// #[test]
// fn empty_lit_ws_both() {
//     check(
//         "
// 123, 123
//         0b123, 123  \t\t\t\t
// \t\t\t\t0x123 123               \t
// ",
//         expect![[r#"
//             1:(8, 20) = '            ' -> ''
//             2:(0, 8) = '        ' -> ''
//             2:(18, 24) = '  \t\t\t\t' -> ''
//             3:(0, 4) = '\t\t\t\t' -> ''
//             3:(13, 29) = '               \t' -> ''"#]],
//     );
// }
//
// #[test]
// fn pad_instruction_four() {
//     check(
//         "
// mov rax, 12
// mov rax, rax
// mov rax
// ",
//         expect![[r#"
//             1:(0, 0) = '' -> '    '
//             2:(0, 0) = '' -> '    '
//             3:(0, 0) = '' -> '    '"#]],
//     );
// }
//
// #[test]
// fn pad_instruction_n() {
//     check(
//         "
//             mov rax, 12
//     mov rax, rax
//             mov rax
// ",
//         expect![[r#"
//             1:(0, 12) = '            ' -> '    '
//             3:(0, 12) = '            ' -> '    '"#]],
//     );
// }
//
// #[test]
// fn pad_trim() {
//     check(
//         "
//             \tmov rax, 12
//     mov rax, rax
//    \t\t         mov rax
// ",
//         expect![[r#"
//             1:(0, 13) = '            \t' -> '    '
//             1:(24, 29) = '     ' -> ''
//             2:(16, 32) = '                ' -> ''
//             3:(0, 14) = '   \t\t         ' -> '    '
//             3:(21, 30) = '         ' -> ''"#]],
//     );
// }
//
// #[test]
// fn deref() {
//     check(
//         "
//     rax [deref]
//     rax [deref      ]
//     rax [           deref]
//     rax [\t \t deref  \t\t   ]
// ",
//         expect![[r#"
//             2:(14, 20) = '      ' -> ''
//             3:(9, 20) = '           ' -> ''
//             4:(9, 13) = '\t \t ' -> ''
//             4:(18, 25) = '  \t\t   ' -> ''"#]],
//     );
// }
//
// #[test]
// fn comment_empty() {
//     check(
//         "
// ;
//     ;   \t\t
//     \t\t;   \t\t
//             ;;;;    \t
// ",
//         expect![[r#"
//             2:(0, 4) = '    ' -> ''
//             2:(5, 10) = '   \t\t' -> ''
//             3:(0, 6) = '    \t\t' -> ''
//             3:(7, 12) = '   \t\t' -> ''
//             4:(0, 12) = '            ' -> ''
//             4:(13, 13) = '' -> ' '
//             4:(16, 21) = '    \t' -> ''"#]],
//     );
// }
//
// #[test]
// fn ins_comment() {
//     check(
//         "
//     mov rax, 12 ; do nothing
//     mov rax, 12     ;\t do something
//     mov rax, 12 ;\t   do something
//     mov rax, 12     ;\t   do something
//     mov rax, 12;\t   do something
// ; do nothing
// ",
//         expect![[r#"
//             2:(15, 20) = '     ' -> ' '
//             2:(21, 23) = '\t ' -> ' '
//             3:(17, 21) = '\t   ' -> ' '
//             4:(15, 20) = '     ' -> ' '
//             4:(21, 25) = '\t   ' -> ' '
//             5:(16, 20) = '\t   ' -> ' '"#]],
//     );
// }
//
// #[test]
// fn hello_world() {
//     check(
//         include_str!("../../test-sample/0-hello.asm"),
//         expect![[r#"
//             0:(7, 10) = '   ' -> ' '
//             1:(33, 35) = '  ' -> ' '
//             3:(7, 10) = '   ' -> ' '
//             4:(10, 14) = '    ' -> ' '
//             6:(7, 8) = ' ' -> ''
//             7:(7, 14) = '       ' -> ' '
//             7:(20, 32) = '            ' -> ' '
//             8:(7, 14) = '       ' -> ' '
//             8:(20, 32) = '            ' -> ' '
//             9:(7, 14) = '       ' -> ' '
//             9:(26, 32) = '      ' -> ' '
//             10:(7, 14) = '       ' -> ' '
//             10:(21, 32) = '           ' -> ' '
//             11:(11, 32) = '                     ' -> ' '
//             12:(7, 14) = '       ' -> ' '
//             12:(21, 32) = '           ' -> ' '
//             13:(7, 14) = '       ' -> ' '
//             13:(22, 32) = '          ' -> ' '
//             14:(11, 32) = '                     ' -> ' '"#]],
//     );
// }
