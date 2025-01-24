use string_interner::{DefaultBackend, DefaultSymbol, StringInterner};

pub mod lex;
pub mod parse;

pub type Address = u16;

#[derive(Debug, Default)]
pub struct Basm {
    pub si: StringInterner<DefaultBackend>,
    pub lines: Vec<Line>,
}

#[derive(Debug)]
pub enum Line {
    NoOp,
    Section {
        name: DefaultSymbol,
    },
    Label {
        name: DefaultSymbol,
    },
    Instruction {
        ins: DefaultSymbol,
        values: Vec<Value>,
    },
    Variable {
        name: DefaultSymbol,
        r#type: DefaultSymbol,
        values: Vec<Value>,
    },
}

#[derive(Debug)]
pub enum Value {
    Deref(DefaultSymbol),
    Ident(DefaultSymbol),
    String(DefaultSymbol),
    Digit(DigitBase, u32),
}

pub use self::lex::DigitBase;

#[derive(Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
