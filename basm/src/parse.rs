use crate::{
    lex::{
        Advance, DigitBase,
        Lexeme::{self, *},
        Lexer, Span,
    },
    Basm, Line, Section, SectionKind, Value,
};

#[cfg(test)]
mod test;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    basm: Basm<'a>,
    errors: Vec<ParseError>,
}

const VALUES: [Lexeme; 5] = [Ident, String, Colon, OpenBracket, Digit(DigitBase::Decimal)];

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
            match self.parse_line() {
                Ok(true) => (),
                Ok(false) => break,
                Err(e) => self.errors.push(e),
            }
            while let Err(e) = self.eat_rest() {
                self.errors.push(e);
            }
        }
        (self.basm, self.errors)
    }

    fn parse_line(&mut self) -> ParseResult<bool> {
        loop {
            let ad = self.lexer.advance();
            match ad.lex {
                Whitespace => (),
                Ident => {
                    match self.slice(ad.span) {
                        "section" => self.section(ad)?,
                        _ => self.var_or_ins(ad)?,
                    }
                    break Ok(true);
                }
                Eol(_) => {
                    self.basm.lines.push(Line::NoOp);
                    break Ok(true);
                }
                Eof => {
                    self.basm.lines.push(Line::NoOp);
                    return Ok(false);
                }
                _ => return self.handle_etc(ad, [Ident]).full(ad),
            }
        }
    }

    fn section(&mut self, ad: Advance) -> ParseResult<()> {
        match self.value()? {
            Some(Value::Ident(span)) => {
                let kind = match self.slice(span) {
                    "bss" => SectionKind::Bss,
                    "data" => SectionKind::Data,
                    "text" => SectionKind::Text,
                    _ => return ParseErrorKind::InvalidValue(Value::Ident(span)).full(ad),
                };
                let address = self.current_address();
                self.basm.sections.push(Section { kind, address });
                Ok(())
            }
            Some(val) => ParseErrorKind::InvalidValue(val).full(ad),
            None => ParseErrorKind::Terminated.full(ad),
        }
    }

    fn var_or_ins(&mut self, ad: Advance) -> ParseResult<()> {
        let Some(second) = self.value()? else {
            self.basm.lines.push(Line::Instruction {
                values: vec![Value::Ident(ad.span)],
            });
            return Ok(());
        };
        let (is_var, values) = self.values()?;
        if !is_var {
            self.basm.lines.push(Line::Instruction { values });
            return Ok(());
        }
        let Value::Ident(r#type) = second else {
            return ParseErrorKind::InvalidValue(second).full(ad);
        };
        self.basm.lines.push(Line::Variable {
            name: ad.span,
            r#type,
            values,
        });
        Ok(())
    }

    fn values(&mut self) -> ParseResult<(bool, Vec<Value>)> {
        let mut values = Vec::new();
        let mut first_comma = 0;
        loop {
            let ad = self.lexer.peek();
            if first_comma == 0 {
                first_comma = if Comma == ad.lex {
                    self.lexer.advance();
                    1
                } else {
                    -1
                };
            } else if Comma != ad.lex {
                self.lexer.advance();
                return ParseErrorKind::MissingComma(ad.span).full(ad);
            } else {
                self.lexer.advance();
            }
            let Some(value) = self.value()? else {
                break Ok((first_comma > 0, values));
            };
            values.push(value);
        }
    }

    fn value(&mut self) -> ParseResult<Option<Value>> {
        loop {
            let ad = self.lexer.advance();
            let value = match ad.lex {
                Whitespace => continue,
                Ident => Value::Ident(ad.span),
                String => Value::String(self.slice((ad.span.from + 1, ad.span.to - 1)).to_owned()),
                OpenBracket => todo!("deref not supported yet"),
                Digit(digit) => match u32::from_str_radix(self.slice(ad.span), digit as u32) {
                    Ok(n) => Value::Digit(digit, n),
                    Err(e) => return ParseErrorKind::ParseIntError(ad.span, e).full(ad),
                },
                Eof | Eol(_) => break Ok(None),
                _ => return self.handle_etc(ad, VALUES).full(ad),
            };
            break Ok(Some(value));
        }
    }

    fn eat_rest(&mut self) -> ParseResult<()> {
        loop {
            let ad = self.lexer.advance();
            match ad.lex {
                Whitespace => (),
                Eol(_) | Eof => break Ok(()),
                _ => {
                    break self
                        .handle_etc(ad, [Whitespace, Eol(false), Eol(true), Eof])
                        .full(ad)
                }
            }
        }
    }

    #[must_use]
    fn handle_etc(&mut self, ad: Advance, expected: impl Into<Box<[Lexeme]>>) -> ParseErrorKind {
        match ad.lex {
            Other => ParseErrorKind::OtherInput(ad.span),
            _ => ParseErrorKind::Expected(ad, expected.into()),
        }
    }

    fn current_address(&mut self) -> u16 {
        u16::try_from(self.basm.lines.len()).expect("failed to convert line address to u16")
    }

    fn slice(&self, span: impl Into<Span>) -> &'a str {
        span.into().slice(self.basm.src)
    }
}

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    line: u32,
    offset: u32,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Expected(Advance, Box<[Lexeme]>),
    OtherInput(Span),
    NoOpValues,
    ParseIntError(Span, std::num::ParseIntError),
    MissingComma(Span),
    InvalidValue(Value),
    Terminated,
}

impl ParseErrorKind {
    fn full<T>(self, ad: Advance) -> ParseResult<T> {
        Err(ParseError {
            line: ad.line,
            offset: ad.offset,
            kind: self,
        })
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
                expected {expected:?} but found {lex:?}",
                span.from - offset,
                span.to - offset,
            ),
            OtherInput(span) => writeln!(
                f,
                "unknown input found at {line}:{},{}",
                span.from - offset,
                span.to - offset,
            ),
            NoOpValues => writeln!(f, "line {line} has values but no instruction"),
            ParseIntError(span, parse_int_error) => writeln!(
                f,
                "parse int error found at {line}:{},{}. \
                error: {parse_int_error}",
                span.from - offset,
                span.to - offset,
            ),
            MissingComma(span) => writeln!(
                f,
                "missing comma at {line}:{},{}",
                span.from - offset,
                span.to - offset,
            ),
            InvalidValue(value) => {
                writeln!(f, "variable at line {line} had invalid type: {value:?}",)
            }
            Terminated => writeln!(f, "unexpected termination on line {line}",),
        }
    }
}

impl std::error::Error for ParseError {}
