use ahash::AHashMap;

use Literal::*;

use crate::lex::{Lexer, Literal, Span};
use crate::{Basm, Line};

pub fn parse(src: &str) -> (Basm, Vec<ParseError>) {
    use crate::lex::LineKind::*;

    let mut labels = AHashMap::new();
    let mut lines = Vec::new();
    let mut sections = Vec::new();
    let mut errors = Vec::new();

    let mut lexer = Lexer::new(src);

    let mut i = 0;
    let mut ls = 0;
    while let Some(ll) = lexer.line() {
        let start = lexer.line_starts()[ls - 1];
        let end = lexer.line_starts()[ls];
        let src_line = &src[start as usize..end as usize];
        let literals = ll.slice_lit(&lexer.literals());
        match ll.kind {
            Empty => {
                lines.push(Line::NoOp);
                if !literals.is_empty() {
                    // handle error
                }
            }
            Label => {
                todo!()
                // labels.insert(, v)
            }
            Section => todo!(),
            Global => todo!(),
            Instruction => todo!(),
            Variable => todo!(),
        };
        while i < literals.len() {}
        ls += 1;
    }

    (
        Basm {
            src,
            labels,
            lines,
            sections,
        },
        errors,
    )
}

fn label(literals: &[(Span, Literal)]) -> Result<(), ParseError> {
    let ident = ident(literals)?;
    todo!()
}

fn ident(literals: &[(Span, Literal)]) -> Result<usize, ParseError> {
    let mut liter = literals.iter().enumerate();
    loop {
        let Some((i, (_, lit))) = liter.next() else {
            break Err(CompilerError::WrongToken().into());
        };
        match lit {
            Whitespace => (),
            Ident => break Ok(i),
            Other => eprintln!("Other token found"),
            _ => {
                eprintln!("incorrect token found: {lit:?}");
                break Err(CompilerError::WrongToken().into());
            }
        }
    }
    // Err(())
}

fn values() {}

#[derive(Debug)]
pub enum ParseError {
    User(UserError),
    Compiler(CompilerError),
}

#[derive(Debug)]
pub enum UserError {}

#[derive(Debug)]
pub enum CompilerError {
    WrongToken(),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::User(user_error) => user_error.fmt(f),
            ParseError::Compiler(compiler_error) => compiler_error.fmt(f),
        }
    }
}
impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for ParseError {}
impl From<UserError> for ParseError {
    fn from(value: UserError) -> Self {
        Self::User(value)
    }
}
impl From<CompilerError> for ParseError {
    fn from(value: CompilerError) -> Self {
        Self::Compiler(value)
    }
}
// use crate::lex::{LexOutput, Literal, Span};
//
// #[derive(Debug)]
// pub struct Parser<S> {
//     lo: LexOutput<S>,
// }
//
// struct Literator<'a> {
//     pos: usize,
//     items: &'a [(Span, Literal)],
// }
//
// pub enum Value {
//     Digit(DigitKind, u32),
//     Ident(Span),
//     String(String),
// }
//
// pub enum DigitKind {
//     Binary,
//     Octal,
//     Decimal,
//     Hex,
// }
//
// impl<'a> Literator<'a> {
//     fn new(items: &'a [(Span, Literal)]) -> Self {
//         Self { pos: 0, items }
//     }
//     fn back(&mut self) {
//         self.pos = self.pos.saturating_sub(1);
//     }
// }
//
// impl<'a> Iterator for Literator<'a> {
//     type Item = (usize, Span, Literal);
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.items
//             .get(self.pos)
//             .map(|&(s, l)| (self.pos, s, l))
//             .inspect(|_| self.pos += 1)
//     }
// }
//
// impl<S: AsRef<str>> Parser<S> {
//     pub fn new(src: S) -> Self {
//         let lo = LexOutput::lex_all(src);
//         Self { lo }
//     }
//     // TODO: proper error handling
//     pub fn parse(&self) -> () {
//         use crate::lex::LineKind::*;
//         // use crate::lex::Literal::*;
//         let src = self.lo.src.as_ref();
//         for al in &self.lo.lines {
//             let line_src = al.line_src(src);
//             let line = al.line;
//             let literals = line.slice_lit(&self.lo.literals);
//             let mut liter = Literator::new(literals);
//             match line.kind {
//                 Empty if literals.is_empty() => {}
//                 Empty => {
//                     eprintln!("empty line with extra values found: {literals:?}");
//                 }
//                 Label => {
//                     let label = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                     self.colon(&mut liter)
//                         .expect("Invalid State Encountered: Colon Not Found");
//                 }
//                 Section => {
//                     let ident = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                     assert_eq!(ident.slice(line_src), "section");
//                     let name = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                 }
//                 Global => {
//                     let ident = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                     assert_eq!(ident.slice(line_src), "global");
//                 }
//                 Instruction => {
//                     let instruction = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                     println!("ins: {instruction:?}");
//                 }
//                 Variable => {
//                     let name = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                     let r#type = self
//                         .ident(&mut liter)
//                         .expect("Invalid State Encountered: Ident Not Found");
//                     println!("var: {name:?} {type:?}");
//                 }
//             }
//             // NOTE: if trailing tokens are left, the wll become errors,
//             // the previous parsers NEED to consume all tokens for it to be valid
//             self.consume_rest(&mut liter);
//             if let Some(comment) = al.line.comment {
//                 // store comments seperately
//             }
//         }
//     }
//
//     fn find(&self, liter: &mut Literator, find: Literal) -> Option<Span> {
//         use Literal::*;
//         while let Some((i, span, lit)) = liter.next() {
//             if lit == find {
//                 return Some(span);
//             }
//             match lit {
//                 Other => self.handle_other(i, span),
//                 Whitespace => (),
//                 _ => {
//                     liter.back();
//                     return None;
//                 }
//             }
//         }
//         None
//     }
//
//     fn colon(&self, liter: &mut Literator) -> Option<Span> {
//         self.find(liter, Literal::Colon)
//     }
//
//     fn ident(&self, liter: &mut Literator) -> Option<Span> {
//         self.find(liter, Literal::Ident)
//     }
//
//     fn values(&self, liter: &mut Literator, src: &str) -> Vec<Value> {
//         use Literal::*;
//         let mut values = Vec::new();
//         let mut comma = false;
//         while let Some((i, span, lit)) = liter.next() {
//             let number = |radix| {
//                 u32::from_str_radix(span.slice(src), radix)
//                     .inspect_err(|e| eprintln!("{e}"))
//                     .unwrap_or_default()
//             };
//             values.push(match lit {
//                 Whitespace => continue,
//                 Binary => Value::Digit(DigitKind::Binary, number(2)),
//                 Octal => Value::Digit(DigitKind::Octal, number(8)),
//                 Decimal => Value::Digit(DigitKind::Decimal, number(10)),
//                 Hex => Value::Digit(DigitKind::Hex, number(16)),
//                 Ident => Value::Ident(span),
//                 String => Value::String(span.slice(src).to_owned()),
//                 Comma => {
//                     if comma {
//                         eprintln!("extra comma");
//                     }
//                     comma = true;
//                     continue;
//                 }
//                 Colon => {
//                     eprintln!("unexpected colon");
//                     continue;
//                 }
//                 OpenBracket => todo!(),
//                 CloseBracket => todo!(),
//                 Other => {
//                     self.handle_other(i, span);
//                     continue;
//                 }
//             });
//             comma = false;
//         }
//         values
//     }
//
//     fn consume_rest(&self, liter: &mut Literator) {
//         use Literal::*;
//         while let Some((i, span, lit)) = liter.next() {
//             match lit {
//                 Whitespace => (),
//                 Other => self.handle_other(i, span),
//                 _ => {
//                     eprintln!("extra lit found: {lit:?} at {i}:{span:?}");
//                 }
//             }
//         }
//     }
//
//     fn handle_other(&self, line: usize, span: Span) {
//         eprintln!(
//             "other literal '{}' found: at {line}:{span:?}",
//             span.slice(self.lo.line_src(line))
//         );
//     }
// }
