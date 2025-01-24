use std::str::Chars;

// TODO: create a testing system that is synchronized between
// the lexing testing and the lsp's semantic token testing.

// TODO: create testing that ensures that the number of lines in
// lexing && parsing remain consistent

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lexeme {
    Whitespace,
    Ident,
    Str,
    Comma,
    Colon,
    OpenBracket,
    CloseBracket,
    Digit(DigitBase),
    Eol(bool),
    Eof,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitBase {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hex = 16,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    // NOTE: 'from' must come before 'to' for proper ordering
    pub from: u32,
    pub to: u32,
}

impl From<u32> for Span {
    fn from(value: u32) -> Self {
        Span::point(value)
    }
}
impl From<(u32, u32)> for Span {
    fn from((from, to): (u32, u32)) -> Self {
        Span::new(from, to)
    }
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

const EOF_CHAR: char = '\0';

pub trait Lexer {
    fn pop_peek(&mut self) -> Option<Advance>;
    fn peek(&mut self) -> Advance;
    fn advance(&mut self) -> Advance;
}

impl Lexer for BaseLexer<'_> {
    fn pop_peek(&mut self) -> Option<Advance> {
        self.pop_peek()
    }
    fn peek(&mut self) -> Advance {
        self.peek()
    }
    fn advance(&mut self) -> Advance {
        self.advance()
    }
}

#[derive(Debug)]
pub struct RecordedLexer<'a> {
    pub base: BaseLexer<'a>,
    store: Vec<Advance>,
}

impl<'a> RecordedLexer<'a> {
    pub fn new(src: &'a str) -> Self {
        let base = BaseLexer::new(src);
        let store = Vec::new();
        Self { base, store }
    }

    pub fn store(&self) -> &[Advance] {
        &self.store
    }
    pub fn parts(self) -> (BaseLexer<'a>, Vec<Advance>) {
        (self.base, self.store)
    }
}

impl Lexer for RecordedLexer<'_> {
    fn pop_peek(&mut self) -> Option<Advance> {
        self.base.pop_peek()
    }

    fn peek(&mut self) -> Advance {
        if self.base.prev.is_some() {
            self.base.peek()
        } else {
            let ad = self.base.peek();
            self.store.push(ad);
            ad
        }
    }

    fn advance(&mut self) -> Advance {
        if self.base.prev.is_some() {
            self.base.advance()
        } else {
            let ad = self.base.advance();
            self.store.push(ad);
            ad
        }
    }
}

#[derive(Debug)]
pub struct BaseLexer<'a> {
    pub src: &'a str,
    prev: Option<Advance>,
    chars: Chars<'a>,
    pos: u32,
    line: u32,
    line_start: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Advance {
    pub lex: Lexeme,
    pub line: u32,
    pub offset: u32,
    pub span: Span,
}

impl<'a> BaseLexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.chars(),
            prev: None,
            pos: 0,
            line: 0,
            line_start: 0,
        }
    }

    fn pop_peek(&mut self) -> Option<Advance> {
        self.prev.take()
    }

    fn peek(&mut self) -> Advance {
        if let Some(prev) = self.prev {
            prev
        } else {
            let next = self.advance();
            self.prev = Some(next);
            next
        }
    }

    fn advance(&mut self) -> Advance {
        if let Some(prev) = self.prev.take() {
            return prev;
        }
        let line = self.line;
        let offset = self.line_start;
        let start = self.pos;
        let Some(first_char) = self.bump() else {
            return Advance {
                lex: Lexeme::Eof,
                line,
                offset,
                span: start.into(),
            };
        };
        let lex = match first_char {
            ';' => {
                self.eat_while(|ch| ch != '\n');
                self.bump();
                Lexeme::Eol(true)
            }
            c if ws_not_nl(c) => self.whitespace(),
            c if is_id_start(c) => self.ident(),
            c @ '0'..='9' => Lexeme::Digit(self.number(c)),
            // One-symbol tokens.
            '\n' => Lexeme::Eol(false),
            ',' => Lexeme::Comma,
            '[' => Lexeme::OpenBracket,
            ']' => Lexeme::CloseBracket,
            ':' => Lexeme::Colon,
            // String literal.
            '"' => self.string(),
            _ => {
                self.eat_while(is_other);
                Lexeme::Other
            }
        };
        if let Lexeme::Eol(_) = lex {
            self.line += 1;
            self.line_start = self.pos();
        }
        let span = (start, self.pos()).into();
        Advance {
            lex,
            line,
            offset,
            span,
        }
    }

    fn whitespace(&mut self) -> Lexeme {
        self.eat_while(ws_not_nl);
        Lexeme::Whitespace
    }

    fn ident(&mut self) -> Lexeme {
        self.eat_while(is_id_continue);
        Lexeme::Ident
    }
    fn string(&mut self) -> Lexeme {
        while let Some(c) = self.bump() {
            match c {
                '"' => break,
                '\\' if self.first() == '\\' || self.first() == '"' => {
                    // Bump again to skip escaped character.
                    self.bump();
                }
                _ => (),
            }
        }
        Lexeme::Str
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
    fn first(&mut self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }
    #[allow(unused)]
    fn second(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }
    fn bump(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }
    /// Checks if there is nothing more to consume.
    #[must_use]
    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }
    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
    fn pos(&mut self) -> u32 {
        self.pos
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

/// returns true on all non newline whitespace
#[must_use]
const fn ws_not_nl(c: char) -> bool {
    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

// WARNING: needs to be synchronised with Lexer::advance
fn is_other(c: char) -> bool {
    !(ws_not_nl(c)
        | is_id_start(c)
        | matches!(c, '0'..='9' | '\n' | ',' | '[' | ']' | ':' | '"' | ';'))
}
