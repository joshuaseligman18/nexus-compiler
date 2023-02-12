use std::fmt;

use crate::nexus::token::Token;

pub enum CstNode {
    Terminal(Token),
    NonTerminal(NonTerminals)
}

impl fmt::Debug for CstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CstNode::Terminal(tok) => {
                write!(f, "{}", tok.text)
            },
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

#[derive (Debug, PartialEq)]
pub enum CstNodeTypes {
    Root,
    Branch,
    Leaf
}