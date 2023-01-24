#[derive (Debug, Clone)]
pub enum Token {
    Keyword(Keywords),
    Identifier(String),
    Symbol(Symbols),
    Digit(u32),
    Char(String),
    Unrecognized(String)
}

#[derive (Debug, Clone)]
pub enum Keywords {
    If,
    While,
    Print,
    String,
    Int,
    Boolean,
    True,
    False
}

#[derive (Debug, Clone)]
pub enum Symbols {
    L_Paren,
    R_Paren,
    L_Brace,
    R_Brace,
    Addition_Op,
    Eq_Op,
    Neq_Op,
    Assignment_Op,
    Quote,
}