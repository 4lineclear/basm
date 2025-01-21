use ahash::AHashMap;

pub mod lex;
pub mod parse;

pub type Address = u16;

#[derive(Debug, Default)]
pub struct Basm<'a> {
    pub src: &'a str,
    pub labels: AHashMap<Span, Address>,
    pub lines: Vec<Line>,
    pub sections: Vec<Section>,
}

impl<'a> Basm<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            ..Default::default()
        }
    }
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
pub enum Line {
    NoOp,
    Instruction {
        values: Vec<Value>,
    },
    Variable {
        name: Span,
        r#type: Span,
        values: Vec<Value>,
    },
}

#[derive(Debug)]
pub enum Value {
    Ident(Span),
    String(String),
    Digit(DigitBase, u32),
}

pub use self::lex::DigitBase;
use self::lex::Span;

#[derive(Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
