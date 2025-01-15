use expect_test::{expect, Expect};

fn debug_check<D: std::fmt::Debug>(src: D, expect: Expect) {
    check(&format!("{src:?}"), expect);
}
fn check(src: &str, expect: Expect) {
    expect.assert_eq(src);
}

fn apply_lines<'a, D: std::fmt::Debug + 'a>(src: &'a str, apply: impl Fn(&'a str) -> D) -> String {
    src.lines()
        .map(apply)
        .map(|r| format!("{r:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn label() {
    debug_check(crate::label("   _one : "), expect![[r#"Ok(("", "_one"))"#]]);
}
#[test]
fn label_fail() {
    panic!("{:#?}", crate::label("yeah()"));
    // println!("");
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
    let actual = apply_lines("section .data\nsection .bss\nsection .text", crate::section);

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
    check(
        &apply_lines(include_str!("./0-hello-world.asm"), crate::line),
        expect![[r#"
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
#[test]
fn print_any() {
    check(
        &apply_lines(include_str!("./1-print-any.asm"), crate::line),
        expect![[r#"
            Ok(("", Section("data")))
            Ok(("", Var { name: "hello_world", dir: "db", val: [String("Hello, World!"), Decimal("10"), Decimal("0")] }))
            Ok(("", Var { name: "whats_up", dir: "db", val: [String("What's up"), Decimal("10"), Decimal("0")] }))
            Ok(("", Var { name: "long_text", dir: "db", val: [String("this is a longer line of text."), Decimal("10"), Decimal("0")] }))
            Ok(("", Empty))
            Ok(("", Section("text")))
            Ok(("", Ins("global", [Identifier("_start")])))
            Ok(("", Empty))
            Ok(("", Label("_start")))
            Ok(("", Ins("mov", [Identifier("rax"), Identifier("hello_world")])))
            Ok(("", Ins("call", [Identifier("print")])))
            Ok(("", Empty))
            Ok(("", Ins("mov", [Identifier("rax"), Identifier("whats_up")])))
            Ok(("", Ins("call", [Identifier("print")])))
            Ok(("", Empty))
            Ok(("", Ins("mov", [Identifier("rax"), Identifier("long_text")])))
            Ok(("", Ins("call", [Identifier("print")])))
            Ok(("", Empty))
            Ok(("", Ins("mov", [Identifier("rax"), Decimal("60")])))
            Ok(("", Ins("mov", [Identifier("rdi"), Decimal("0")])))
            Ok(("", Ins("syscall", [])))
            Ok(("", Label("print")))
            Ok(("", Ins("push", [Identifier("rax")])))
            Ok(("", Ins("mov", [Identifier("rbx"), Decimal("0")])))
            Ok(("", Label("print_loop")))
            Ok(("", Ins("inc", [Identifier("rax")])))
            Ok(("", Ins("inc", [Identifier("rbx")])))
            Ok(("", Ins("mov", [Identifier("cl"), Deref("rax")])))
            Ok(("", Ins("cmp", [Identifier("cl"), Decimal("0")])))
            Ok(("", Ins("jne", [Identifier("print_loop")])))
            Ok(("", Empty))
            Ok(("", Ins("mov", [Identifier("rax"), Decimal("1")])))
            Ok(("", Ins("mov", [Identifier("rdi"), Decimal("1")])))
            Ok(("", Ins("pop", [Identifier("rsi")])))
            Ok(("", Ins("mov", [Identifier("rdx"), Identifier("rbx")])))
            Ok(("", Ins("syscall", [])))
            Ok(("", Empty))
            Ok(("", Ins("ret", [])))"#]],
    );
}
