use super::{LexLine, LineKind, Literal, Span};

#[derive(Debug, Default)]
pub struct Context<'a> {
    pub line_src: &'a str,
    literals: Vec<(Span, Literal)>,
    kind: LineKind,
    lit_start: u32,
    comma_found: bool,
    bracket: bool,
    last_comma: u32,
    valid_literals: u32,
}

impl Context<'_> {
    pub(super) fn literals(&self) -> &[(Span, Literal)] {
        &self.literals
    }
    pub(super) fn line(&mut self) -> LexLine {
        let line = LexLine {
            kind: self.kind,
            literals: (self.lit_start, self.literals.len() as u32),
            comment: None,
        };
        self.kind = LineKind::Empty;
        self.lit_start = self.literals.len() as u32;
        self.comma_found = false;
        self.bracket = false;
        self.last_comma = 0;
        self.valid_literals = 0;
        line
    }
    pub(super) fn push_lit(&mut self, span: impl Spanned, lit: Literal) {
        use LineKind::*;
        use Literal::*;
        let span = span.spanned();
        if let (Ident, Empty, false) = (lit, self.kind, self.comma_found) {
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
            self.comma_found = true;
            self.last_comma = 0;
            if let (1, Instruction) = (self.valid_literals, self.kind) {
                self.kind = Empty;
            }
        } else if OpenBracket == lit {
            self.bracket = true;
        } else if CloseBracket == lit {
            self.bracket = false;
        } else {
            if self.valid_literals > 1
                && self.last_comma > 1
                && !self.bracket
                && Instruction == self.kind
            {
                self.kind = Variable;
            }
            self.last_comma += 1;
            self.valid_literals += 1;
        }
        self.literals.push((span, lit));
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
    pub fn parts(self) -> Vec<(Span, Literal)> {
        self.literals
    }
}

impl<'a> Context<'a> {
    pub fn reset(&mut self, line: &'a str) {
        self.line_src = line;
        self.kind = LineKind::Empty;
        self.lit_start = self.literals.len() as u32;
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
