use basm::lex::{LexOutput, LineError, LineKind, Literal, Span};

// TODO: consider rewriting the below to be less insane

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
            errors: al.line.slice_err(&lo.errors),
            literals: al.line.slice_lit(&lo.literals),
            fmt,
        })
    })
}

// TODO: remove excess spaces

pub struct LineCtx<'a> {
    pub line: u32,
    pub kind: LineKind,
    pub comment: Option<Span>,
    pub line_src: &'a str,
    pub errors: &'a [(Span, LineError)],
    pub literals: &'a [(Span, Literal)],
    pub fmt: &'a FmtContext,
}

pub fn fmt_line(ctx: LineCtx<'_>) -> impl Iterator<Item = Edit> + '_ {
    let LineCtx {
        line,
        kind,
        comment,
        line_src,
        errors,
        literals,
        fmt,
    } = ctx;
    let edit = move |(span, change)| Edit { line, span, change };
    let replace_none = move |span| edit((span, String::new()));

    let trim_start = match kind {
        // no need to trim the start of the entire thing is ws
        LineKind::Empty if errors.is_empty() && literals.is_empty() && comment.is_none() => false,
        LineKind::Empty | LineKind::Label | LineKind::Section => true,
        LineKind::Instruction | LineKind::Variable | LineKind::Global => false,
    };
    let (start, end) = try_trim(line_src, trim_start);
    let pad_start = if let LineKind::Instruction | LineKind::Variable = kind {
        try_pad(line_src, fmt.tab_size).map(edit)
    } else {
        None
    };
    let comment = fmt_comment(comment, line, line_src, fmt.tab_size);
    [end.map(replace_none), start.map(replace_none), pad_start]
        .into_iter()
        .flatten()
        .chain(fmt_literals(literals, line_src).map(edit))
        .chain(comment.into_iter().flatten())
}

// TODO: format so when the first comment in a chain rests on a multiple of 4
// the following comments will be aligned to that column.
// maybe also: leading comment must be on a column of larger
// than the length of  the longest line in that chain.
// could also create an in-text flag, sucha as another semicolon at the end of
// the comment
fn fmt_comment(comment: Option<Span>, line: u32, src: &str, tab_size: u32) -> [Option<Edit>; 2] {
    let Some(c) = comment else {
        return [None, None];
    };
    let (diff, spaces) = count_ws(src[..c.from as usize].chars().rev(), tab_size);
    let ws_start = c.from - diff;
    let pre = (spaces != 1 && ws_start as usize != 0).then(|| {
        let span = Span::new(ws_start, c.from);
        let change = " ".to_owned();
        Edit { line, span, change }
    });
    let (diff, spaces) = count_ws(src[c.from as usize + 1..].chars(), tab_size);
    let ws_end = c.from + 1 + diff;
    let post = (spaces != 1 && ws_end as usize != src.len()).then(|| {
        let span = Span::new(c.from + 1, ws_end);
        let change = " ".to_owned();
        Edit { line, span, change }
    });
    [pre, post]
}

fn fmt_literals<'a>(
    literals: &'a [(Span, Literal)],
    src: &'a str,
) -> impl Iterator<Item = (Span, String)> + 'a {
    literals.iter().flat_map(|(s, lit)| {
        let Literal::Deref = lit else {
            return None;
        };
        let src = s.slice(src).trim_start_matches('[').trim_end_matches(']');
        let new_text = src.trim();
        if new_text.len() == src.len() {
            return None;
        }
        Some((Span::new(s.from + 1, s.to - 1), new_text.to_owned()))
    })
}

fn try_trim(src: &str, s: bool) -> (Option<Span>, Option<Span>) {
    let s = s.then_some(()).and_then(|()| {
        let trim = src.trim_start();
        let diff = src.len() as u32 - trim.len() as u32;
        (diff != 0).then(|| Span::new(0, diff))
    });
    let e = {
        let trim = src.trim_end();
        (trim.len() != src.len()).then(|| Span::new(trim.len() as u32, src.len() as u32))
    };
    (s, e)
}

// TODO: normalize whitespace:
// either tab-only or space-only
fn try_pad(src: &str, tab_size: u32) -> Option<(Span, String)> {
    let (to, spaces) = count_ws(src.chars(), tab_size);
    (spaces != tab_size).then(|| (Span { from: 0, to }, " ".repeat(tab_size as usize)))
}

/// returns (byte diff, space diff)
///
/// space diff includes the difference made by `tab_size`
fn count_ws(chars: impl Iterator<Item = char>, tab_size: u32) -> (u32, u32) {
    chars
        .map_while(|c| match c {
            ' ' => Some(1),
            '\t' => Some(tab_size),
            _ => None,
        })
        .fold((0, 0), |(a, b), n| (a + 1, b + n))
}
