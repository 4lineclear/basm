use basm::lex::{LexOutput, LineKind, Literal, Span};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Edit {
    pub line: u32,
    pub span: Span,
    pub change: String,
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

pub fn fmt<'a, S: AsRef<str>>(
    lo: &'a LexOutput<S>,
    fmt: &'a FmtContext,
) -> impl Iterator<Item = Edit> + 'a {
    lo.lines.iter().enumerate().flat_map(|(l, al)| {
        fmt_line(LineCtx {
            line: l as u32,
            kind: al.line.kind,
            comment: al.line.comment,
            line_src: al.line_src(lo.src.as_ref()),
            literals: al.line.slice_lit(&lo.literals),
            fmt,
        })
    })
}

// TODO: remove excess spaces

#[derive(Debug, Clone, Copy)]
pub struct LineCtx<'a> {
    pub line: u32,
    pub kind: LineKind,
    pub comment: Option<Span>,
    pub line_src: &'a str,
    pub literals: &'a [(Span, Literal)],
    pub fmt: &'a FmtContext,
}

pub fn fmt_line(ctx: LineCtx<'_>) -> impl Iterator<Item = Edit> + '_ {
    let LineCtx {
        line,
        // kind,
        comment,
        line_src,
        // errors,
        literals,
        // fmt,
        ..
    } = ctx;
    let edit = |span, change| Edit { line, span, change };
    let delete_change = move |span| edit(span, "".to_owned());

    let (cts, cte) = comment
        .map(|comment| {
            let post_semi = Span::new(comment.from + 1, comment.to);
            let src = post_semi.slice(line_src);
            let trim_end = src.trim_end();
            let trim_end = (trim_end.len() != src.len()).then(|| {
                let mut span = post_semi;
                span.from += trim_end.len() as u32;
                delete_change(span)
            });
            let trim_start = src.trim_start();
            let trim_start =
                (trim_start.len() + 1 != src.len() && trim_start.len() != 0).then(|| {
                    let mut span = post_semi;
                    span.to -= trim_start.len() as u32;
                    edit(span, " ".into())
                });
            (trim_start, trim_end)
        })
        .unwrap_or((None, None));
    let mut valid_literals = 0;
    literals
        .iter()
        .enumerate()
        .filter_map(move |(i, &(span, lit))| fmt_literals(i, span, lit, &mut valid_literals, ctx))
        .chain([cts, cte].into_iter().flatten())
}

fn fmt_literals(
    i: usize,
    span: Span,
    lit: Literal,
    valid_literals: &mut u32,
    ctx: LineCtx<'_>,
) -> Option<Edit> {
    use LineKind::*;
    use Literal::*;
    let LineCtx {
        line,
        kind,
        comment,
        line_src,
        literals,
        fmt,
    } = ctx;
    let edit = |span, change| Edit { line, span, change };
    let delete_change = move |span| edit(span, "".to_owned());

    *valid_literals += (!matches!(lit, Whitespace | Comma | Colon | Other)) as u32;
    match lit {
        Whitespace if comment.is_none() && literals.len() == i + 1 => Some(delete_change(span)),
        Whitespace => match (kind, valid_literals) {
            (Empty | Label | Section, 0) => Some(delete_change(span)),
            (Global | Instruction | Variable, 0) if span.len() != fmt.tab_size => {
                Some(edit(span, " ".repeat(fmt.tab_size as usize)))
            }
            // TODO: consider move to own function
            (_, n) => {
                if let Some((_, Comma | CloseBracket)) = literals.get(i + 1) {
                    Some(edit(span, "".into()))
                } else if let Some((_, OpenBracket)) = literals.get(i.saturating_sub(1)) {
                    Some(edit(span, "".into()))
                } else if *n > 0 && span.len() > 1 {
                    Some(edit(span, " ".into()))
                } else {
                    span.slice(line_src)
                        .chars()
                        .any(|c| c.is_whitespace() && c != ' ')
                        .then(|| edit(span, " ".into()))
                }
            }
        },
        Ident => match (kind, *valid_literals - 1) {
            (Empty, 0) => None,
            (Label, 0) => None,
            (Section, 0) => None,
            (Global, 0) | (Instruction, 0) | (Variable, 0) if span.from == 0 => {
                Some(edit(Span::new(0, 0), " ".repeat(fmt.tab_size as usize)))
            }
            _ => None,
        },
        OpenBracket | CloseBracket => None,
        String | Binary | Octal | Decimal | Hex => None,
        Comma => line_src[span.to as usize..]
            .chars()
            .next()
            .and_then(|ch| (ch != ' ').then(|| edit(Span::new(span.to, span.to), " ".into()))),
        Colon => None,
        Other => None,
    }
}
