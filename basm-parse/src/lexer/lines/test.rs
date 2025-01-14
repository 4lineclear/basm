use expect_test::{expect, Expect};

#[test]
fn inc() {
    check_parsing(
        "inc rax",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Ident }
        "#]],
    );
    check_parsing(
        "inc rax tax",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Ident }
            Unit { span: BSpan { from: 8, to: 11 }, kind: Ident }
        "#]],
    );
    check_parsing(
        "inc rax, max",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 9, to: 12 }, kind: Ident }
        "#]],
    );
}

#[test]
fn multi_inc() {
    check_parsing(
        "inc rax\ninc rax",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Ident }
            Unit { span: BSpan { from: 8, to: 11 }, kind: Ident }
            Unit { span: BSpan { from: 12, to: 15 }, kind: Ident }
        "#]],
    );
    check_parsing(
        "inc rax tax\ninc rax tax",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Ident }
            Unit { span: BSpan { from: 8, to: 11 }, kind: Ident }
            Unit { span: BSpan { from: 12, to: 15 }, kind: Ident }
            Unit { span: BSpan { from: 16, to: 19 }, kind: Ident }
            Unit { span: BSpan { from: 20, to: 23 }, kind: Ident }
        "#]],
    );
    check_parsing(
        "inc rax, max\ninc rax, max",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 9, to: 12 }, kind: Ident }
            Unit { span: BSpan { from: 13, to: 16 }, kind: Ident }
            Unit { span: BSpan { from: 17, to: 20 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 22, to: 25 }, kind: Ident }
        "#]],
    );
}

#[test]
fn int() {
    check_parsing(
        "inc 1 2 3, 123, 98234895234029384",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 5 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 1 } }
            Unit { span: BSpan { from: 6, to: 7 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 1 } }
            Unit { span: BSpan { from: 8, to: 9 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 1 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 11, to: 14 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 3 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 16, to: 33 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 17 } }
        "#]],
    );
}

#[test]
fn float() {
    check_parsing(
        "inc 1.0 2.0 3.0, 12.03f64, 9823489.05234029384",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 3 } }
            Unit { span: BSpan { from: 8, to: 11 }, kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 3 } }
            Unit { span: BSpan { from: 12, to: 15 }, kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 3 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 17, to: 25 }, kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 5 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 27, to: 46 }, kind: Literal { kind: Float { base: Decimal, empty_exponent: false }, suffix_start: 19 } }
        "#]],
    );
}

#[test]
fn char() {
    check_parsing(
        "inc 'a' 'b', 'c', '\\n'",
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 7 }, kind: Literal { kind: Char { terminated: true }, suffix_start: 3 } }
            Unit { span: BSpan { from: 8, to: 11 }, kind: Literal { kind: Char { terminated: true }, suffix_start: 3 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 13, to: 16 }, kind: Literal { kind: Char { terminated: true }, suffix_start: 3 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 18, to: 22 }, kind: Literal { kind: Char { terminated: true }, suffix_start: 4 } }
        "#]],
    );
}

#[test]
fn string() {
    check_parsing(
        r#"inc "asm" "basm", "compile" ,"Hello, World!""#,
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 9 }, kind: Literal { kind: Str { terminated: true }, suffix_start: 5 } }
            Unit { span: BSpan { from: 10, to: 16 }, kind: Literal { kind: Str { terminated: true }, suffix_start: 6 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 18, to: 27 }, kind: Literal { kind: Str { terminated: true }, suffix_start: 9 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 29, to: 44 }, kind: Literal { kind: Str { terminated: true }, suffix_start: 15 } }
        "#]],
    );
}

#[test]
fn raw_string() {
    check_parsing(
        r####"inc r"asm" r#"basm"#, r##"compile"## ,r###"Hello, World!###""####,
        expect![[r#"
            Unit { span: BSpan { from: 0, to: 3 }, kind: Ident }
            Unit { span: BSpan { from: 4, to: 10 }, kind: Literal { kind: RawStr { n_hashes: Some(0) }, suffix_start: 6 } }
            Unit { span: BSpan { from: 11, to: 20 }, kind: Literal { kind: RawStr { n_hashes: Some(1) }, suffix_start: 9 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 22, to: 36 }, kind: Literal { kind: RawStr { n_hashes: Some(2) }, suffix_start: 14 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 38, to: 60 }, kind: Literal { kind: RawStr { n_hashes: None }, suffix_start: 22 } }
        "#]],
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
            Unit { span: BSpan { from: 1, to: 8 }, kind: Ident }
            Lexeme { kind: Dot, len: 1 }
            Unit { span: BSpan { from: 12, to: 16 }, kind: Ident }
            Unit { span: BSpan { from: 21, to: 28 }, kind: Ident }
            Lexeme { kind: Colon, len: 1 }
            Unit { span: BSpan { from: 30, to: 32 }, kind: Ident }
            Unit { span: BSpan { from: 33, to: 47 }, kind: Literal { kind: Str { terminated: true }, suffix_start: 14 } }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 49, to: 51 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 2 } }
            Unit { span: BSpan { from: 53, to: 60 }, kind: Ident }
            Lexeme { kind: Dot, len: 1 }
            Unit { span: BSpan { from: 64, to: 68 }, kind: Ident }
            Unit { span: BSpan { from: 73, to: 79 }, kind: Ident }
            Unit { span: BSpan { from: 83, to: 89 }, kind: Ident }
            Unit { span: BSpan { from: 91, to: 97 }, kind: Ident }
            Lexeme { kind: Colon, len: 1 }
            Unit { span: BSpan { from: 104, to: 107 }, kind: Ident }
            Unit { span: BSpan { from: 114, to: 117 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 119, to: 120 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 1 } }
            Unit { span: BSpan { from: 125, to: 128 }, kind: Ident }
            Unit { span: BSpan { from: 135, to: 138 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 140, to: 141 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 1 } }
            Unit { span: BSpan { from: 146, to: 149 }, kind: Ident }
            Unit { span: BSpan { from: 156, to: 159 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 161, to: 168 }, kind: Ident }
            Unit { span: BSpan { from: 173, to: 176 }, kind: Ident }
            Unit { span: BSpan { from: 183, to: 186 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 188, to: 190 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 2 } }
            Unit { span: BSpan { from: 195, to: 202 }, kind: Ident }
            Unit { span: BSpan { from: 207, to: 210 }, kind: Ident }
            Unit { span: BSpan { from: 217, to: 220 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 222, to: 224 }, kind: Literal { kind: Int { base: Decimal, empty_int: false }, suffix_start: 2 } }
            Unit { span: BSpan { from: 229, to: 232 }, kind: Ident }
            Unit { span: BSpan { from: 239, to: 242 }, kind: Ident }
            Lexeme { kind: Comma, len: 1 }
            Unit { span: BSpan { from: 244, to: 247 }, kind: Ident }
            Unit { span: BSpan { from: 252, to: 259 }, kind: Ident }
        "#]],
    );
}

fn check_parsing(src: &str, expect: Expect) {
    use std::fmt::Write;
    let (li, parse) = super::parse(src);
    let mut actual: String = parse
        .comments
        .iter()
        .map(|e| format!("{e:?}\n"))
        .chain(parse.docs.iter().map(|e| format!("{e:?}\n")))
        .chain(parse.errors.iter().map(|e| format!("{e:?}\n")))
        .collect();
    for li in li {
        match li {
            // super::LineItem::Ins(instruction) => write!(actual, "{instruction:?}\n"),
            super::LineItem::Unit(unit) => write!(actual, "{unit:?}\n"),
            super::LineItem::Other(lexeme) => write!(actual, "{lexeme:?}\n"),
        }
        .unwrap();
    }
    // for ins in &parse.instrutions {
    //     actual.push_str(ins.span.slice(src));
    //     for arg in &parse.arguments[ins.arguments.range()] {
    //         actual.push(' ');
    //         actual.push_str(arg.span.slice(src));
    //     }
    //     actual.push('\n');
    // }
    expect.assert_eq(&actual)
}
