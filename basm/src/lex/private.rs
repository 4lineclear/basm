use std::str::{Chars, Lines};
use std::u32;

use super::context::{Context, Spanned};
use super::{LexLine, Literal, Span};

#[derive(Debug)]
pub struct Lexer<'a> {
    lines: Lines<'a>,
    line_starts: Vec<u32>,
    context: Context<'a>,
    chars: Chars<'a>,
    pos: u32,
}

const EOF_CHAR: char = '\0';

impl<'a> Lexer<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        Self {
            lines: src.lines(),
            line_starts: vec![0],
            context: Context::default(),
            chars: "".chars(),
            pos: 0,
        }
    }
    pub(crate) fn parts(self) -> Vec<(Span, Literal)> {
        self.context.parts()
    }
    pub(crate) fn literals(&self) -> &[(Span, Literal)] {
        self.context.literals()
    }
    pub(crate) fn line_starts(&self) -> &[u32] {
        &self.line_starts
    }
    pub(crate) fn get_line(&mut self) -> Option<&'a str> {
        self.lines.next()
    }
    pub(crate) fn reset(&mut self, line: &'a str) {
        self.pos = 0;
        self.chars = line.chars();
        self.context.reset(line);
        self.line_starts
            .push(self.line_starts.last().copied().unwrap_or(0) + line.len() as u32 + 1);
    }
    pub(crate) fn first(&mut self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }
    #[allow(unused)]
    pub(crate) fn second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }
    pub(crate) fn bump(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }
    pub(crate) fn push_lit(&mut self, span: impl Spanned, lit: Literal) {
        self.context.push_lit(span, lit);
    }
    pub(crate) fn line_info(&mut self) -> LexLine {
        self.context.line()
    }
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }
    pub(crate) fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
    pub(crate) fn pos(&mut self) -> u32 {
        self.pos
    }
}
