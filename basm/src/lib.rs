use ahash::AHashMap;
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner};

pub mod lex;
pub mod parse;

pub type Address = u16;

#[derive(Debug, Default)]
pub struct Basm<S> {
    pub si: StringInterner<DefaultBackend>,
    pub src: S,
    pub labels: AHashMap<DefaultSymbol, Address>,
    pub lines: Vec<Line>,
    pub sections: Vec<Section>,
}

pub fn transfer_basm<S1, S2>(
    Basm {
        si,
        labels,
        lines,
        sections,
        src: _,
    }: Basm<S2>,
    src: S1,
) -> Basm<S1> {
    Basm {
        src,
        si,
        labels,
        lines,
        sections,
    }
}

impl<S: Default> Basm<S> {
    fn new(src: S) -> Self {
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
        ins: Span,
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
    Deref(Span),
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
