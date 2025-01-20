use ahash::AHashMap;

pub mod lex;
pub mod parse;

pub type Address = u16;

#[derive(Debug)]
pub struct Basm<'a> {
    pub src: &'a str,
    pub labels: AHashMap<&'a str, Address>,
    pub lines: Vec<Line<'a>>,
    pub sections: Vec<Section>,
}

#[derive(Debug)]
pub struct Section {
    pub kind: SectionKind,
    pub address: Address,
}

#[derive(Debug)]
pub enum SectionKind {
    Bss,
    Data,
    Text,
}

#[derive(Debug)]
pub enum Line<'a> {
    NoOp,
    Instruction {
        values: Vec<Value<'a>>,
    },
    VariableValue {
        name: &'a str,
        r#type: &'a str,
        value: Either<u16, Vec<Value<'a>>>,
    },
}

#[derive(Debug)]
pub enum Value<'a> {
    Digit(DigitKind, u32),
    Ident(&'a str),
    String(String),
}

#[derive(Debug)]
pub enum DigitKind {
    Binary,
    Octal,
    Decimal,
    Hex,
    Float,
}

#[derive(Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
