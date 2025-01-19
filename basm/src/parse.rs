use crate::lex::Lexer;

#[derive(Debug)]
pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
}
