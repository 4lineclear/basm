// TODO: move to using a hand-written parser.

use std::{
    iter::Peekable,
    str::{CharIndices, Lines},
};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub kind: LineKind,
    pub literals: (u32, u32),
    pub comment: Option<Span>,
}

#[derive(Default, Clone, Copy)]
pub struct Span {
    pub line_from: u32,
    pub line_to: u32,
    pub from: u32,
    pub to: u32,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}.{}, {}.{})",
            self.line_from, self.line_to, self.from, self.to
        )
    }
}

impl Span {
    pub fn to(mut self, other: Self) -> Self {
        self.line_to = other.line_to;
        self.to = other.to;
        self
    }
    pub fn between(mut self, other: Self) -> Self {
        self.from = self.to;
        self.to = other.from;
        self.line_to = self.line_to.max(other.line_from);
        self
    }
    pub fn point(line: u32, pos: u32) -> Self {
        Self {
            line_from: line,
            line_to: line + 1,
            from: pos,
            to: pos + 1,
        }
    }
    pub fn inline(line: u32, from: u32, to: u32) -> Self {
        Self {
            line_from: line,
            line_to: line + 1,
            from,
            to,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LineKind {
    Empty,
    Label(Span),
    Section(Span),
    Instruction(Span),
    Variable(Span, Span),
}

#[derive(Debug, Clone, Copy)]
pub enum Literal {
    Hex,
    Octal,
    Binary,
    Decimal,
    Float,
    Ident,
    Deref,
    String,
}

#[derive(Debug)]
pub enum LineError {
    LineFound,
    MissingComma,
    UnknownChar(char),
    UnclosedDeref,
}

type Charred<'a> = Peekable<CharIndices<'a>>;

#[derive(Debug)]
pub struct Lexer<'a> {
    src: &'a str,
    line: u32,
    lines: Lines<'a>,
    line_starts: Vec<u32>,
    errors: Vec<(Span, LineError)>,
    literals: Vec<(Span, Literal)>,
    chars: Charred<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            line: 0,
            lines: src.lines(),
            errors: Vec::new(),
            line_starts: vec![0],
            literals: Vec::new(),
            chars: "".char_indices().peekable(),
        }
    }
    pub fn src(&self) -> &str {
        self.src
    }
    pub fn errors(&self) -> &[(Span, LineError)] {
        &self.errors
    }
    pub fn literals(&self) -> &[(Span, Literal)] {
        &self.literals
    }
    pub fn line(&mut self) -> Option<Line> {
        let line = self.lines.next()?;
        self.chars = line.char_indices().peekable();

        let mut last_comma = 0;
        let mut post_comma = false;
        let mut deref_active = false;
        let mut comment = None;
        let mut kind = LineKind::Empty;

        let lit_start = self.literals.len() as u32;

        self.line_starts
            .push(self.line_starts.last().copied().unwrap_or(0) + line.len() as u32 + 1);

        while let Some((pos, ch)) = self.chars.next() {
            let pos = pos as u32;
            if ch == ',' {
                last_comma = 0;
                post_comma = true;
                continue;
            }

            if let ' ' | '\t' = ch {
                continue;
            }
            last_comma += 1;

            if let 'a'..='z' | 'A'..='Z' | '_' = ch {
                let end = self.ident(pos);
                let span = Span::inline(self.line, pos, end);
                self.check_comma(&mut kind, last_comma, lit_start, span);
                match (post_comma, &kind) {
                    (false, LineKind::Empty) => kind = LineKind::Instruction(span),
                    (false, LineKind::Instruction(s0)) if self.slice(*s0) == "section" => {
                        kind = LineKind::Section(span);
                    }
                    _ => self.literals.push((span, Literal::Ident)),
                };
                continue;
            }
            if let '"' = ch {
                let end = self.string(pos);
                let span = Span::inline(self.line, pos, end);
                self.check_comma(&mut kind, last_comma, lit_start, span);
                self.literals.push((span, Literal::String));
                continue;
            }
            if let ':' = ch {
                if let LineKind::Instruction(s) = kind {
                    kind = LineKind::Label(s)
                } else {
                    self.errors
                        .push((Span::point(self.line, pos), LineError::UnknownChar(ch)))
                }
                continue;
            }
            if let '0'..='9' = ch {
                if ch == '0' {
                } else {
                    let end = self.decimal(pos);
                    let span = Span::inline(self.line, pos, end);
                    self.check_comma(&mut kind, last_comma, lit_start, span);
                    self.literals.push((span, Literal::Decimal));
                }
                continue;
            }
            if let '[' = ch {
                deref_active = true;
                continue;
            }
            if let ']' = ch {
                if !deref_active {
                    continue;
                }
                deref_active = false;
                if let Some(b @ (_, Literal::Ident)) = self.literals.last_mut() {
                    b.1 = Literal::Deref;
                }
                continue;
            }
            if let ';' = ch {
                comment = Some(Span::inline(self.line, pos, line.len() as u32));
                break;
            }
            self.errors
                .push((Span::point(self.line, pos), LineError::UnknownChar(ch)));
        }
        self.line += 1;
        Some(Line {
            kind,
            literals: (lit_start, self.literals.len() as u32),
            comment,
        })
    }

    #[allow(unused)]
    fn skip_spaces(&mut self) -> u32 {
        let mut last = 0;
        while let Some((i, ' ' | '\t')) = self.chars.peek().copied() {
            last = i as u32;
            self.chars.next();
        }
        last + 1
    }

    fn check_comma(&mut self, kind: &mut LineKind, last_comma: i32, lit_start: u32, span: Span) {
        match &self.literals()[lit_start as usize..] {
            [(s1, Literal::Ident)] if last_comma > 1 => {
                if let LineKind::Instruction(s0) = kind {
                    *kind = LineKind::Variable(*s0, *s1);
                }
                self.literals.pop();
            }
            [.., l1] if last_comma > 1 => {
                self.errors
                    .push((l1.0.between(span), LineError::MissingComma));
            }
            _ => (),
        }
    }

    fn slice(&self, span: Span) -> &str {
        let f = self.line_starts[span.line_from as usize] + span.from;
        let t = self.line_starts[span.line_to as usize - 1] + span.to;
        debug_assert!(f <= t, "given span was invalid: {span:?}");
        &self.src()[f as usize..t as usize]
    }

    fn ident(&mut self, start: u32) -> u32 {
        let mut last = start;
        while let Some((i, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')) = self.chars.peek().copied() {
            last = i as u32;
            self.chars.next();
        }
        last + 1
    }
    fn decimal(&mut self, start: u32) -> u32 {
        let mut last = start;
        while let Some((i, '0'..='9' | '_' | '.')) = self.chars.peek().copied() {
            last = i as u32;
            self.chars.next();
        }
        last + 1
    }
    fn string(&mut self, start: u32) -> u32 {
        let mut last = start;
        loop {
            match self.chars.next() {
                Some((_, '"')) | None => break,
                Some((i, _)) => last = i as u32,
            }
        }
        last + 2
    }
}
