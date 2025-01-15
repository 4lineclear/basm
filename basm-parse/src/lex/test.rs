use expect_test::{expect, Expect};

use super::Span;

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
impl Stringify for super::Line<'_> {
    fn stringify(&self) -> String {
        self.kind.stringify()
            + &self
                .comment
                .map(|s| " ".to_owned() + &s.stringify())
                .unwrap_or("".to_owned())
    }
}
impl Stringify for super::LineKind<'_> {
    fn stringify(&self) -> String {
        use super::LineKind::*;
        match self {
            Empty => "".into(),
            Section(s) | Label(s) => s.stringify(),
            Ins(s, val) => s.stringify() + " " + &val.stringify(),
            Var { name, dir, val } => {
                name.stringify() + " " + &dir.stringify() + " " + &val.stringify()
            }
        }
    }
}
impl Stringify for super::Value<'_> {
    fn stringify(&self) -> String {
        use super::Value::*;
        match self {
            Hex(s) | Octal(s) | Binary(s) | Decimal(s) | Float(s) | Identifier(s) | Deref(s) => {
                s.stringify()
            }
            String(s) => format!("\"{s}\""),
        }
    }
}
impl Stringify for Vec<super::Value<'_>> {
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
    debug_check(super::label("   _one : ".into()), expect!["'', _one"]);
}
#[test]
fn label_fail() {
    debug_check(
        super::label("12one:".into()),
        expect![[
            r#"Error(Error { input: LocatedSpan { offset: 0, line: 1, fragment: "12one:", extra: () }, code: Tag })"#
        ]],
    );
}
#[test]
fn nullary() {
    debug_check(super::ins_or_var("   _one   ".into()), expect!["'', _one "]);
}
#[test]
fn unary() {
    debug_check(
        super::ins_or_var("   _one  two ".into()),
        expect!["'', _one two"],
    );
}
#[test]
fn binary() {
    debug_check(
        super::ins_or_var("   _one  two,   threee ".into()),
        expect!["'', _one two, threee"],
    );
}
#[test]
fn nullary_list() {
    check(
        &apply_lines("one\ntwo\nthree\nfour", super::ins_or_var),
        expect![[r#"
            '', one 
            '', two 
            '', three 
            '', four "#]],
    );
}
#[test]
fn unary_list() {
    check(
        &apply_lines(
            "one two\nthree four\nfive six\nseven eight",
            super::ins_or_var,
        ),
        expect![[r#"
            '', one two
            '', three four
            '', five six
            '', seven eight"#]],
    );
}
#[test]
fn unit_list_1() {
    check(
        &apply_lines(
            "one\ntwo three\nfour five\nsix seven, eight",
            super::ins_or_var,
        ),
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
        super::section("   section .data".into()),
        expect!["'', data"],
    );
}
#[test]
fn sections() {
    check(
        &apply_lines("section .data\nsection .bss\nsection .text", super::section),
        expect![[r#"
            '', data
            '', bss
            '', text"#]],
    );
}

#[test]
fn hello_world() {
    check(
        &apply_lines(include_str!("./test/0-hello-world.asm"), super::line),
        expect![[r#"
            '', data
            '', message db "Hello, World", 10 ; note the newline at the end
            '', 
            '', text
            '', global _start
            '', 
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
        &apply_lines(include_str!("./test/1-print-any.asm"), super::line),
        expect![[r#"
            '', data
            '', hello_world db "Hello, World!", 10, 0
            '', whats_up db "What's up", 10, 0
            '', long_text db "this is a longer line of text.", 10, 0
            '', 
            '', text
            '', global _start
            '', 
            '', _start
            '', mov rax, hello_world
            '', call print
            '', 
            '', mov rax, whats_up
            '', call print
            '', 
            '', mov rax, long_text
            '', call print
            '', 
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
            '', 
            '', mov rax, 1
            '', mov rdi, 1
            '', pop rsi
            '', mov rdx, rbx
            '', syscall 
            '', 
            '', ret "#]],
    );
}

#[test]
fn comments_strings() {
    check(
        &apply_lines(
            r#"
;;;;;; one comment
;;;;;; two comment
msg db "; not ; a ; comment ;", 0, 10 ; a comment
"#,
            super::line,
        ),
        expect![[r#"
            '', 
            '',  ;;;;;; one comment
            '',  ;;;;;; two comment
            '', msg db "; not ; a ; comment ;", 0, 10 ; a comment"#]],
    );
}
