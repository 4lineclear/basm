use crate::lexer::{
    lines::{LineItem, Lines},
    LexKind::*,
    Lexeme,
};

pub struct Parser<'a> {
    lines: Lines<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            lines: Lines::new(src),
        }
    }
}

impl Parser<'_> {
    pub fn parse_all(&mut self) {
        while self.next() {}
    }
    fn next(&mut self) -> bool {
        #[allow(unused)]
        match self.lines.next_li() {
            LineItem::Unit(a) => {}
            LineItem::Other(Lexeme { kind: Eof, len }) => return false,
            LineItem::Other(Lexeme { kind, len }) => {}
            LineItem::Other(l) => {}
        };
        true
    }
}
