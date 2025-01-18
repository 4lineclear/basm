use std::{
    iter::Peekable,
    str::{CharIndices, Lines},
};

#[cfg(test)]
mod test;

// TODO: create float lexing

// TODO: create a testing system that is synchronized between
// the lexing testing and the lsp's semantic token testing.

#[derive(Debug)]
pub struct LexOutput<S> {
    pub src: S,
    pub lines: Vec<AssembledLine>,
    pub errors: Vec<(Span, LineError)>,
    pub literals: Vec<(Span, Literal)>,
}

impl<S> From<S> for LexOutput<S>
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::lex_all(value)
    }
}

impl<S> LexOutput<S>
where
    S: AsRef<str>,
{
    pub fn lex_all(src: S) -> Self {
        let mut lexer = Lexer::new(src.as_ref());
        let mut i = 1;
        let lines: Vec<_> = std::iter::from_fn(|| {
            let line = lexer.line()?;
            let start = lexer.line_starts[i - 1];
            let end = lexer.line_starts[i];
            i += 1;
            Some(AssembledLine { start, line, end })
        })
        .collect();
        let errors = lexer.errors;
        let literals = lexer.literals;

        Self {
            src,
            lines,
            errors,
            literals,
        }
    }

    pub fn line_src(&self, line: usize) -> &str {
        let al = &self.lines[line];
        &self.src.as_ref()[al.start as usize..al.end as usize - 1]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AssembledLine {
    start: u32,
    end: u32,
    pub line: Line,
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub kind: LineKind,
    pub errors: (u32, u32),
    pub literals: (u32, u32),
    pub comment: Option<Span>,
}

impl Line {
    pub fn slice_lit<'a, T>(&'a self, literals: &'a [T]) -> &'a [T] {
        &literals[self.literals.0 as usize..self.literals.1 as usize]
    }
    pub fn slice_err<'a, T>(&'a self, literals: &'a [T]) -> &'a [T] {
        &literals[self.errors.0 as usize..self.errors.1 as usize]
    }
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
}

#[derive(Debug, Clone, Copy)]
pub enum LineKind {
    Empty,
    Label(Span),
    Section(Span, Span),
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
    MissingComma,
    UnknownChar(char),
    UnclosedDeref,
    /// A deref with incorrect token
    EmptyDeref,
    /// A deref with incorrect token
    MuddyDeref,
}

type Charred<'a> = Peekable<CharIndices<'a>>;

#[derive(Debug)]
pub struct Lexer<'a> {
    lines: Lines<'a>,
    line_starts: Vec<u32>,
    errors: Vec<(Span, LineError)>,
    literals: Vec<(Span, Literal)>,
    chars: Charred<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            lines: src.lines(),
            errors: Vec::new(),
            line_starts: vec![0],
            literals: Vec::new(),
            chars: "".char_indices().peekable(),
        }
    }
    pub fn line(&mut self) -> Option<Line> {
        let line = self.lines.next()?;
        Some(self.line_inner(line))
    }
    fn line_inner(&mut self, line: &'a str) -> Line {
        self.chars = line.char_indices().peekable();

        let mut last_comma = 0;
        let mut deref = None;
        let mut comment = None;
        let mut kind = LineKind::Empty;

        let err_start = self.errors.len() as u32;
        let lit_start = self.literals.len() as u32;

        self.line_starts
            .push(self.line_starts.last().copied().unwrap_or(0) + line.len() as u32 + 1);

        while let Some((pos, ch)) = self.chars.next() {
            let pos = pos as u32;

            match ch {
                ',' => last_comma = 0,
                ' ' | '\t' => (),
                'a'..='z' | 'A'..='Z' | '_' => {
                    let span = Span::new(pos, self.ident(pos));
                    self.check_comma(&mut kind, &mut last_comma, lit_start, span);
                    match (&kind, self.literals.len() - lit_start as usize) {
                        (LineKind::Empty, 0) => kind = LineKind::Instruction(span),
                        (LineKind::Instruction(s0), 0) if s0.slice(line) == "section" => {
                            kind = LineKind::Section(*s0, span);
                        }
                        _ => self.literals.push((span, Literal::Ident)),
                    };
                }
                '"' => {
                    let span = Span::new(pos, self.string(pos));
                    self.check_comma(&mut kind, &mut last_comma, lit_start, span);
                    self.literals.push((span, Literal::String));
                }
                ':' => {
                    if let LineKind::Instruction(s) = kind {
                        kind = LineKind::Label(s)
                    } else {
                        self.errors
                            .push((Span::point(pos), LineError::UnknownChar(ch)))
                    }
                }
                '0'..='9' => {
                    let (span, lit) = self.digit(pos, ch);
                    self.check_comma(&mut kind, &mut last_comma, lit_start, span);
                    self.literals.push((span, lit));
                }
                '[' => {
                    if let Some((_, deref)) = deref {
                        self.errors
                            .push((Span::point(deref), LineError::UnknownChar(ch)));
                    }
                    deref = Some((self.literals.len(), pos));
                }
                ']' if deref.is_some() => {
                    let (orig, deref_open) = deref.unwrap();
                    let span = Span::new(deref_open, pos + 1);
                    match &mut self.literals[orig..] {
                        [l @ (_, Literal::Ident)] => *l = (span, Literal::Deref),
                        [.., _] => self.errors.push((span, LineError::MuddyDeref)),
                        [] => self.errors.push((span, LineError::EmptyDeref)),
                    }
                    deref = None;
                }
                ';' => {
                    comment = Some(Span::new(pos, line.len() as u32));
                    break;
                }
                _ => self
                    .errors
                    .push((Span::point(pos), LineError::UnknownChar(ch))),
            }
        }
        if let Some((_, p)) = deref {
            self.errors.push((Span::point(p), LineError::UnclosedDeref));
        }
        Line {
            kind,
            errors: (err_start, self.errors.len() as u32),
            literals: (lit_start, self.literals.len() as u32),
            comment,
        }
    }

    fn check_comma(
        &mut self,
        kind: &mut LineKind,
        last_comma: &mut u32,
        lit_start: u32,
        span: Span,
    ) {
        match &self.literals[lit_start as usize..] {
            [(s1, Literal::Ident)] if *last_comma > 0 => {
                if let LineKind::Instruction(s0) = kind {
                    *kind = LineKind::Variable(*s0, *s1);
                }
                self.literals.pop();
            }
            [.., l1] if *last_comma > 0 => {
                self.errors
                    .push((l1.0.between(span), LineError::MissingComma));
            }
            _ => (),
        }
        *last_comma += 1;
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
                Some((_, '"')) | None => break,
                Some((i, _)) => last = i as u32,
            }
        }
        last + 2
    }
}
