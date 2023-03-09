use std::fmt;

use crate::nexus::token::Token;

pub enum CstNode {
    Terminal(Token),
    NonTerminal(NonTerminals)
}

// Instead of deriving Debug, we are going to implement it so it prints out the way we want it to when the debug print is called
// Basic idea found from https://users.rust-lang.org/t/how-can-i-implement-fmt-display-for-enum/24111/10
impl fmt::Debug for CstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            // Print out the token text if it is a terminal
            CstNode::Terminal(tok) => {
                write!(f, "{}", tok.text)
            },
            // Otherwise print the display text for NonTerminals, which will be in PascalCase
            CstNode::NonTerminal(non_term) => {
                write!(f, "{}", non_term)
            }
        }
    }
}

#[derive (Debug, strum::Display)]
#[strum (serialize_all = "PascalCase")]
pub enum NonTerminals {
    Program,
    Block,
    StatementList,
    Statement,
    PrintStatement,
    AssignmentStatement,
    VarDecl,
    WhileStatement,
    IfStatement,
    Expr,
    IntExpr,
    StringExpr,
    BooleanExpr,
    Id,
    CharList,
    Type,
    Char,
    Space,
    Digit,
    BoolOp,
    BoolVal,
    IntOp
}

// The type of a node relative to the tree
#[derive (Debug, PartialEq)]
pub enum CstNodeTypes {
    Root,
    Branch,
    Leaf
}