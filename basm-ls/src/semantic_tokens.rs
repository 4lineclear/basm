use std::ops::BitAnd;
use std::sync::Arc;

use basm::lex::{LexOutput, LineKind, Span};
use tower_lsp::lsp_types::{Range, SemanticToken, SemanticTokenModifier, SemanticTokenType};

#[allow(unused)]
#[derive(Debug)]
enum TypeKind {
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
    fn push(&mut self, line: u32, span: Span, kind: TypeKind, modi: impl Into<TokenModded>) {
        let mut start = span.from;
        if self.prev_line == line {
            debug_assert!(
                start >= self.prev_end,
                "{start} < {}, {span:?} {kind:?}",
                self.prev_end
            );
            start -= self.prev_end;
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

pub(crate) fn semantic_tokens(
    lex: &LexOutput<Arc<str>>,
    range: Option<Range>,
) -> Vec<SemanticToken> {
    use LineKind::*;
    let mut data = Tokenizer::default();

    for (line, al) in lex.lines.iter().enumerate() {
        let line = line as u32;
        if range.is_some_and(|range| line < range.start.line || line > range.end.line) {
            continue;
        }
        match al.line.kind {
            Empty => (),
            Label(name) => {
                data.push(line, name, TypeKind::Function, TokenMod::Declaration);
            }
            Section(sec, name) => {
                data.push(line, sec, TypeKind::Keyword, TokenMod::None);
                data.push(
                    line,
                    name,
                    TypeKind::Parameter,
                    TokenMod::Declaration & TokenMod::Definition,
                );
            }
            Instruction(span) => {
                let kind = match span.slice(lex.line_src(line as usize)) {
                    "global" => TypeKind::Keyword,
                    _ => TypeKind::Function,
                };
                data.push(line, span, kind, TokenMod::None);
            }
            Variable(name, var_type) => {
                data.push(line, name, TypeKind::Variable, TokenMod::None);
                data.push(line, var_type, TypeKind::Type, TokenMod::None);
            }
        }
        for &(lspan, l) in al.line.slice_lit(&lex.literals) {
            use basm::lex::Literal::*;
            let kind = match l {
                Hex | Octal | Binary | Decimal | Float => TypeKind::Number,
                Ident => TypeKind::Variable,
                String => TypeKind::String,
                Deref => {
                    data.push(
                        line,
                        Span::point(lspan.from),
                        TypeKind::Operator,
                        TokenMod::None,
                    );
                    data.push(
                        line,
                        Span::new(lspan.from + 1, lspan.to - 1),
                        TypeKind::Variable,
                        TokenMod::None,
                    );
                    data.push(
                        line,
                        Span::point(lspan.to - 1),
                        TypeKind::Operator,
                        TokenMod::None,
                    );
                    continue;
                }
                Other | Whitespace => {
                    continue;
                }
            };
            data.push(line, lspan, kind, TokenMod::None);
        }
        if let Some(comment) = al.line.comment {
            data.push(line, comment, TypeKind::Comment, TokenMod::None);
        }
    }
    data.inner
}
