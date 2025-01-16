use std::{
    iter::Peekable,
    str::{CharIndices, Lines},
};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub kind: LineKind,
    pub errors: (u32, u32),
    pub literals: (u32, u32),
    pub comment: Option<Span>,
}

#[derive(Default, Clone, Copy)]
pub struct Span {
    pub from: u32,
    pub to: u32,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.from, self.to)
    }
}

impl Span {
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

        let err_start = self.errors.len() as u32;
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
                let span = Span::new(pos, end);
                self.check_comma(&mut kind, last_comma, lit_start, span);
                match (post_comma, &kind) {
                    (false, LineKind::Empty) => kind = LineKind::Instruction(span),
                    (false, LineKind::Instruction(s0))
                        if self.slice(self.line, *s0) == "section" =>
                    {
                        kind = LineKind::Section(span);
                    }
                    _ => self.literals.push((span, Literal::Ident)),
                };
                continue;
            }
            if let '"' = ch {
                let end = self.string(pos);
                let span = Span::new(pos, end);
                self.check_comma(&mut kind, last_comma, lit_start, span);
                self.literals.push((span, Literal::String));
                continue;
            }
            if let ':' = ch {
                if let LineKind::Instruction(s) = kind {
                    kind = LineKind::Label(s)
                } else {
                    self.errors
                        .push((Span::point(pos), LineError::UnknownChar(ch)))
                }
                continue;
            }
            if let '0'..='9' = ch {
                let end = match self
                    .chars
                    .next_if(|(_, a)| ch == '0' && matches!(*a, 'b' | 'o' | 'x'))
                {
                    Some((_, 'b')) => self.binary(pos),
                    Some((_, 'o')) => self.octal(pos),
                    Some((_, 'x')) => self.hex(pos),
                    _ => self.decimal(pos),
                };
                let span = Span::new(pos, end);
                self.check_comma(&mut kind, last_comma, lit_start, span);
                self.literals.push((span, Literal::Decimal));
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
                comment = Some(Span::new(pos, line.len() as u32));
                break;
            }
            self.errors
                .push((Span::point(pos), LineError::UnknownChar(ch)));
        }
        self.line += 1;
        Some(Line {
            kind,
            errors: (err_start, self.errors.len() as u32),
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

    fn slice(&self, line: u32, span: Span) -> &str {
        let base = self.line_starts[line as usize];
        let f = base + span.from;
        let t = base + span.to;
        debug_assert!(f <= t, "given span was invalid: {span:?}");
        &self.src()[f as usize..t as usize]
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

    fn hex(&mut self, start: u32) -> u32 {
        self.ident(start)
    }

    fn decimal(&mut self, start: u32) -> u32 {
        self.until(start, |ch| matches!(ch, '0'..='9' | '_'))
    }

    fn octal(&mut self, start: u32) -> u32 {
        self.until(start, |ch| matches!(ch, '0'..='7' | '_'))
    }

    fn binary(&mut self, start: u32) -> u32 {
        self.until(start, |ch| matches!(ch, '0' | '1' | '_'))
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
