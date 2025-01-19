use std::io::{stdin, Read};

fn main() {
    let mut src = String::new();
    stdin()
        .read_to_string(&mut src)
        .expect("failed to read stdin");

    let lo = basm::lex::LexOutput::lex_all(&src);
    let fmt = &Default::default();
    let mut out = String::with_capacity(src.len());
    for (l, al) in lo.lines.iter().enumerate() {
        fmt_line(
            &mut out,
            basm_fmt::LineCtx {
                line: l as u32,
                kind: al.line.kind,
                comment: al.line.comment,
                line_src: al.line_src(lo.src.as_ref()),
                errors: al.line.slice_err(&lo.errors),
                literals: al.line.slice_lit(&lo.literals),
                fmt,
            },
        );
    }
    println!("{out}");
}

fn fmt_line(out: &mut String, ctx: basm_fmt::LineCtx<'_>) {
    use basm::lex::LineKind::*;
    let basm_fmt::LineCtx {
        kind,
        comment,
        line_src,
        errors,
        literals,
        fmt,
        ..
    } = ctx;
    if errors.len() != 0 {
        out.push_str(line_src);
        out.push('\n');
        return;
    }
    let spaces = " ".repeat(fmt.tab_size as usize);
    match kind {
        Empty => (),
        Label(span) => {
            out.push_str(span.slice(line_src));
        }
        Section(span, span1) => {
            out.push_str(span.slice(line_src));
            out.push_str(" ");
            out.push_str(span1.slice(line_src));
        }
        Instruction(span) => {
            out.push_str(&spaces);
            out.push_str(span.slice(line_src));
        }
        Variable(span, span1) => {
            out.push_str(&spaces);
            out.push_str(span.slice(line_src));
            out.push_str(" ");
            out.push_str(span1.slice(line_src));
        }
    };
    if literals.len() != 0 {
        if !matches!(kind, Empty) {
            out.push_str(" ");
        }
        out.push_str(literals[0].0.slice(line_src));
        for (span, _) in &literals[1..] {
            out.push_str(", ");
            out.push_str(span.slice(line_src));
        }
    }
    if let Some(span) = comment {
        if !out.ends_with('\n') {
            out.push_str(" ");
        }
        out.push_str(";");
        let trim = &span.slice(line_src)[1..].trim_start();
        if trim.len() != 0 {
            out.push_str(" ");
        }
        out.push_str(trim);
    }
    while out.ends_with(|c| matches!(c, ' ' | '\t')) {
        out.pop();
    }
    out.push('\n');
}
