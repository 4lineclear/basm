use std::usize;

use basm::{
    lex::{Advance, Lexeme, Span},
    Basm, Line,
};

#[cfg(test)]
mod test;

// TODO: consider changing this to two advances

#[derive(Debug, Default)]
pub struct Edit {
    pub line: u32,
    pub offset: u32,
    pub span: Span,
    pub text: String,
}

impl Edit {
    fn new(line: u32, offset: u32, span: Span, text: impl Into<String>) -> Self {
        Self {
            line,
            offset,
            span: (span.from - offset, span.to - offset).into(),
            text: text.into(),
        }
    }
    fn ad(
        Advance {
            line, offset, span, ..
        }: Advance,
        text: impl Into<String>,
    ) -> Self {
        Self::new(line, offset, span, text)
    }
    fn delete(ad: Advance) -> Self {
        Self::ad(ad, "")
    }
    fn space(ad: Advance, n: u32) -> Self {
        Self::ad(ad, " ".repeat(n as usize))
    }
    #[allow(unused)]
    fn replace(ad: Advance, text: impl Into<String>) -> Self {
        Self::ad(ad, text)
    }
}

#[derive(Debug)]
pub struct FmtContext {
    /// default = `4`
    pub tab_size: u32,
}

impl Default for FmtContext {
    fn default() -> Self {
        Self { tab_size: 4 }
    }
}

struct Formatter<'a> {
    basm: &'a Basm,
    lex: &'a [Advance],
    fmt: &'a FmtContext,
    src: &'a str,
    out: Vec<Edit>,
}

impl Formatter<'_> {
    fn fmt(mut self) -> Self {
        self.lex
            .split_inclusive(|f| matches!(f.lex, Lexeme::Eol(_)))
            .for_each(|a| self.fmt_line(a));
        self
    }

    fn fmt_line(&mut self, lex: &[Advance]) {
        if lex.len() < 2 {
            return;
        }
        let first = lex[0];
        let eol = lex[lex.len() - 1];
        if Lexeme::Eof == first.lex {
            return;
        }
        let Lexeme::Eol(comment) = eol.lex else {
            return;
        };
        let line = &self.basm.lines[first.line as usize];
        match line {
            Line::NoOp => self.fmt_noop(lex),
            Line::Section { .. } | Line::Label { .. } => self.fmt_kw(lex),
            Line::Instruction { .. } | Line::Variable { .. } => self.fmt_norm(lex),
        }
        let slast = lex[lex.len() - 2];

        if Lexeme::Whitespace == slast.lex {
            // no comment or comment on empty line
            if !comment || matches!(line, Line::NoOp) {
                self.out.push(Edit::delete(slast));
            } else if check_space(slast.span.slice(self.src).chars()) {
                self.out.push(Edit::space(slast, 1));
            }
        }
        if comment {
            self.comment(eol);
        }
    }

    fn comment(&mut self, eol: Advance) {
        let post_semi = Span::new(eol.span.from + 1, eol.span.to - 1);
        let src = post_semi.slice(self.src);
        if check_space(src.chars()) {
            let trim_start = src.trim_start();
            if trim_start.len() != 0 {
                let mut span = post_semi;
                span.to -= trim_start.len() as u32;
                self.out.push(Edit::space(Advance { span, ..eol }, 1));
            }
        }
        let trim_end = src.trim_end();
        if trim_end.len() != src.len() {
            let mut span = post_semi;
            span.from += trim_end.len() as u32;
            self.out.push(Edit::delete(Advance { span, ..eol }));
        }
    }
    fn fmt_noop(&mut self, lex: &[Advance]) {
        use Lexeme::*;
        let first = lex[0];
        if Whitespace == first.lex && lex.len() != 2 {
            self.out.push(Edit::delete(first));
        }
    }

    fn fmt_kw(&mut self, lex: &[Advance]) {
        use Lexeme::*;
        let first = lex[0];
        if Whitespace == first.lex {
            self.out.push(Edit::delete(first));
        }
        self.check_ws_range(lex);
    }

    fn fmt_norm(&mut self, lex: &[Advance]) {
        use Lexeme::*;
        let first = lex[0];
        if Whitespace != first.lex {
            let mut ad = first;
            ad.span.to = ad.span.from;
            self.out.push(Edit::space(ad, self.fmt.tab_size));
        } else if first.span.len() != self.fmt.tab_size {
            self.out.push(Edit::space(first, self.fmt.tab_size));
        }
        if lex.len() < 3 {
            return;
        }
        self.check_ws_range(lex);
    }

    fn check_ws_range(&mut self, lex: &[Advance]) {
        for (i, &ad) in lex[1..lex.len() - 2].iter().enumerate() {
            let i = i + 1; // since starting at index 1
            self.check_ws(ad, lex[i + 1].lex, lex[i - 1].lex);
        }
    }

    fn check_ws(&mut self, ad: Advance, next: Lexeme, prev: Lexeme) {
        use Lexeme::*;
        if ad.lex == Whitespace {
            if matches!(next, Comma | Colon | CloseBracket) || matches!(prev, OpenBracket) {
                self.out.push(Edit::delete(ad));
            } else if check_space(ad.span.slice(self.src).chars()) {
                self.out.push(Edit::space(ad, 1));
            }
        } else if ad.lex == Comma && !matches!(next, Whitespace | Eol(_)) {
            let mut ad = ad;
            ad.span.from = ad.span.to;
            self.out.push(Edit::space(ad, 1));
        }
    }
}

fn check_space(mut ch: impl Iterator<Item = char>) -> bool {
    let Some(' ') = ch.next() else {
        return true;
    };
    ch.next().is_some_and(char::is_whitespace)
}

pub fn fmt<'a>(basm: &Basm, lex: &[Advance], src: &str, fmt: &FmtContext) -> Vec<Edit> {
    Formatter {
        basm,
        lex,
        fmt,
        src,
        out: Vec::new(),
    }
    .fmt()
    .out
}
