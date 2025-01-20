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
