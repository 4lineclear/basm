// TODO: move to using a hand-written parser.

use std::iter::Peekable;
use std::str::{CharIndices, Lines};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Line {
    pub kind: LineKind,
    pub literals: (u32, u32),
    pub comment: Option<u32>,
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
            self.line_from, self.from, self.line_to, self.to
        )
    }
}

impl Span {
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

#[derive(Debug)]
pub enum LineKind {
    Empty,
    Label(Span),
    Section(Span),
    Instruction(Span),
    Variable(Span, Span),
    Extra(Vec<Span>),
}

#[derive(Debug)]
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
    UnknownChar(char),
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
        enum Mode {
            Zero,
            One(Span),
            Two(Span, Span),
            Section(Span),
        }
        let line = self.lines.next()?;
        let mut chars = line.char_indices().peekable();
        let mut post_comma = false;
        let mut comment = None;
        let mut mode = Mode::Zero;

        let lit_start = self.literals.len() as u32;

        self.line_starts
            .push(self.line_starts.last().copied().unwrap_or(0) + line.len() as u32 + 1);

        while let Some((pos, ch)) = chars.next() {
            let pos = pos as u32;
            match ch {
                ' ' | '\t' => (),
                'a'..='z' | 'A'..='Z' | '_' => {
                    let end = self.ident(&mut chars);
                    let span = Span::inline(self.line, pos, end);
                    if post_comma {
                        self.literals.push((span, Literal::Ident));
                    } else {
                        mode = match mode {
                            Mode::Zero => Mode::One(span),
                            Mode::One(s0) if self.slice(s0) == "section" => Mode::Section(span),
                            Mode::One(s0) => Mode::Two(s0, span),
                            Mode::Two(s0, s1) => {
                                self.literals.push((span, Literal::Ident));
                                Mode::Two(s0, s1)
                            }
                            Mode::Section(s0) => {
                                self.literals.push((span, Literal::Ident));
                                Mode::Section(s0)
                            }
                        };
                    }
                }
                ',' => {
                    if post_comma {
                        continue;
                    }
                    if let &Mode::Two(s0, s1) = &mode {
                        self.literals.push((s1, Literal::Ident));
                        mode = Mode::One(s0);
                    } else {
                        self.errors
                            .push((Span::point(self.line, pos), LineError::UnknownChar(ch)))
                    }
                    post_comma = true
                }
                '0'..='9' => {
                    if ch == '0' {
                    } else {
                    }
                }
                ';' => {
                    comment = Some(pos);
                    break;
                }
                ch => self
                    .errors
                    .push((Span::point(self.line, pos), LineError::UnknownChar(ch))),
            }
        }
        self.line += 1;
        Some(Line {
            kind: match mode {
                Mode::Zero => LineKind::Empty,
                Mode::One(s0) => LineKind::Instruction(s0),
                Mode::Two(s0, s1) => LineKind::Variable(s0, s1),
                Mode::Section(s0) => LineKind::Section(s0),
            },
            literals: (lit_start, self.literals.len() as u32),
            comment,
        })
    }

    fn slice(&self, span: Span) -> &str {
        let f = self.line_starts[span.line_from as usize] + span.from;
        let t = self.line_starts[span.line_to as usize - 1] + span.to;
        dbg!(&self.src()[f as usize..t as usize])
    }

    fn ident(&mut self, chars: &mut Charred) -> u32 {
        let mut last = 0;
        while let Some((i, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')) = chars.peek().copied() {
            last = i as u32;
            chars.next();
        }
        last + 1
    }
}
