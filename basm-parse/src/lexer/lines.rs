use crate::{
    lexer::{
        Cursor,
        LexKind::{self, *},
        Lexeme,
    },
    span::{BSpan, TSpan},
    util::AsBSpan,
    Comment, DocComment, DocStyle,
};

/// A single item within a list
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Instruction {
    arguments: TSpan,
    span: BSpan,
}

/// An instruction's argument
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Argument {
    span: BSpan,
    kind: ArgKind,
}

/// An instruction's argument
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArgKind {
    // #[default]
    // None,
    Ident,
    Literal {
        kind: LiteralKind,
        suffix_start: u32,
    },
}

use error::{Error, ErrorKind};

use super::LiteralKind;

pub mod error;

#[cfg(test)]
mod test;

// TODO: add lines parser

pub fn parse(input: &str) -> Parser {
    let mut parser = Parser::new(input);
    parser.all_lines();
    parser
}

#[derive(Default)]
pub struct Parser<'a> {
    curr: Option<(Lexeme, BSpan)>,
    pub cursor: Cursor<'a>,
    pub comments: Vec<Comment>,
    pub docs: Vec<DocComment>,
    pub errors: Vec<Error>,
    pub instrutions: Vec<Instruction>,
    pub arguments: Vec<Argument>,
}

// #[allow(unused)]
impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            cursor: Cursor::new(input),
            ..Default::default()
        }
    }
    fn advance(&mut self) -> (Lexeme, BSpan) {
        self.curr.take().unwrap_or_else(|| {
            let token = self.cursor.advance();
            (token, self.span(token))
        })
    }
    // fn reverse(&mut self, token: Lexeme) -> Option<(Lexeme, BSpan)> {
    //     self.curr.replace((token, self.span(token)))
    // }
    fn until_non_wc(&mut self) -> (Lexeme, BSpan) {
        loop {
            let (token, span) = self.advance();
            if !self.handle_wc(token) {
                break (token, span);
            }
        }
    }
    fn push_comment(&mut self, style: Option<DocStyle>, content: impl Into<AsBSpan>) {
        let span = self.span(content);
        match style {
            Some(style) => self.docs.push(DocComment(style, span)),
            None => self.comments.push(Comment(span)),
        }
    }
    fn handle_wc(&mut self, token: Lexeme) -> bool {
        if let LineComment { doc_style } = token.kind {
            self.push_comment(doc_style, token);
        }
        if let BlockComment {
            doc_style,
            terminated,
        } = token.kind
        {
            if !terminated {
                self.push_err(Error {
                    span: self.span(token),
                    kind: ErrorKind::Unterminated,
                });
            }
            self.push_comment(doc_style, token);
        }
        matches!(
            token.kind,
            LineComment { .. } | BlockComment { .. } | Whitespace
        )
    }
    fn err_expected(&mut self, span: impl Into<AsBSpan>, expected: impl Into<Box<[LexKind]>>) {
        self.push_err(Error {
            span: self.span(span),
            kind: ErrorKind::Expected(expected.into()),
        });
    }
    fn push_err(&mut self, err: impl Into<Error>) {
        let err: Error = err.into();
        let Some(prev) = self.errors.last_mut() else {
            self.errors.push(err);
            return;
        };
        if let Some(err) = prev.congregate(err) {
            self.errors.push(err);
        }
    }
    #[must_use]
    const fn src(&self) -> &str {
        self.cursor.src()
    }
    #[must_use]
    pub fn slice(&self, span: impl Into<AsBSpan>) -> &str {
        self.span(span).slice(self.src())
    }
    pub fn span(&self, span: impl Into<AsBSpan>) -> BSpan {
        match span.into() {
            AsBSpan::Len(len) => self.token_span(len),
            AsBSpan::Lex(token) => self.token_span(token.len),
            AsBSpan::Span(span) => span,
        }
    }
    #[must_use]
    const fn token_pos(&self) -> u32 {
        self.cursor.lex_pos()
    }
    #[must_use]
    const fn token_span(&self, len: u32) -> BSpan {
        BSpan::new(self.token_pos(), self.token_pos() + len)
    }
}

impl Parser<'_> {
    pub fn all_lines(&mut self) {
        while self.next_line() {}
    }
    pub fn next_line(&mut self) -> bool {
        loop {
            let mut ins = match self.instruction() {
                Ok(Some(ins)) => ins,
                Ok(None) => continue,
                Err(ParseEnd::Eof) => break false,
                Err(ParseEnd::NewLine) => break true,
            };
            let nl = loop {
                match self.argument() {
                    Ok(Some(arg)) => {
                        ins.arguments.to = self.arguments.len() as u32 + 1;
                        self.arguments.push(arg);
                    }
                    Ok(None) => (),
                    Err(ParseEnd::Eof) => break false,
                    Err(ParseEnd::NewLine) => break true,
                }
            };
            self.instrutions.push(ins);
            break nl;
        }
    }
    fn instruction(&mut self) -> ParseRes<Option<Instruction>> {
        let args = self.arguments.len() as u32;
        let (lex, span) = self.until_non_wc();
        let inst = Instruction {
            span,
            arguments: TSpan::new(args, args),
        };
        match lex.kind {
            Eof => Err(ParseEnd::Eof),
            NewLine => Err(ParseEnd::NewLine),
            Ident => Ok(Some(inst)),
            _ => {
                self.handle_wc(lex);
                self.err_expected(span, [Ident]);
                Ok(None)
            }
        }
    }
    fn argument(&mut self) -> ParseRes<Option<Argument>> {
        let (lex, span) = self.until_non_wc();
        match lex.kind {
            Eof => Err(ParseEnd::Eof),
            NewLine => Err(ParseEnd::NewLine),
            Ident => Ok(Some(Argument {
                span,
                kind: ArgKind::Ident,
            })),
            Literal { kind, suffix_start } => Ok(Some(Argument {
                span,
                kind: ArgKind::Literal { kind, suffix_start },
            })),
            Dot | Colon | Comma => Ok(None),
            _ => {
                self.handle_wc(lex);
                self.err_expected(span, [Ident, Comma, LITERAL]);
                Ok(None)
            }
        }
    }
}

const LITERAL: LexKind = LexKind::Literal {
    kind: crate::lexer::LiteralKind::Int {
        base: crate::lexer::Base::Binary,
        empty_int: false,
    },
    suffix_start: 0,
};

#[allow(dead_code)]
const LINE_COMMENT: LexKind = LexKind::LineComment { doc_style: None };

type ParseRes<T> = Result<T, ParseEnd>;

enum ParseEnd {
    Eof,
    NewLine,
}
