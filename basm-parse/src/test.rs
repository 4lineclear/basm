use expect_test::{expect, Expect};

use crate::Span;

// TODO: create human readable version of SP

trait Stringify {
    fn stringify(&self) -> String;
}

impl<T: Stringify, E: std::fmt::Debug> Stringify for Result<T, E> {
    fn stringify(&self) -> String {
        match self {
            Ok(t) => t.stringify(),
            Err(e) => format!("{e:?}"),
        }
    }
}

impl<T0: Stringify, T1: Stringify> Stringify for (T0, T1) {
    fn stringify(&self) -> String {
        "'".to_owned() + &self.0.stringify() + "', " + &self.1.stringify()
    }
}
impl Stringify for Span<'_> {
    fn stringify(&self) -> String {
        format!("{}", self.fragment())
    }
}
impl Stringify for crate::Line<'_> {
    fn stringify(&self) -> String {
        self.kind.stringify()
            + &self
                .comment
                .map(|s| " ".to_owned() + &s.stringify())
                .unwrap_or("".to_owned())
    }
}
impl Stringify for crate::LineKind<'_> {
    fn stringify(&self) -> String {
        use crate::LineKind::*;
        match self {
            Empty => "empty".into(),
            Section(s) | Label(s) => s.stringify(),
            Ins(s, val) => s.stringify() + " " + &val.stringify(),
            Var { name, dir, val } => {
                name.stringify() + " " + &dir.stringify() + " " + &val.stringify()
            }
        }
    }
}
impl Stringify for crate::Value<'_> {
    fn stringify(&self) -> String {
        use crate::Value::*;
        match self {
            Hex(s) | Octal(s) | Binary(s) | Decimal(s) | Float(s) | Identifier(s) | Deref(s) => {
                s.stringify()
            }
            String(s) => format!("\"{s}\""),
        }
    }
}
impl Stringify for Vec<crate::Value<'_>> {
    fn stringify(&self) -> String {
        self.into_iter()
            .map(Stringify::stringify)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn debug_check<D: Stringify>(src: D, expect: Expect) {
    check(&src.stringify(), expect);
}
fn check(src: &str, expect: Expect) {
    expect.assert_eq(src);
}

fn apply_lines<'a, D: Stringify + 'a>(src: &'a str, apply: impl Fn(Span<'a>) -> D) -> String {
    src.lines()
        .map(Span::new)
        .map(apply)
        .map(|s| s.stringify())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn label() {
    debug_check(crate::label("   _one : ".into()), expect!["'', _one"]);
}
#[test]
fn label_fail() {
    debug_check(
        crate::label("12one:".into()),
        expect![[
            r#"Error(Error { input: LocatedSpan { offset: 0, line: 1, fragment: "12one:", extra: () }, code: Tag })"#
        ]],
    );
}
#[test]
fn nullary() {
    debug_check(crate::ins_or_var("   _one   ".into()), expect!["'', _one "]);
}
#[test]
fn unary() {
    debug_check(
        crate::ins_or_var("   _one  two ".into()),
        expect!["'', _one two"],
    );
}
#[test]
fn binary() {
    debug_check(
        crate::ins_or_var("   _one  two,   threee ".into()),
        expect!["'', _one two, threee"],
    );
}
#[test]
fn nullary_list() {
    let actual = apply_lines("one\ntwo\nthree\nfour", crate::ins_or_var);
    check(
        &actual,
        expect![[r#"
            '', one 
            '', two 
            '', three 
            '', four "#]],
    );
}
#[test]
fn unary_list() {
    let actual = apply_lines(
        "one two\nthree four\nfive six\nseven eight",
        crate::ins_or_var,
    );
    check(
        &actual,
        expect![[r#"
            '', one two
            '', three four
            '', five six
            '', seven eight"#]],
    );
}
#[test]
fn unit_list_1() {
    let actual = apply_lines(
        "one\ntwo three\nfour five\nsix seven, eight",
        crate::ins_or_var,
    );

    check(
        &actual,
        expect![[r#"
            '', one 
            '', two three
            '', four five
            '', six seven, eight"#]],
    );
}
#[test]
fn section() {
    debug_check(
        crate::section("   section .data".into()),
        expect!["'', data"],
    );
}
#[test]
fn sections() {
    let actual = apply_lines("section .data\nsection .bss\nsection .text", crate::section);

    check(
        &actual,
        expect![[r#"
            '', data
            '', bss
            '', text"#]],
    );
}

#[test]
fn hello_world() {
    check(
        &apply_lines(include_str!("./0-hello-world.asm"), crate::line),
        expect![[r#"
            '', data
            '', message db "Hello, World", 10 ; note the newline at the end
            '', empty
            '', text
            '', global _start
            '', empty
            '', _start
            '', mov rax, 1 ; system call for write
            '', mov rdi, 1 ; file handle 1 is stdout
            '', mov rsi, message ; address of string to output
            '', mov rdx, 13 ; number of bytes
            '', syscall  ; invoke operating system to do the write
            '', mov rax, 60 ; system call for exit
            '', xor rdi, rdi ; exit code 0
            '', syscall  ; invoke operating system to exit"#]],
    );
}
#[test]
fn print_any() {
    check(
        &apply_lines(include_str!("./1-print-any.asm"), crate::line),
        expect![[r#"
            '', data
            '', hello_world db "Hello, World!", 10, 0
            '', whats_up db "What's up", 10, 0
            '', long_text db "this is a longer line of text.", 10, 0
            '', empty
            '', text
            '', global _start
            '', empty
            '', _start
            '', mov rax, hello_world
            '', call print
            '', empty
            '', mov rax, whats_up
            '', call print
            '', empty
            '', mov rax, long_text
            '', call print
            '', empty
            '', mov rax, 60
            '', mov rdi, 0
            '', syscall 
            '', print
            '', push rax
            '', mov rbx, 0
            '', print_loop
            '', inc rax
            '', inc rbx
            '', mov cl, rax
            '', cmp cl, 0
            '', jne print_loop
            '', empty
            '', mov rax, 1
            '', mov rdi, 1
            '', pop rsi
            '', mov rdx, rbx
            '', syscall 
            '', empty
            '', ret "#]],
    );
}
