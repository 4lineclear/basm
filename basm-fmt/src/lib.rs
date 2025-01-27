use ahash::AHashSet;
use basm::{
    lex::{Advance, Lexeme, Span},
    parse::ParseError,
    Basm, Line,
};

#[cfg(test)]
mod test;

// TODO: create vertical alignment

#[derive(Debug, Default)]
pub struct Edit {
    pub line: u32,
    pub offset: u32,
    /// span excludes offset
    ///
    /// to get the actual byte position, add offset
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
    err: AHashSet<u32>,
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
        if lex.len() < 1 {
            return;
        }
        if self.err.contains(&lex[0].line) {
            return;
        }
        let first = lex[0];
        let eol = lex[lex.len() - 1];
        let Lexeme::Eol(comment) = eol.lex else {
            return;
        };
        if Lexeme::Eof == first.lex || lex.len() < 2 {
            if comment {
                self.in_comment(eol);
            }
            return;
        }
        let line = &self.basm.lines[first.line as usize];
        match line {
            Line::NoOp => self.fmt_noop(lex),
            Line::Label { .. } => self.fmt_label(lex),
            Line::Global { .. } | Line::Variable { .. } => self.fmt_unpadded(lex),
            Line::Instruction { .. } => self.fmt_instruction(lex),
        }
        let slast = lex[lex.len() - 2];

        if Lexeme::Whitespace == slast.lex {
            // no comment or comment on empty line
            if !comment || matches!(line, Line::NoOp) {
                self.out.push(Edit::delete(slast));
            } else if check_space(slast.span.slice(self.src)) {
                self.out.push(Edit::space(slast, 1));
            }
        } else if comment && !matches!(line, Line::NoOp) {
            let mut span = slast.span;
            span.from = span.to;
            self.out.push(Edit::space(Advance { span, ..slast }, 1));
        }
        if comment {
            self.in_comment(eol);
        }
    }

    fn in_comment(&mut self, eol: Advance) {
        let post_semi = Span::new(eol.span.from + 1, eol.span.to - 1);
        let src = post_semi.slice(self.src);
        if check_space(src) {
            let trim_start = src.trim_start();
            if !trim_start.is_empty() {
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

    fn fmt_label(&mut self, lex: &[Advance]) {
        use Lexeme::*;
        let first = lex[0];
        if Whitespace == first.lex {
            self.out.push(Edit::delete(first));
        }
        if lex.len() > 2 {
            self.check_ws_range(lex);
        }
    }

    fn fmt_unpadded(&mut self, lex: &[Advance]) {
        use Lexeme::*;
        let first = lex[0];
        if Whitespace == first.lex {
            self.out.push(Edit::delete(first));
        }
        if lex.len() > 2 {
            self.check_ws_range(lex);
        }
    }

    fn fmt_instruction(&mut self, lex: &[Advance]) {
        use Lexeme::*;
        let first = lex[0];
        if Whitespace != first.lex {
            let mut ad = first;
            ad.span.to = ad.span.from;
            self.out.push(Edit::space(ad, self.fmt.tab_size));
        } else if first.span.len() != self.fmt.tab_size {
            self.out.push(Edit::space(first, self.fmt.tab_size));
        }
        if lex.len() > 2 {
            self.check_ws_range(lex);
        }
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
            } else if check_space(ad.span.slice(self.src)) {
                self.out.push(Edit::space(ad, 1));
            }
        } else if ad.lex == Comma && !matches!(next, Whitespace | Eol(_)) {
            let mut ad = ad;
            ad.span.from = ad.span.to;
            self.out.push(Edit::space(ad, 1));
        }
    }
}

fn check_space(src: &str) -> bool {
    let mut src = src.chars();
    let Some(' ') = src.next() else {
        return true;
    };
    src.next().is_some_and(char::is_whitespace)
}

pub fn fmt(
    basm: &Basm,
    lex: &[Advance],
    src: &str,
    errors: &[ParseError],
    fmt: &FmtContext,
) -> Vec<Edit> {
    // NOTE: consider treating lines with errors as normal lines(ins & var lines)
    let err: AHashSet<u32> = errors.iter().map(|s| s.advance().line).collect();
    Formatter {
        basm,
        lex,
        fmt,
        src,
        err,
        out: Vec::new(),
    }
    .fmt()
    .out
}

// TODO: detect overlapping ranges(which won't work)
pub fn apply_fmt(src: &str) -> String {
    fn create_fmt(src: &str) -> Vec<Edit> {
        use basm::parse::Parser;
        let (basm, errors, lex) = Parser::recorded(&src).parse();
        let mut fmt = fmt(&basm, &lex, &src, &errors, &Default::default());
        fmt.sort_unstable_by_key(|e| (e.line, e.span));
        return fmt;
    }
    let mut out = String::with_capacity(src.len());
    let mut p = 0;
    for mut e in create_fmt(&src) {
        e.span.from += e.offset;
        e.span.to += e.offset;
        out.push_str(&src[p as usize..e.span.from as usize]);
        out.push_str(&e.text);
        p = e.span.to;
    }
    out.push_str(&src[p as usize..]);
    out
}
