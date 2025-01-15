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
        crate::ins_or_var("   _one   "),
        expect![[r#"Ok(("", Ins("_one", [])))"#]],
    );
}
#[test]
fn unary() {
    debug_check(
        crate::ins_or_var("   _one  two "),
        expect![[r#"Ok(("", Ins("_one", [Identifier("two")])))"#]],
    );
}
#[test]
fn binary() {
    debug_check(
        crate::ins_or_var("   _one  two,   threee "),
        expect![[r#"Ok(("", Ins("_one", [Identifier("two"), Identifier("threee")])))"#]],
    );
}
#[test]
fn nullary_list() {
    let actual = "one\ntwo\nthree\nfour"
        .lines()
        .map(crate::ins_or_var)
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    check(
        &actual,
        expect![[r#"
            Ok(("", Ins("one", [])))
            Ok(("", Ins("two", [])))
            Ok(("", Ins("three", [])))
            Ok(("", Ins("four", [])))"#]],
    );
}
#[test]
fn unary_list() {
    let actual = ("one two\nthree four\nfive six\nseven eight")
        .lines()
        .map(crate::ins_or_var)
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    check(
        &actual,
        expect![[r#"
            Ok(("", Ins("one", [Identifier("two")])))
            Ok(("", Ins("three", [Identifier("four")])))
            Ok(("", Ins("five", [Identifier("six")])))
            Ok(("", Ins("seven", [Identifier("eight")])))"#]],
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
            Ok(("", Ins("one", [])))
            Ok(("", Ins("two", [Identifier("three")])))
            Ok(("", Ins("four", [Identifier("five")])))
            Ok(("", Ins("six", [Identifier("seven"), Identifier("eight")])))"#]],
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
            Ok(("", Var { name: "message", dir: "db", val: [String("Hello, World"), Decimal("10")] }))
            Ok(("", Empty))
            Ok(("", Section("text")))
            Ok(("", Ins("global", [Identifier("_start")])))
            Ok(("", Empty))
            Ok(("", Label("_start")))
            Ok(("", Ins("mov", [Identifier("rax"), Decimal("1")])))
            Ok(("", Ins("mov", [Identifier("rdi"), Decimal("1")])))
            Ok(("", Ins("mov", [Identifier("rsi"), Identifier("message")])))
            Ok(("", Ins("mov", [Identifier("rdx"), Decimal("13")])))
            Ok(("", Ins("syscall", [])))
            Ok(("", Ins("mov", [Identifier("rax"), Decimal("60")])))
            Ok(("", Ins("xor", [Identifier("rdi"), Identifier("rdi")])))
            Ok(("", Ins("syscall", [])))"#]],
    );
}
