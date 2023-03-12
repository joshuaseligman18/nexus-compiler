use std::fmt;

use crate::nexus::token::Token;

pub enum AstNode {
    Terminal(Token),
    NonTerminal(NonTerminals)
}

// Instead of deriving Debug, we are going to implement it so it prints out the way we want it to when the debug print is called
// Basic idea found from https://users.rust-lang.org/t/how-can-i-implement-fmt-display-for-enum/24111/10
impl fmt::Debug for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            // Print out the token text if it is a terminal
            AstNode::Terminal(tok) => {
                write!(f, "{}", tok.text)
            },
            // Otherwise print the display text for NonTerminals, which will be in PascalCase
            AstNode::NonTerminal(non_term) => {
                write!(f, "{}", non_term)
            }
        }
    }
}

#[derive (Debug, strum::Display)]
#[strum (serialize_all = "PascalCase")]
pub enum NonTerminals {
    Block,
    VarDecl,
    Assign,
    Print,
    While,
    If
}

// The type of a node relative to the tree
#[derive (Debug, PartialEq)]
pub enum AstNodeTypes {
    Root,
    Branch,
    Leaf
}
