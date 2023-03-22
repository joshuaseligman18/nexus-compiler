use std::fmt;

use crate::nexus::token::Token;

pub enum SyntaxTreeNode {
    Terminal(Token),
    NonTerminalCst(NonTerminalsCst),
    NonTerminalAst(NonTerminalsAst)
}

// Instead of deriving Debug, we are going to implement it so it prints out the way we want it to when the debug print is called
// Basic idea found from https://users.rust-lang.org/t/how-can-i-implement-fmt-display-for-enum/24111/10
impl fmt::Debug for SyntaxTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            // Print out the token text if it is a terminal
            SyntaxTreeNode::Terminal(tok) => write!(f, "{}", tok.text),
            // Otherwise print the display text for NonTerminals, which will be in PascalCase
            SyntaxTreeNode::NonTerminalCst(non_term) => write!(f, "{}", non_term),
            SyntaxTreeNode::NonTerminalAst(non_term) => write!(f, "{}", non_term)
        }
    }
}

// Valid nonterminals for a CST
#[derive (Debug, strum::Display)]
#[strum (serialize_all = "PascalCase")]
pub enum NonTerminalsCst {
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

// Valid nonterminals for an AST
#[derive (Debug, strum::Display)]
#[strum (serialize_all = "PascalCase")]
pub enum NonTerminalsAst {
    Block,
    VarDecl,
    Assign,
    Print,
    While,
    If,
    Add,
    IsEq,
    NotEq
}

// The type of a node relative to the tree
#[derive (Debug, PartialEq)]
pub enum SyntaxTreeNodeTypes {
    Root,
    Branch,
    Leaf
}
