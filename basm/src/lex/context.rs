use super::{Line, LineError, LineKind, Literal, Span};

#[derive(Debug, Default)]
pub struct Context<'a> {
    pub line_src: &'a str,
    errors: Vec<(Span, LineError)>,
    literals: Vec<(Span, Literal)>,
    kind: LineKind,
    lit_start: u32,
    err_start: u32,
    muddled: bool,
    last_comma: u32,
    valid_literals: u32,
}

impl Context<'_> {
    pub fn line(&mut self) -> Line {
        let line = Line {
            kind: self.kind,
            errors: (self.err_start, self.errors.len() as u32),
            literals: (self.lit_start, self.literals.len() as u32),
            comment: None,
        };
        self.kind = LineKind::Empty;
        self.lit_start = self.literals.len() as u32;
        self.err_start = self.errors.len() as u32;
        self.muddled = false;
        self.last_comma = 0;
        self.valid_literals = 0;
        line
    }
    pub fn push_lit(&mut self, span: impl Spanned, lit: Literal) {
        use LineKind::*;
        use Literal::*;
        let span = span.spanned();
        // TODO: consider removing global & section
        if let (Ident, Empty, false) = (lit, self.kind, self.muddled) {
            self.kind = match span.slice(self.line_src) {
                "section" => Section,
                "global" => Global,
                _ => Instruction,
            };
        }
        if let (Colon, Instruction) = (lit, self.kind) {
            self.kind = Label;
        }
        if let Other | Whitespace = lit {
            self.combined(span, lit);
            return;
        }
        if Comma == lit {
            self.last_comma = 0;
            if let (1, Instruction) = (self.valid_literals, self.kind) {
                self.kind = Empty;
            }
        } else {
            if self.valid_literals > 2 {
                if self.last_comma > 0 {
                    let span = self.literals.last().map(|(s, _)| *s).unwrap_or(span);
                    self.errors.push((span, LineError::MissingComma));
                }
                if Instruction == self.kind {
                    self.kind = Variable;
                }
            }
            self.last_comma += 1;
            self.valid_literals += 1;
        }
        self.literals.push((span, lit));
        self.muddled |= lit == Comma;
    }
    fn combined(&mut self, span: Span, lit: Literal) {
        if let Some((orig, orig_lit)) = self.literals.last_mut() {
            if *orig_lit == lit && orig.to == span.from {
                orig.to = span.to;
                return;
            }
        }
        self.literals.push((span, lit));
    }
    // fn err(&mut self, span: impl Spanned, err: LineError) {
    //     let span = span.spanned();
    //     self.errors.push((span, err));
    // }
    // fn current_literals(&self) -> &[(Span, Literal)] {
    //     &self.literals[self.lit_start as usize..]
    // }
    // fn literal_count(&self) -> u32 {
    //     self.literals.len() as u32 - self.lit_start
    // }

    pub fn parts(self) -> (Vec<(Span, Literal)>, Vec<(Span, LineError)>) {
        (self.literals, self.errors)
    }
}

impl<'a> Context<'a> {
    // pub fn new(src: &'a str) -> Self {
    //     Self {
    //         src,
    //         ..Self::default()
    //     }
    // }
    pub fn reset(&mut self, line: &'a str) {
        self.line_src = line;
        self.kind = LineKind::Empty;
        self.lit_start = self.literals.len() as u32;
        self.err_start = self.errors.len() as u32;
    }
}

pub trait Spanned {
    fn spanned(self) -> Span;
}
impl Spanned for Span {
    fn spanned(self) -> Span {
        self
    }
}
impl Spanned for u32 {
    fn spanned(self) -> Span {
        Span::point(self)
    }
}
impl Spanned for (u32, u32) {
    fn spanned(self) -> Span {
        Span::new(self.0, self.1)
    }
}
