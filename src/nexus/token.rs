#[derive (Debug, Clone)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Symbol(String),
    Digit(String),
    Char(String),
    Unrecognized(String)
}