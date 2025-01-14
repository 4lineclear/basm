use crate::{
    span::{BSpan, LSpan},
    util::AsBSpan,
    Comment, DocComment, DocStyle,
};

use super::{
    Cursor,
    LexKind::{self, *},
    Lexeme, LiteralKind,
};

use error::{Error, ErrorKind};

pub mod error;

#[cfg(test)]
mod test;

pub enum LineItem {
    Unit(Unit),
    Other(Lexeme),
}

// /// A single item within a list
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
// pub struct Instruction {
//     // arguments: TSpan,
//     span: BSpan,
// }
//
/// An instruction's argument
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unit {
    span: BSpan,
    kind: UnitKind,
}

/// An instruction's argument
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitKind {
    Ident,
    Literal {
        kind: LiteralKind,
        suffix_start: u32,
    },
}

// TODO: add lines parser

pub fn parse(input: &str) -> (Vec<LineItem>, Lines) {
    let mut parser = Lines::new(input);
    (parser.all_lis(), parser)
}

#[derive(Default)]
pub struct Lines<'a> {
    ls: bool,
    lines: u32,
    // curr: Option<(Lexeme, BSpan)>,
    pub cursor: Cursor<'a>,
    pub comments: Vec<Comment>,
    pub docs: Vec<DocComment>,
    pub errors: Vec<Error>,
    // pub instrutions: Vec<Instruction>,
    // pub arguments: Vec<Argument>,
}

impl<'a> Lines<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            ls: true,
            cursor: Cursor::new(input),
            ..Default::default()
        }
    }
    fn advance(&mut self) -> (Lexeme, BSpan) {
        let token = self.cursor.advance();
        (token, self.span(token))
        // self.curr.take().unwrap_or_else(|| {
        //     let token = self.cursor.advance();
        //     (token, self.span(token))
        // })
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

// pub struct Line {
//     pub units: Box<[Unit]>,
// }

impl Lines<'_> {
    // pub fn line(&mut self) -> Option<Line> {
    //     let mut units = Vec::new();
    //     loop {
    //         let li = self.next_li();
    //         match li {
    //             // LineItem::Ins(instruction) => todo!(),
    //             LineItem::Unit(unit) => units.push(unit),
    //             LineItem::Other(lexeme) => todo!(),
    //             LineItem::Other(Lexeme { kind: Eof, .. }) => break,
    //             LineItem::Other(Lexeme { kind: NewLine, .. }) => {}
    //         }
    //     }
    //     Some(Line {
    //         units: units.into(),
    //     })
    //     // std::iter::from_fn(|| Some(self.next_li()))
    //     //     .take_while(|li| !matches!(li, LineItem::Other(Lexeme { kind: Eof, .. })))
    //     //     .filter(|li| !matches!(li, LineItem::Other(Lexeme { kind: NewLine, .. })))
    //     //     .collect()
    // }
    pub fn all_lis(&mut self) -> Vec<LineItem> {
        std::iter::from_fn(|| Some(self.next_li()))
            .take_while(|li| !matches!(li, LineItem::Other(Lexeme { kind: Eof, .. })))
            .filter(|li| !matches!(li, LineItem::Other(Lexeme { kind: NewLine, .. })))
            .collect()
    }
    pub fn next_li(&mut self) -> LineItem {
        let r = self.try_next();
        match &r {
            LineItem::Unit(_) => self.ls = false,
            LineItem::Other(lex) => self.ls = lex.kind == NewLine,
        }
        r
    }
    pub fn try_next(&mut self) -> LineItem {
        self.unit()
            .map(LineItem::Unit)
            .unwrap_or_else(LineItem::Other)
    }
    // fn instruction(&mut self) -> Result<Unit, Lexeme> {
    //     let (lex, span) = self.until_non_wc();
    //     let inst = Instruction { span };
    //     match lex.kind {
    //         Ident => Ok(inst),
    //         Eof | NewLine => Err(lex),
    //         _ => {
    //             self.handle_wc(lex);
    //             self.err_expected(span, [Ident]);
    //             Err(lex)
    //         }
    //     }
    // }
    fn unit(&mut self) -> Result<Unit, Lexeme> {
        let (lex, span) = self.until_non_wc();
        match lex.kind {
            Ident => Ok(Unit {
                span,
                kind: UnitKind::Ident,
            }),
            Literal { kind, suffix_start } => Ok(Unit {
                span,
                kind: UnitKind::Literal { kind, suffix_start },
            }),
            Dot | Colon | Comma | Eof | NewLine => Err(lex),
            _ => {
                self.handle_wc(lex);
                self.err_expected(span, [Ident, Comma, LITERAL]);
                Err(lex)
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

// pub type ParseRes<T> = Result<T, ParseEnd>;
//
// pub enum ParseEnd {
//     Eof,
//     NewLine,
// }
