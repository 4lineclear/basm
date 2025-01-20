#[cfg(test)]
mod test;

mod context;

// TODO: create float lexing

// TODO: create a testing system that is synchronized between
// the lexing testing and the lsp's semantic token testing.

#[derive(Debug)]
pub struct LexOutput<S> {
    pub src: S,
    pub lines: Vec<AssembledLine>,
    pub literals: Vec<(Span, Literal)>,
}

impl<S> LexOutput<S>
where
    S: AsRef<str>,
{
    pub fn lex_all(src: S) -> Self {
        let mut lexer = Lexer::new(src.as_ref());
        let lines: Vec<_> = (1..)
            .map_while(|i| {
                let line = lexer.line()?;
                let start = lexer.line_starts()[i - 1];
                let end = lexer.line_starts()[i];
                Some(AssembledLine { start, line, end })
            })
            .collect();
        let literals = lexer.parts();

        Self {
            src,
            lines,
            literals,
        }
    }

    pub fn line_src(&self, line: usize) -> &str {
        self.lines[line].line_src(self.src.as_ref())
    }
    pub fn line_range(&self, line: usize) -> (u32, u32) {
        (self.lines[line].start, self.lines[line].end)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AssembledLine {
    pub start: u32,
    pub end: u32,
    pub line: LexLine,
}

impl AssembledLine {
    pub fn line_src<'a>(&self, src: &'a str) -> &'a str {
        &src[self.start as usize..self.end as usize - 1]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LexLine {
    pub kind: LineKind,
    pub literals: (u32, u32),
    pub comment: Option<Span>,
}

impl LexLine {
    pub fn slice_lit<'a, T>(&self, literals: &'a [T]) -> &'a [T] {
        &literals[self.literals.0 as usize..self.literals.1 as usize]
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    // NOTE: 'from' must come before 'to' for proper ordering
    pub from: u32,
    pub to: u32,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.from, self.to)
    }
}

impl Span {
    pub fn slice(self, src: &str) -> &str {
        &src[self.from as usize..self.to as usize]
    }
    pub fn to(mut self, other: Self) -> Self {
        self.to = other.to;
        self
    }
    pub fn between(mut self, other: Self) -> Self {
        self.from = self.to;
        self.to = other.from;
        self
    }
    pub fn point(pos: u32) -> Self {
        Self {
            from: pos,
            to: pos + 1,
        }
    }
    pub fn new(from: u32, to: u32) -> Self {
        Self { from, to }
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.from += offset;
        self.to += offset;
        self
    }

    pub fn len(&self) -> u32 {
        self.to - self.from
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineKind {
    #[default]
    Empty,
    Label,
    Section,
    Global,
    Instruction,
    Variable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    Whitespace,
    Digit(DigitBase),
    Ident,
    String,
    Comma,
    Colon,
    OpenBracket,
    CloseBracket,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitBase {
    Binary,
    Octal,
    Decimal,
    Hex,
}

pub use private::Lexer;

mod private {
    use std::str::{Chars, Lines};
    use std::u32;

    use super::context::{Context, Spanned};
    use super::{LexLine, Literal};

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
        pub(crate) fn parts(self) -> Vec<(super::Span, Literal)> {
            self.context.parts()
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
}

impl<'a> Lexer<'a> {
    pub fn line(&mut self) -> Option<LexLine> {
        let line = self.get_line()?;
        self.reset(line);
        let comment = loop {
            match self.advance(line) {
                Some(None) => (),
                Some(Some(comment)) => break Some(comment),
                None => break None,
            }
        };
        Some(LexLine {
            comment,
            ..self.line_info()
        })
    }
    fn advance(&mut self, line: &'a str) -> Option<Option<Span>> {
        let pos = self.pos();
        let Some(ch) = self.bump() else {
            return None;
        };
        match ch {
            _ if ch.is_whitespace() => self.push_lit(pos, Literal::Whitespace),
            _ if is_id_start(ch) => {
                self.ident();
                let span = (pos, self.pos());
                self.push_lit(span, Literal::Ident);
            }
            '"' => {
                self.string();
                let span = (pos, self.pos());
                self.push_lit(span, Literal::String);
            }
            '0'..='9' => {
                let base = self.number(ch);
                let span = (pos, self.pos());
                self.push_lit(span, Literal::Digit(base));
            }
            '[' => self.push_lit(pos, Literal::OpenBracket),
            ']' => self.push_lit(pos, Literal::CloseBracket),
            ':' => self.push_lit(pos, Literal::Colon),
            ',' => self.push_lit(pos, Literal::Comma),
            ';' => return Some(Some(Span::new(pos, line.len() as u32))),
            _ => self.push_lit(pos, Literal::Other),
        }
        Some(None)
    }

    fn ident(&mut self) {
        self.eat_while(is_id_continue);
    }
    fn string(&mut self) {
        while let Some(c) = self.bump() {
            match c {
                '"' => return,
                '\\' if self.first() == '\\' || self.first() == '"' => {
                    // Bump again to skip escaped character.
                    self.bump();
                }
                _ => (),
            }
        }
    }

    // TODO: eventually add floats back in
    fn number(&mut self, first_digit: char) -> DigitBase {
        // dassert!('0' <= self.prev() && self.prev() <= '9');
        let mut base = DigitBase::Decimal;
        if first_digit == '0' {
            // Attempt to parse encoding base.
            match self.first() {
                'b' => {
                    base = DigitBase::Binary;
                    self.bump();
                    if !self.eat_decimal_digits() {
                        return DigitBase::Decimal;
                    }
                }
                'o' => {
                    base = DigitBase::Octal;
                    self.bump();
                    if !self.eat_decimal_digits() {
                        return DigitBase::Decimal;
                    }
                }
                'x' => {
                    base = DigitBase::Hex;
                    self.bump();
                    if !self.eat_hexadecimal_digits() {
                        return DigitBase::Decimal;
                    }
                }
                // Not a base prefix; consume additional digits.
                '0'..='9' | '_' => {
                    self.eat_decimal_digits();
                }
                // Just a 0.
                _ => return DigitBase::Decimal,
            }
        } else {
            // No base prefix, parse number in the usual way.
            self.eat_decimal_digits();
        };
        base
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }

    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {
                    self.bump();
                }
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    self.bump();
                }
                _ => break,
            }
        }
        has_digits
    }
}

fn is_id_start(first: char) -> bool {
    matches!(first,
    'a'..='z' | 'A'..='Z' | '_'
        )
}
fn is_id_continue(ch: char) -> bool {
    matches!(ch,'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
}
