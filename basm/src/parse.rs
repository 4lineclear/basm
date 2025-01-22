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
            match ad.lex {
                Whitespace => (),
                Ident => match self.slice(ad.span) {
                    "section" => self.section(),
                    _ => self.post_ident(ad),
                },
                Eol(_) => {
                    self.basm.lines.push(Line::NoOp);
                }
                Eof => break,
                _ => {
                    self.expected(ad, "Ident | Eol | Eof");
                    self.kill_line();
                }
            }
        }
        (self.basm, self.errors)
    }

    fn post_ident(&mut self, ad: Advance) {
        let ad2 = self.non_ws();
        match ad2.lex {
            Ident => {
                let Some((values, ins)) = self.values(ad2.span) else {
                    return;
                };
                self.basm.lines.push(if ins || values.is_empty() {
                    Line::Instruction {
                        ins: ad.span,
                        values,
                    }
                } else {
                    Line::Variable {
                        name: ad.span,
                        r#type: ad2.span,
                        values,
                    }
                });
            }
            Eol(_) | Eof => self.basm.lines.push(Line::Instruction {
                ins: ad.span,
                values: vec![],
            }),
            Colon if self.clear_line() => {
                let address = self.current_address();
                let key = self.slice(ad.span);
                if self.basm.labels.contains_key(key) {
                    self.errors
                        .push(ParseErrorKind::DuplicateLabel(key.to_owned(), address).full(ad));
                } else {
                    self.basm.labels.insert(key, address);
                }
            }
            Colon => (),
            _ => {
                self.expected(ad2, "Ident | Colon");
                self.kill_line();
            }
        }
    }

    fn section(&mut self) {
        let ad = self.non_ws();
        match ad.lex {
            Ident => (),
            Eol(_) | Eof => {
                self.errors.push(ParseErrorKind::InputEnd(ad.line).full(ad));
                return;
            }
            _ => {
                self.expected(ad, "Ident");
                self.kill_line();
                return;
            }
        }
        let kind = match self.slice(ad.span) {
            "bss" => SectionKind::Bss,
            "data" => SectionKind::Data,
            "text" => SectionKind::Text,
            _ => {
                self.expected(ad, "'bss' | 'data' | 'text'");
                self.kill_line();
                return;
            }
        };
        if self.clear_line() {
            let address = self.current_address();
            self.basm.sections.push(Section { kind, address });
        }
    }

    fn values(&mut self, id2: Span) -> Option<(Vec<Value>, bool)> {
        let mut values = vec![];

        let peek = self.lexer.peek();
        let ins = Lexeme::Comma == peek.lex;
        if ins {
            values.push(Value::Ident(id2));
            self.lexer.pop_peek();
        }
        loop {
            let ad = self.non_ws();
            match ad.lex {
                Eol(_) | Eof => break Some((values, ins)),
                Ident => values.push(Value::Ident(ad.span)),
                Str => values.push(Value::String(
                    self.slice((ad.span.from + 1, ad.span.to - 1)).to_owned(),
                )),
                Digit(base) => match u32::from_str_radix(self.slice(ad.span), base as u32) {
                    Ok(n) => values.push(Value::Digit(base, n)),
                    Err(e) => self
                        .errors
                        .push(ParseErrorKind::ParseIntError(ad.span, e).full(ad)),
                },
                OpenBracket => values.push(Value::Deref(self.after_bracket()?)),
                _ => {
                    self.expected(ad, "Ident | Str | Colon | OpenBracket | Digit");
                    self.kill_line();
                    break None;
                }
            }
            let ad = self.non_ws();
            match ad.lex {
                Comma => (),
                Eol(_) | Eof => break Some((values, ins)),
                _ => {
                    let expected = if ins { "Comma" } else { "Comma | Ident | Str" };
                    self.expected(ad, expected);
                    self.kill_line();
                    break None;
                }
            }
        }
    }

    fn non_ws(&mut self) -> Advance {
        while let Lexeme::Whitespace = self.lexer.peek().lex {
            self.lexer.pop_peek();
        }
        self.lexer.advance()
    }

    fn clear_line(&mut self) -> bool {
        let ad = self.non_ws();
        if let Eol(_) | Eof = ad.lex {
            true
        } else {
            self.expected(ad, "Whitespace");
            self.kill_line();
            false
        }
    }

    fn kill_line(&mut self) {
        while !matches!(self.lexer.advance().lex, Eol(_) | Eof) {}
    }

    fn current_address(&mut self) -> u16 {
        u16::try_from(self.basm.lines.len()).expect("failed to convert line address to u16")
    }

    fn expected(&mut self, ad: Advance, expected: impl Into<String>) {
        self.errors
            .push(ParseErrorKind::Expected(ad, expected.into()).full(ad));
    }
    fn slice(&self, span: impl Into<Span>) -> &'a str {
        span.into().slice(self.basm.src)
    }

    fn after_bracket(&mut self) -> Option<Span> {
        let ident = self.non_ws();
        match ident.lex {
            Ident => (),
            Eol(_) | Eof => {
                self.errors
                    .push(ParseErrorKind::InputEnd(ident.line).full(ident));
                return None;
            }
            _ => {
                self.expected(ident, "Ident");
                self.kill_line();
                return None;
            }
        }
        let close = self.non_ws();
        match close.lex {
            CloseBracket => (),
            Eol(_) | Eof => {
                self.errors
                    .push(ParseErrorKind::InputEnd(close.line).full(close));
                return None;
            }
            _ => {
                self.expected(close, "CloseBracket");
                self.kill_line();
                return None;
            }
        }
        // values.push(Value::Deref(ident.span));
        Some(ident.span)
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
