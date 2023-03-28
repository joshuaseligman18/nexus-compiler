// Defines a token
#[derive (Debug, Clone)]
pub struct Token {
    // The type of the token
    pub token_type: TokenType,
    // The content of the token
    pub text: String,
    // The position in the source code the token is located
    pub position: (usize, usize)
}

impl Token {
    // Create a new token with the given information
    pub fn new(token_type_in: TokenType, token_text: String, line_number: usize, col_number: usize) -> Self {
        return Token {
            token_type: token_type_in,
            text: token_text,
            position: (line_number, col_number)
        }
    }
}

// Defines the token types and what they hold
#[derive (Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(Keywords),
    Identifier(String),
    Symbol(Symbols),
    Digit(u8),
    Char(String),
    Unrecognized(String)
}

// Defines the keywords
#[derive (Debug, Clone, PartialEq)]
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

// Defines the possible symbols
#[derive (Debug, Clone, PartialEq)]
pub enum Symbols {
    LParen, // (
    RParen, // )
    LBrace, // {
    RBrace, // }
    AdditionOp, // +
    EqOp, // ==
    NeqOp, // !=
    AssignmentOp, // =
    Quote, // "
    EOP // $
}
