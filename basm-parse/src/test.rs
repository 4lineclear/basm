use expect_test::{expect, Expect};

fn debug_check<D: std::fmt::Debug>(src: D, expect: Expect) {
    check(&format!("{src:?}"), expect);
}
fn check(src: &str, expect: Expect) {
    expect.assert_eq(src);
}

#[test]
fn label() {
    debug_check(crate::label("   _one : "), expect![[r#"Ok(("", "_one"))"#]]);
}
#[test]
fn label_fail() {
    debug_check(
        crate::label("12one:"),
        expect![[r#"Err(Error(Error { input: "12one:", code: Tag }))"#]],
    );
}
#[test]
fn nullary() {
    debug_check(
        crate::instruction("   _one   "),
        expect![[r#"Ok(("", ("_one", None)))"#]],
    );
}
#[test]
fn unary() {
    debug_check(
        crate::instruction("   _one  two "),
        expect![[r#"Ok(("", ("_one", Some((Identifier("two"), None)))))"#]],
    );
}
#[test]
fn binary() {
    debug_check(
        crate::instruction("   _one  two,   threee "),
        expect![[r#"Ok(("", ("_one", Some((Identifier("two"), Some(Identifier("threee")))))))"#]],
    );
}
#[test]
fn nullary_list() {
    let actual = "one\ntwo\nthree\nfour"
        .lines()
        .map(crate::instruction)
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    check(
        &actual,
        expect![[r#"
            Ok(("", ("one", None)))
            Ok(("", ("two", None)))
            Ok(("", ("three", None)))
            Ok(("", ("four", None)))"#]],
    );
}
#[test]
fn unary_list() {
    let actual = ("one two\nthree four\nfive six\nseven eight")
        .lines()
        .map(crate::instruction)
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    check(
        &actual,
        expect![[r#"
            Ok(("", ("one", Some((Identifier("two"), None)))))
            Ok(("", ("three", Some((Identifier("four"), None)))))
            Ok(("", ("five", Some((Identifier("six"), None)))))
            Ok(("", ("seven", Some((Identifier("eight"), None)))))"#]],
    );
}
#[test]
fn unit_list_1() {
    let actual = "one\ntwo three\nfour five\nsix seven, eight"
        .lines()
        .inspect(|s| println!("{s}"))
        .map(|s| crate::line(s))
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n");

    check(
        &actual,
        expect![[r#"
            Ok(("", Nullary("one")))
            Ok(("", Unary("two", Identifier("three"))))
            Ok(("", Unary("four", Identifier("five"))))
            Ok(("", Binary("six", Identifier("seven"), Identifier("eight"))))"#]],
    );
}
#[test]
fn section() {
    debug_check(
        crate::section("   section .data"),
        expect![[r#"Ok(("", "data"))"#]],
    );
}
#[test]
fn sections() {
    let actual = "section .data\nsection .bss\nsection .text"
        .lines()
        .map(|s| crate::section(s))
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n");

    check(
        &actual,
        expect![[r#"
                Ok(("", "data"))
                Ok(("", "bss"))
                Ok(("", "text"))"#]],
    );
}

#[test]
fn hello_world() {
    let actual = r#"
section   .data
    message db "Hello, World", 10

section   .text
    global    _start

_start: 
    mov       rax, 1
    mov       rdi, 1
    mov       rsi, message
    mov       rdx, 13
    syscall
    mov       rax, 60
    xor       rdi, rdi
    syscall
"#
    .lines()
    .map(|s| crate::line(s))
    .map(|r| format!("{r:?}"))
    .collect::<Vec<_>>()
    .join("\n");

    check(
        &actual,
        expect![[r#"
            Ok(("", Empty))
            Ok(("", Section("data")))
            Err(Error(Error { input: "\"Hello, World\", 10", code: Eof }))
            Ok(("", Empty))
            Ok(("", Section("text")))
            Ok(("", Unary("global", Identifier("_start"))))
            Ok(("", Empty))
            Ok(("", Label("_start")))
            Ok(("", Binary("mov", Identifier("rax"), Decimal("1"))))
            Ok(("", Binary("mov", Identifier("rdi"), Decimal("1"))))
            Ok(("", Binary("mov", Identifier("rsi"), Identifier("message"))))
            Ok(("", Binary("mov", Identifier("rdx"), Decimal("13"))))
            Ok(("", Nullary("syscall")))
            Ok(("", Binary("mov", Identifier("rax"), Decimal("60"))))
            Ok(("", Binary("xor", Identifier("rdi"), Identifier("rdi"))))
            Ok(("", Nullary("syscall")))"#]],
    );
}
