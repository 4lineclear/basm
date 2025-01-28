use expect_test::{expect, Expect};

fn debug_iter<D: std::fmt::Debug>(d: impl Iterator<Item = D>) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    d.into_iter().for_each(|d| write!(out, "\n{d:?}").unwrap());
    out
}

fn sorted_iter<T: std::cmp::Ord>(t: impl Iterator<Item = T>) -> Vec<T> {
    let mut t: Vec<_> = t.collect();
    t.sort();
    t
}

fn check(src: &str, expect: Expect) {
    let (
        crate::Code {
            si: _,
            sequences,
            variables,
            globals,
            labels,
        },
        errors,
    ) = super::reparse(src);
    let output = format!(
        "output:\nerrors:{}\nlabels:{}\nglobals:{}\nvariables:{}\nsequences:{}",
        match errors {
            basm::Either::A(errors) => debug_iter(errors.into_iter()),
            basm::Either::B(errors) => debug_iter(errors.into_iter()),
        },
        debug_iter(sorted_iter(labels.into_iter()).into_iter()),
        debug_iter(sorted_iter(globals.into_iter()).into_iter()),
        debug_iter(sorted_iter(variables.into_iter()).into_iter()),
        debug_iter(sequences.into_iter()),
    );
    expect.assert_eq(&output);
}

#[test]
fn empty() {
    check(
        "",
        expect![[r#"
        output:
        errors:
        labels:
        globals:
        variables:
        sequences:"#]],
    );
}

#[test]
fn empty_lines() {
    check(
        "\n\n\n\n",
        expect![[r#"
        output:
        errors:
        labels:
        globals:
        variables:
        sequences:"#]],
    );
}

#[test]
fn empty_lines_ws() {
    check(
        "   \t   \n    \t\t\t \n\t\t\t\n\n",
        expect![[r#"
        output:
        errors:
        labels:
        globals:
        variables:
        sequences:"#]],
    );
}

#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            output:
            errors:
            labels:
            (SymbolU32 { value: 4 }, 0)
            globals:
            SymbolU32 { value: 4 }
            variables:
            (SymbolU32 { value: 3 }, [18533, 27756, 28460, 8279, 28530, 27748, 10])
            sequences:
            Mov(ValueAndLoc { value: Digit(Decimal, 1), loc: Loc { deref: false, loc: SymbolU32 { value: 5 } } })
            Mov(ValueAndLoc { value: Digit(Decimal, 1), loc: Loc { deref: false, loc: SymbolU32 { value: 7 } } })
            Mov(ValueAndLoc { value: Ident(SymbolU32 { value: 3 }), loc: Loc { deref: false, loc: SymbolU32 { value: 8 } } })
            Mov(ValueAndLoc { value: Digit(Decimal, 13), loc: Loc { deref: false, loc: SymbolU32 { value: 9 } } })
            SysCall
            Mov(ValueAndLoc { value: Digit(Decimal, 60), loc: Loc { deref: false, loc: SymbolU32 { value: 5 } } })
            Xor(ValueAndLoc { value: Ident(SymbolU32 { value: 7 }), loc: Loc { deref: false, loc: SymbolU32 { value: 7 } } })
            SysCall"#]],
    );
}

#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            output:
            errors:
            labels:
            (SymbolU32 { value: 8 }, 0)
            (SymbolU32 { value: 11 }, 9)
            (SymbolU32 { value: 17 }, 11)
            globals:
            SymbolU32 { value: 8 }
            variables:
            (SymbolU32 { value: 3 }, [18533, 27756, 28460, 8279, 28530, 27748, 8448, 10, 0])
            (SymbolU32 { value: 5 }, [22376, 24948, 10099, 8309, 28672, 10, 0])
            (SymbolU32 { value: 7 }, [29800, 26995, 8297, 29472, 24864, 27759, 28263, 25970, 8300, 26990, 25888, 28518, 8308, 25976, 29742, 10, 0])
            sequences:
            Mov(ValueAndLoc { value: Ident(SymbolU32 { value: 3 }), loc: Loc { deref: false, loc: SymbolU32 { value: 9 } } })
            Call(Loc { deref: false, loc: SymbolU32 { value: 11 } })
            Mov(ValueAndLoc { value: Ident(SymbolU32 { value: 5 }), loc: Loc { deref: false, loc: SymbolU32 { value: 9 } } })
            Call(Loc { deref: false, loc: SymbolU32 { value: 11 } })
            Mov(ValueAndLoc { value: Ident(SymbolU32 { value: 7 }), loc: Loc { deref: false, loc: SymbolU32 { value: 9 } } })
            Call(Loc { deref: false, loc: SymbolU32 { value: 11 } })
            Mov(ValueAndLoc { value: Digit(Decimal, 60), loc: Loc { deref: false, loc: SymbolU32 { value: 9 } } })
            Mov(ValueAndLoc { value: Digit(Decimal, 0), loc: Loc { deref: false, loc: SymbolU32 { value: 13 } } })
            SysCall
            Push(Ident(SymbolU32 { value: 9 }))
            Mov(ValueAndLoc { value: Digit(Decimal, 0), loc: Loc { deref: false, loc: SymbolU32 { value: 16 } } })
            Inc(Loc { deref: false, loc: SymbolU32 { value: 9 } })
            Inc(Loc { deref: false, loc: SymbolU32 { value: 16 } })
            Mov(ValueAndLoc { value: Deref(SymbolU32 { value: 9 }), loc: Loc { deref: false, loc: SymbolU32 { value: 19 } } })
            Cmp(Ident(SymbolU32 { value: 19 }), Digit(Decimal, 0))
            Jne(Loc { deref: false, loc: SymbolU32 { value: 17 } })
            Mov(ValueAndLoc { value: Digit(Decimal, 1), loc: Loc { deref: false, loc: SymbolU32 { value: 9 } } })
            Mov(ValueAndLoc { value: Digit(Decimal, 1), loc: Loc { deref: false, loc: SymbolU32 { value: 13 } } })
            Pop(Loc { deref: false, loc: SymbolU32 { value: 22 } })
            Mov(ValueAndLoc { value: Ident(SymbolU32 { value: 16 }), loc: Loc { deref: false, loc: SymbolU32 { value: 24 } } })
            SysCall
            Ret"#]],
    );
}
