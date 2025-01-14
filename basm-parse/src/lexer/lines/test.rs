use expect_test::{expect, Expect};

#[test]
fn inc() {
    check_parsing(
        "inc rax",
        expect![[r#"
            inc rax
        "#]],
    );
    check_parsing(
        "inc rax tax",
        expect![[r#"
            inc rax tax
        "#]],
    );
    check_parsing(
        "inc rax, max",
        expect![[r#"
            inc rax max
        "#]],
    );
}

#[test]
fn multi_inc() {
    check_parsing(
        "inc rax\ninc rax",
        expect![[r#"
            inc rax
            inc rax
        "#]],
    );
    check_parsing(
        "inc rax tax\ninc rax tax",
        expect![[r#"
            inc rax tax
            inc rax tax
        "#]],
    );
    check_parsing(
        "inc rax, max\ninc rax, max",
        expect![[r#"
            inc rax max
            inc rax max
        "#]],
    );
}

#[test]
fn int() {
    check_parsing(
        "inc 1 2 3, 123, 98234895234029384",
        expect![[r#"
            inc 1 2 3 123 98234895234029384
        "#]],
    );
}

#[test]
fn float() {
    check_parsing(
        "inc 1.0 2.0 3.0, 12.03f64, 9823489.05234029384",
        expect![[r#"
            inc 1.0 2.0 3.0 12.03f64 9823489.05234029384
        "#]],
    );
}

#[test]
fn char() {
    check_parsing(
        "inc 'a' 'b', 'c', '\\n'",
        expect![[r#"
            inc 'a' 'b' 'c' '\n'
        "#]],
    );
}

#[test]
fn string() {
    check_parsing(
        r#"inc "asm" "basm", "compile" ,"Hello, World!""#,
        expect![[r#"
            inc "asm" "basm" "compile" "Hello, World!"
        "#]],
    );
}

#[test]
fn raw_string() {
    check_parsing(
        r####"inc r"asm" r#"basm"#, r##"compile"## ,r###"Hello, World!###""####,
        expect![[r###"
            inc r"asm" r#"basm"# r##"compile"## r###"Hello, World!###"
        "###]],
    );
}

#[test]
fn hello_world() {
    check_parsing(
        r#"
section   .data
    message: db "Hello, World", 10

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
    syscall"#,
        expect![[r#"
            section data
            message db "Hello, World" 10
            section text
            global _start
            _start
            mov rax 1
            mov rdi 1
            mov rsi message
            mov rdx 13
            syscall
            mov rax 60
            xor rdi rdi
            syscall
        "#]],
    );
}

fn check_parsing(src: &str, expect: Expect) {
    let parse = super::parse(src);
    let mut actual: String = parse
        .comments
        .iter()
        .map(|e| format!("{e:?}\n"))
        .chain(parse.docs.iter().map(|e| format!("{e:?}\n")))
        .chain(parse.errors.iter().map(|e| format!("{e:?}\n")))
        .collect();
    for ins in &parse.instrutions {
        actual.push_str(ins.span.slice(src));
        for arg in &parse.arguments[ins.arguments.range()] {
            actual.push(' ');
            actual.push_str(arg.span.slice(src));
        }
        actual.push('\n');
    }
    expect.assert_eq(&actual)
}
