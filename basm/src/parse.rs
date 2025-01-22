use crate::{
    lex::{
        Advance,
        Lexeme::{self, *},
        Lexer, Span,
    },
    Basm, Line, Section, SectionKind, Value,
};

#[cfg(test)]
mod test;

// TODO: improve spans

// TODO: replace breaks with error propagation

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    basm: Basm<'a>,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            lexer: Lexer::new(src),
            basm: Basm::new(src),
            errors: Vec::new(),
        }
    }
    pub fn parse(mut self) -> (Basm<'a>, Vec<ParseError>) {
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
        (self.basm, self.errors)
    }

    fn parse_line(&mut self, first: Advance) -> ParseResult<()> {
        let second = self.peek_non_ws();
        if let Colon = second.lex {
            self.lexer.pop_peek();
            self.clear_line()?;
            let address = self.current_address();
            let key = self.slice(first.span);
            if self.basm.labels.contains_key(key) {
                return Err(ParseErrorKind::DuplicateLabel(key.to_owned(), address).full(first));
            } else {
                self.basm.labels.insert(key, address);
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
            Eol(_) | Eof => return Err(ParseErrorKind::InputEnd(ad.line).full(ad)),
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
                    .map_err(|e| ParseErrorKind::ParseIntError(ad.span, e).full(ad))?;
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
        ParseErrorKind::Expected(ad, expected.into()).full(ad)
    }

    fn slice(&self, span: impl Into<Span>) -> &'a str {
        span.into().slice(self.basm.src)
    }

    fn after_bracket(&mut self) -> ParseResult<Span> {
        let ident = self.non_ws();
        match ident.lex {
            Ident => (),
            Eol(_) | Eof => return Err(ParseErrorKind::InputEnd(ident.line).full(ident)),
            _ => {
                self.kill_line();
                return Err(self.expected(ident, "Ident"));
            }
        }
        let close = self.non_ws();
        match close.lex {
            CloseBracket => (),
            Eol(_) | Eof => return Err(ParseErrorKind::InputEnd(close.line).full(close)),
            _ => {
                self.kill_line();
                return Err(self.expected(close, "CloseBracket"));
            }
        }
        Ok(ident.span)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    line: u32,
    offset: u32,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Expected(Advance, String),
    // NoOpValues,
    ParseIntError(Span, std::num::ParseIntError),
    // MissingComma(Span),
    // InvalidValue(Value),
    // Terminated,
    InputEnd(u32),
    DuplicateLabel(String, u16),
}

impl ParseErrorKind {
    fn full(self, ad: Advance) -> ParseError {
        ParseError {
            line: ad.line,
            offset: ad.offset,
            kind: self,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseErrorKind::*;
        let line = self.line;
        let offset = self.offset;
        match &self.kind {
            Expected(
                Advance {
                    lex,
                    line,
                    offset,
                    span,
                },
                expected,
            ) => writeln!(
                f,
                "unexpected input found at {line}:{},{}. \
                expected {expected} but found {lex:?}",
                span.from - offset,
                span.to - offset,
            ),
            InputEnd(line) => writeln!(f, "unexpected input end on line {line}"),
            DuplicateLabel(name, address) => {
                writeln!(f, "duplicate label '{name}' found at address {address}")
            }
            ParseIntError(span, parse_int_error) => writeln!(
                f,
                "unable to parse integer at {line}:{},{}. error: {parse_int_error}",
                span.from - offset,
                span.to - offset,
            ),
        }
    }
}

impl std::error::Error for ParseError {}
