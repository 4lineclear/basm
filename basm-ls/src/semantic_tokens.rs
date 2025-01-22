use std::ops::BitAnd;

use basm::lex::Advance;
// use basm::lex::{LexOutput, LineKind, Span};
use tower_lsp::lsp_types::{Range, SemanticToken, SemanticTokenModifier, SemanticTokenType};

#[allow(unused)]
#[derive(Debug)]
enum TokenKind {
    Namespace,
    Type,
    Class,
    Enum,
    Interface,
    Struct,
    TypeParameter,
    Parameter,
    Variable,
    Property,
    EnumMember,
    Event,
    Function,
    Method,
    Macro,
    Keyword,
    Modifier,
    Comment,
    String,
    Number,
    Regexp,
    Operator,
    Decorator,
}

pub const TOKEN_TYPES: [SemanticTokenType; 23] = [
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::CLASS,
    SemanticTokenType::ENUM,
    SemanticTokenType::INTERFACE,
    SemanticTokenType::STRUCT,
    SemanticTokenType::TYPE_PARAMETER,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::EVENT,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::METHOD,
    SemanticTokenType::MACRO,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MODIFIER,
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::REGEXP,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::DECORATOR,
];

#[allow(unused)]
enum TokenMod {
    None = 0b0,
    Declaration = 0b1,
    Definition = 0b10,
    Readonly = 0b100,
    Static = 0b1000,
    Deprecated = 0b10000,
    Abstract = 0b100000,
    Async = 0b1000000,
    Modification = 0b10000000,
    Documentation = 0b100000000,
    DefaultLibrary = 0b1000000000,
}

impl BitAnd for TokenMod {
    type Output = u32;

    fn bitand(self, rhs: Self) -> Self::Output {
        self as u32 & rhs as u32
    }
}
impl BitAnd<u32> for TokenMod {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        self as u32 & rhs
    }
}

pub const TOKEN_MODS: [SemanticTokenModifier; 10] = [
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
    SemanticTokenModifier::STATIC,
    SemanticTokenModifier::DEPRECATED,
    SemanticTokenModifier::ABSTRACT,
    SemanticTokenModifier::ASYNC,
    SemanticTokenModifier::MODIFICATION,
    SemanticTokenModifier::DOCUMENTATION,
    SemanticTokenModifier::DEFAULT_LIBRARY,
];

enum TokenModded {
    Variable(u32),
    Static(TokenMod),
}
impl From<TokenModded> for u32 {
    fn from(value: TokenModded) -> Self {
        match value {
            TokenModded::Variable(val) => val,
            TokenModded::Static(token_mod) => token_mod as u32,
        }
    }
}

impl From<TokenMod> for TokenModded {
    fn from(value: TokenMod) -> Self {
        Self::Static(value)
    }
}

impl From<u32> for TokenModded {
    fn from(value: u32) -> Self {
        Self::Variable(value)
    }
}

#[derive(Default)]
struct Tokenizer {
    inner: Vec<SemanticToken>,
    prev_line: u32,
    prev_end: u32,
}

impl Tokenizer {
    fn push(&mut self, ad: Advance, kind: TokenKind, modi: impl Into<TokenModded>) {
        let span = ad.span;
        let line = ad.line;
        let mut start = span.from;

        if self.prev_line == line {
            debug_assert!(
                start >= self.prev_end,
                "{start} < {}, {span:?} {kind:?}",
                self.prev_end
            );
            start -= self.prev_end;
        } else {
            start -= ad.offset;
        }
        self.inner.push(SemanticToken {
            delta_line: line - self.prev_line,
            delta_start: start,
            length: span.to - span.from,
            token_type: kind as u32,
            token_modifiers_bitset: u32::from(modi.into()),
        });
        self.prev_end = span.from;
        self.prev_line = line;
    }
}

fn is_keyword(s: &str) -> bool {
    matches!(s, "global" | "section")
}

impl super::Document {
    pub(crate) fn semantic_tokens(&self, _range: Option<Range>) -> Vec<SemanticToken> {
        use basm::lex::Lexeme::*;
        let mut data = Tokenizer::default();

        let mut li = 0; // line items
        let mut pl = 0; // prev line

        for &ad in &self.lex {
            li = if pl == ad.line { li + 1 } else { 0 };
            let (kind, modi) = match ad.lex {
                Ident if is_keyword(ad.span.slice(&self.source)) => (TokenKind::Keyword, 0),
                Ident if li == 0 => (TokenKind::Function, 0),
                Ident => (TokenKind::Variable, 0),
                Str => (TokenKind::String, 0),
                Colon | OpenBracket | CloseBracket => (TokenKind::Operator, 0),
                Digit(_) => (TokenKind::Number, 0),
                Eol(true) => (TokenKind::Comment, 0),
                Whitespace | Comma | Eol(_) | Eof | Other => continue,
            };
            data.push(ad, kind, modi);
            pl = ad.line;
        }
        data.inner
    }
}
