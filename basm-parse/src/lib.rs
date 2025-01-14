// #![allow(unused)]
use self::span::BSpan;

// TODO: consider making the lexer level it's own crate.
pub mod lexer;
pub mod parser;
pub mod span;
pub mod util;

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
// pub struct Section {
//     pub kind: Option<SectionKind>,
//     pub instructions: TSpan,
//     pub span: BSpan,
// }
//
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// pub enum SectionKind {
//     /// dynamic variables
//     Bss,
//     /// static data
//     Data,
//     /// assembly code
//     Text,
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Comment(BSpan);

impl Comment {
    #[must_use]
    pub const fn span(&self) -> BSpan {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocComment(DocStyle, BSpan);

pub use crate::lexer::DocStyle;

impl DocComment {
    #[must_use]
    pub const fn style(&self) -> DocStyle {
        self.0
    }
    #[must_use]
    pub const fn span(&self) -> BSpan {
        self.1
    }
}
