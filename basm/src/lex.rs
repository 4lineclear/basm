use std::{
    iter::Peekable,
    str::{CharIndices, Lines},
};

use self::context::Context;

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
                let start = lexer.line_starts[i - 1];
                let end = lexer.line_starts[i];
                Some(AssembledLine { start, line, end })
            })
            .collect();
        let literals = lexer.context.parts();

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
    pub line: Line,
}

impl AssembledLine {
    pub fn line_src<'a>(&self, src: &'a str) -> &'a str {
        &src[self.start as usize..self.end as usize - 1]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub kind: LineKind,
    pub literals: (u32, u32),
    pub comment: Option<Span>,
}

impl Line {
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
    Binary,
    Octal,
    Decimal,
    // TODO: currently floats are saved as decimals,
    // save them as floats instead
    // Float,
    Hex,
    Ident,
    String,
    Comma,
    Colon,
    OpenBracket,
    CloseBracket,
    Other,
}

type Charred<'a> = Peekable<CharIndices<'a>>;

#[derive(Debug)]
pub struct Lexer<'a> {
    lines: Lines<'a>,
    line_starts: Vec<u32>,
    context: Context<'a>,
    chars: Charred<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            lines: src.lines(),
            line_starts: vec![0],
            context: Context::default(),
            chars: "".char_indices().peekable(),
        }
    }
    // pub fn literals()
    pub fn line(&mut self) -> Option<Line> {
        let line = self.lines.next()?;
        Some(self.line_inner(line))
    }
    fn line_inner(&mut self, line: &'a str) -> Line {
        self.chars = line.char_indices().peekable();
        self.context.reset(line);
        self.line_starts
            .push(self.line_starts.last().copied().unwrap_or(0) + line.len() as u32 + 1);
        let comment = loop {
            let Some((pos, ch)) = self.chars.next() else {
                break None;
            };
            let pos = pos as u32;
            match ch {
                _ if ch.is_whitespace() => self.context.push_lit(pos, Literal::Whitespace),
                'a'..='z' | 'A'..='Z' | '_' => {
                    let end = self.ident(pos);
                    self.context.push_lit((pos, end), Literal::Ident);
                }
                '"' => {
                    let end = self.string(pos);
                    self.context.push_lit((pos, end), Literal::String);
                }
                '0'..='9' => {
                    let (span, lit) = self.digit(pos, ch);
                    self.context.push_lit(span, lit);
                }
                '[' => self.context.push_lit(pos, Literal::OpenBracket),
                ']' => self.context.push_lit(pos, Literal::CloseBracket),
                //         '[' => {
                //             if let Some((_, deref)) = deref {
                //                 self.push_other(Span::point(deref), &mut muddled)
                //             }
                //             deref = Some((self.literals.len(), pos));
                //         }
                //         ']' if deref.is_some() => {
                //             let (orig, deref_open) = deref.unwrap();
                //             let span = Span::new(deref_open, pos + 1);
                //             match &mut self.literals[orig..] {
                //                 [l @ (_, Literal::Ident)] => *l = (span, Literal::Deref),
                //                 [.., _] => self.errors.push((span, LineError::MuddyDeref)),
                //                 [] => self.errors.push((span, LineError::EmptyDeref)),
                //             }
                //             deref = None;
                //         }
                ':' => self.context.push_lit(pos, Literal::Colon),
                ',' => self.context.push_lit(pos, Literal::Comma),
                ';' => break Some(Span::new(pos, line.len() as u32)),
                _ => self.context.push_lit(pos, Literal::Other),
            }
        };
        Line {
            comment,
            ..self.context.line()
        }
    }

    fn until(&mut self, start: u32, check: impl Fn(char) -> bool) -> u32 {
        let mut last = start;
        while let Some((i, ch)) = self.chars.peek().copied() {
            if !check(ch) {
                break;
            }
            last = i as u32;
            self.chars.next();
        }
        last + 1
    }
    fn ident(&mut self, start: u32) -> u32 {
        self.until(
            start,
            |ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'),
        )
    }
    fn digit(&mut self, pos: u32, ch: char) -> (Span, Literal) {
        let (end, lit) = match self
            .chars
            .next_if(|(_, a)| ch == '0' && matches!(*a, 'b' | 'o' | 'x'))
        {
            Some((_, 'b')) => (self.binary(pos), Literal::Binary),
            Some((_, 'o')) => (self.octal(pos), Literal::Octal),
            Some((_, 'x')) => (self.hex(pos), Literal::Hex),
            _ => (self.decimal(pos), Literal::Decimal),
        };
        (Span::new(pos, end), lit)
    }
    fn hex(&mut self, start: u32) -> u32 {
        self.ident(start)
    }
    fn decimal(&mut self, start: u32) -> u32 {
        self.until(start, |ch| matches!(ch, '0'..='9' | '_' | '.'))
    }
    fn octal(&mut self, start: u32) -> u32 {
        self.until(start, |ch| matches!(ch, '0'..='7' | '_'))
    }
    fn binary(&mut self, start: u32) -> u32 {
        self.until(start, |ch| matches!(ch, '0' | '1' | '_'))
    }
    // TODO: create better string parsing
    fn string(&mut self, start: u32) -> u32 {
        let mut last = start;
        loop {
            match self.chars.next() {
                Some((_, '"')) => break last + 2,
                Some((i, _)) => last = i as u32,
                None => break last + 1,
            }
        }
    }
}
