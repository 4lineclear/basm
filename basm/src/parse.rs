use crate::{
    lex::{
        Advance, BaseLexer,
        Lexeme::{self, *},
        Lexer, RecordedLexer, Span,
    },
    Basm, Line, Section, SectionKind, Value,
};

#[cfg(test)]
mod test;

// TODO: improve spans

// TODO: replace breaks with error propagation

#[derive(Debug)]
pub struct Parser<L, S> {
    lexer: L,
    basm: Basm<S>,
    errors: Vec<ParseError>,
}

impl<'a> Parser<BaseLexer<'a>, &'a str> {
    pub fn base(src: &'a str) -> Self {
        Self {
            lexer: BaseLexer::new(src),
            basm: Basm::new(src),
            errors: Vec::new(),
        }
    }
    pub fn parse(mut self) -> (Basm<&'a str>, Vec<ParseError>) {
        self.parse_inner();
        (self.basm, self.errors)
    }
}
impl<'a> Parser<RecordedLexer<'a>, &'a str> {
    pub fn recorded(src: &'a str) -> Self {
        Self {
            lexer: RecordedLexer::new(src),
            basm: Basm::new(src),
            errors: Vec::new(),
        }
    }
    pub fn parse(mut self) -> (Basm<&'a str>, Vec<ParseError>, Vec<Advance>) {
        self.parse_inner();
        (self.basm, self.errors, self.lexer.parts().1)
    }
}

impl<'a, L: Lexer> Parser<L, &'a str> {
    pub fn lexer(&self) -> &L {
        &self.lexer
    }
    fn parse_inner(&mut self) {
        loop {
            let ad = self.lexer.advance();
            let e = match ad.lex {
                Whitespace => Ok(()),
                Ident => match self.slice(ad.span) {
                    "section" => self.section(),
                    _ => self.parse_line(ad),
                },
                Eol(_) => {
                    self.basm.lines.push(Line::NoOp);
                    Ok(())
                }
                Eof => break,
                _ => Err(self.expected(ad, "Ident | Eol | Eof")),
            };
            if let Err(e) = e {
                self.errors.push(e);
            }
        }
    }

    fn parse_line(&mut self, first: Advance) -> ParseResult<()> {
        let second = self.peek_non_ws();
        if let Colon = second.lex {
            self.lexer.pop_peek();
            self.clear_line()?;
            let address = self.current_address();
            let key = self.slice(first.span);
            let k = self.basm.si.get_or_intern(key);
            if self.basm.labels.contains_key(&k) {
                return Err(ParseErrorKind::DuplicateLabel(key.to_owned(), address).full(first));
            } else {
                self.basm.labels.insert(k, address);
            }
            return Ok(());
        }
        let Some(value) = self.value()? else {
            self.basm.lines.push(Line::Instruction {
                ins: first.span,
                values: vec![],
            });
            return Ok(());
        };
        let (values, ins) = self.ins_or_var(value)?;
        self.basm.lines.push(if ins {
            Line::Instruction {
                ins: first.span,
                values,
            }
        } else {
            Line::Variable {
                name: first.span,
                r#type: second.span,
                values,
            }
        });
        Ok(())
    }

    fn section(&mut self) -> ParseResult<()> {
        let ad = self.non_ws();
        match ad.lex {
            Ident => (),
            Eol(_) | Eof => return Err(ParseErrorKind::InputEnd.full(ad)),
            _ => {
                return Err(self.expected(ad, "Ident"));
            }
        }
        let kind = match self.slice(ad.span) {
            "bss" => SectionKind::Bss,
            "data" => SectionKind::Data,
            "text" => SectionKind::Text,
            _ => {
                return Err(self.expected(ad, "'bss' | 'data' | 'text'"));
            }
        };
        self.clear_line()?;
        let address = self.current_address();
        self.basm.sections.push(Section { kind, address });
        Ok(())
    }

    fn ins_or_var(&mut self, second: Value) -> ParseResult<(Vec<Value>, bool)> {
        if !matches!(second, Value::Ident(_)) {
            return Ok((self.values(second)?, true));
        }
        if let Comma | Eol(_) | Eof = self.peek_non_ws().lex {
            return Ok((self.values(second)?, true));
        }
        if let Some(value) = self.value()? {
            Ok((self.values(value)?, false))
        } else {
            Ok((self.values(second)?, true))
        }
    }

    fn values(&mut self, first: Value) -> ParseResult<Vec<Value>> {
        let mut values = vec![first];
        loop {
            let ad = self.non_ws();
            match ad.lex {
                Comma => (),
                Eol(_) | Eof => break Ok(values),
                _ => {
                    break Err(self.expected(ad, "Comma"));
                }
            }
            let Some(value) = self.value()? else {
                break Ok(values);
            };
            values.push(value);
        }
    }

    fn value(&mut self) -> ParseResult<Option<Value>> {
        let ad = self.non_ws();
        match ad.lex {
            Eol(_) | Eof => Ok(None),
            Ident => Ok(Some(Value::Ident(ad.span))),
            Str => Ok(Some(Value::String(
                self.slice((ad.span.from + 1, ad.span.to - 1)).to_owned(),
            ))),
            Digit(base) => {
                let n = u32::from_str_radix(self.slice(ad.span), base as u32)
                    .map_err(|e| ParseErrorKind::ParseIntError(e).full(ad))?;
                Ok(Some(Value::Digit(base, n)))
            }
            OpenBracket => Ok(Some(Value::Deref(self.after_bracket()?))),
            _ => Err(self.expected(ad, "Ident | Str | Colon | OpenBracket | Digit")),
        }
    }

    fn non_ws(&mut self) -> Advance {
        while let Lexeme::Whitespace = self.lexer.peek().lex {
            self.lexer.pop_peek();
        }
        self.lexer.advance()
    }
    fn peek_non_ws(&mut self) -> Advance {
        while let Lexeme::Whitespace = self.lexer.peek().lex {
            self.lexer.pop_peek();
        }
        self.lexer.peek()
    }

    #[allow(unused)]
    fn advance_if(&mut self, go: impl Fn(Advance) -> bool) {
        let peek = self.peek_non_ws();
        if go(peek) {
            self.lexer.pop_peek();
        }
    }

    fn clear_line(&mut self) -> ParseResult<()> {
        let ad = self.non_ws();
        if let Eol(_) | Eof = ad.lex {
            Ok(())
        } else {
            Err(self.expected(ad, "Whitespace"))
        }
    }

    fn kill_line(&mut self) {
        while !matches!(self.lexer.advance().lex, Eol(_) | Eof) {}
    }

    fn current_address(&mut self) -> u16 {
        u16::try_from(self.basm.lines.len()).expect("failed to convert line address to u16")
    }

    fn expected(&mut self, ad: Advance, expected: impl Into<String>) -> ParseError {
        self.kill_line();
        ParseErrorKind::Expected(expected.into()).full(ad)
    }

    fn slice(&self, span: impl Into<Span>) -> &'a str {
        span.into().slice(self.basm.src)
    }

    fn after_bracket(&mut self) -> ParseResult<Span> {
        let ident = self.non_ws();
        match ident.lex {
            Ident => (),
            Eol(_) | Eof => return Err(ParseErrorKind::InputEnd.full(ident)),
            _ => {
                self.kill_line();
                return Err(self.expected(ident, "Ident"));
            }
        }
        let close = self.non_ws();
        match close.lex {
            CloseBracket => (),
            Eol(_) | Eof => return Err(ParseErrorKind::InputEnd.full(close)),
            _ => {
                self.kill_line();
                return Err(self.expected(close, "CloseBracket"));
            }
        }
        Ok(ident.span)
    }
}

// TODO: improve errors.
// make them usable in the lsp

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    ad: Advance,
    kind: ParseErrorKind,
}

impl ParseError {
    pub fn advance(&self) -> Advance {
        self.ad
    }
    pub fn line(&self) -> u32 {
        self.ad.line
    }

    pub fn offset(&self) -> u32 {
        self.ad.offset
    }

    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Expected(String),
    ParseIntError(std::num::ParseIntError),
    InputEnd,
    DuplicateLabel(String, u16),
}

impl ParseErrorKind {
    fn full(self, ad: Advance) -> ParseError {
        ParseError { ad, kind: self }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseErrorKind::*;
        let ad = self.ad;
        let offset = ad.offset;
        let line = ad.line;
        let span = ad.span;
        let from = span.from - offset;
        let to = span.to - offset;
        match &self.kind {
            Expected(e) => {
                writeln!(
                    f,
                    "unexpected input found at: {line}:{from}:{to}. expected {e} but got {:?}",
                    ad.lex
                )
            }
            ParseIntError(_) => {
                writeln!(f, "unable to parse number at: {line}:{from}:{to}")
            }
            InputEnd => writeln!(f, "input ended early at: {line}:{from}:{to}"),
            DuplicateLabel(_, _) => writeln!(f, "duplicate label found at: {line}:{from}:{to}"),
        }
        // writeln!(f, "{:?}", self.ad)
    }
}

impl std::error::Error for ParseError {}
