use log::*;
use crate::{nexus::token::{Token, TokenType, Symbols, Keywords}, util::nexus_log};

use crate::nexus::ast::{Ast};
use crate::nexus::ast_node::{AstNode, NonTerminals, AstNodeTypes};
use crate::nexus::symbol_table::{SymbolTable, Type};

use petgraph::graph::NodeIndex;

use string_builder::Builder;

pub struct SemanticAnalyzer {
    cur_token_index: usize,
    num_errors: i32,
    num_warnings: i32,
    symbol_table: SymbolTable
}

impl SemanticAnalyzer {
    // Constructor for the parser
    pub fn new() -> Self {
        return SemanticAnalyzer {
            cur_token_index: 0,
            num_errors: 0,
            num_warnings: 0,
            symbol_table: SymbolTable::new()
        };
    }

    // Starting function to generate the AST
    pub fn generate_ast(&mut self, token_stream: &Vec<Token>) -> Ast {
        // Basic initialization
        self.cur_token_index = 0;
        let mut ast: Ast = Ast::new();

        // We start with parsing the block because that is the first
        // part with actual content
        self.parse_block(token_stream, &mut ast);

        // Return the AST
        return ast;
    }

    fn parse_block(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a block
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing Block")
        );

        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Block));

        // Advance a token for the left brace
        self.cur_token_index += 1;

        // Parse all of the content inside of the block
        self.parse_statement_list(token_stream, ast);

        // Advance a token for the right brace
        self.cur_token_index += 1;

        // Move up to the previous level
        ast.move_up();
    }

    fn parse_statement_list(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Make sure that the statement list is not empty
        if !self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::RBrace)) {
            // Log that we are parsing a statement list
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::SemanticAnalyzer,
                String::from("Parsing StatementList")
            );
            // Parse the statement
            self.parse_statement(token_stream, ast);
            self.parse_statement_list(token_stream, ast);
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::SemanticAnalyzer,
                String::from("Parsing StatementList (epsilon base case)")
            );
        }
    }

    fn parse_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing Statement")
        );

        // Look ahead to the next token
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            // Parse the next section in the stream based on the next token 
            match next_token.token_type {
                // Print statements
                TokenType::Keyword(Keywords::Print) => self.parse_print_statement(token_stream, ast),

                // Assignment statements
                TokenType::Identifier(_) => self.parse_assignment_statement(token_stream, ast),

                // VarDecl statements
                TokenType::Keyword(Keywords::Int) | TokenType::Keyword(Keywords::String) | TokenType::Keyword(Keywords::Boolean) => self.parse_var_declaration(token_stream, ast),

                // While statements
                TokenType::Keyword(Keywords::While) => self.parse_while_statement(token_stream, ast), 

                // If statements
                TokenType::Keyword(Keywords::If) => self.parse_if_statement(token_stream, ast),

                // Block statements
                TokenType::Symbol(Symbols::LBrace) => self.parse_block(token_stream, ast),

                // Invalid statement starter tokens
                _ => error!("Invalid statement token [ {:?} ] at {:?}; Valid statement beginning tokens are {:?}", next_token.token_type, next_token.position, vec![TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)])
            };

        }
    }

    fn parse_print_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing PrintStatement")
        );

        // Add the PrintStatement node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Print));

        // Increment the token index by 1 for the print keyword
        self.cur_token_index += 1;

        // Increment the token index by 1 for the left paren
        self.cur_token_index += 1;

        // Parse the expression inside the print statement
        self.parse_expression(token_stream, ast);
        
        // Increment the token index by 1 for the right paren
        self.cur_token_index += 1;

        // All good so we move up
        ast.move_up();
    }

    fn parse_assignment_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing an assignment statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing AssignmentStatement")
        );

        // Add the AssignmentStatement node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Assign));

        // Assignment statements begin with an identifier
        self.parse_identifier(token_stream, ast);
        
        // Increment the index for the = sign that parse checked
        self.cur_token_index += 1;

        // The right hand side of the statement is an expression
        self.parse_expression(token_stream, ast);
       
        // Move back up to the level of the statements
        ast.move_up();
    }

    fn parse_var_declaration(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a variable declaration
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing VarDecl")
        );

        // Add the VarDecl node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::VarDecl));

        // Add the type to the AST
        ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
        self.cur_token_index += 1;

        // Then make sure there is a valid identifier
        self.parse_identifier(token_stream, ast);

        ast.move_up();
    }

    fn parse_while_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a while statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing WhileStatement")
        );

        // Add the node for a while statement
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::While));
        self.cur_token_index += 1;
        
        // While has a boolean expression
        self.parse_bool_expression(token_stream, ast);
        
        // The body of the loop is defined by a block
        self.parse_block(token_stream, ast);
       
        // Move up out of the while
        ast.move_up();
    }

    fn parse_if_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing an if statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing IfStatement")
        );

        // Add the IfStatement node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::If));
        self.cur_token_index += 1;

        // If has a boolean expression
        self.parse_bool_expression(token_stream, ast);
        
        // The body of the if-statement is a block
        self.parse_block(token_stream, ast);

        ast.move_up();
    }

    fn parse_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing an expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing Expr")
        );

        // Look ahead to the next token
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            // Assign a result object to expression_res based on the next token in the stream
            match next_token.token_type {
                // IntExpr
                TokenType::Digit(_) => self.parse_int_expression(token_stream, ast),

                // StringExpr
                TokenType::Symbol(Symbols::Quote) => self.parse_string_expression(token_stream, ast),

                // BooleanExpr
                TokenType::Symbol(Symbols::LParen) | TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => self.parse_bool_expression(token_stream, ast),

                // Id
                TokenType::Identifier(_) => self.parse_identifier(token_stream, ast),

                // Parse already ensured correctness, but have to include this case
                _ => error!("Invalid expression token [ {:?} ] at {:?}; Valid expression beginning tokens are [Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}]", next_token.token_type, next_token.position, TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z"))),
            };
        }
    }

    fn parse_int_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing an integer expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing IntExpr")
        );

        match &token_stream[self.cur_token_index + 1].token_type {
            TokenType::Symbol(Symbols::AdditionOp) => {
                // Add the addition nonterminal
                ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Add));
                // Add the first digit
                ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
                self.cur_token_index += 2;
                
                self.parse_expression(token_stream, ast);
                ast.move_up();
            },
            _ => {
                // It is just the digit, so we can just add the digit (current token) to the ast
                ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
                self.cur_token_index += 1;
            }
        }
      }

    fn parse_string_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a string expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing StringExpr")
        );

        // Get the posititon of the string because we will make a new token for the whole thing
        let string_pos: (usize, usize) = token_stream[self.cur_token_index].position.to_owned();

        // Increment the index for the first quote
        self.cur_token_index += 1;

        // We will build the final string
        let mut str_builder: Builder = Builder::default();

        // Continue until we reach the close quote
        while token_stream[self.cur_token_index].token_type.ne(&TokenType::Symbol(Symbols::Quote)) {
            // Add the character text and go to the next token
            str_builder.append(token_stream[self.cur_token_index].text.to_owned());
            self.cur_token_index += 1;
        }
        
        // Increment the index for the close quote
        self.cur_token_index += 1;

        // Crate a new token and add it to the AST
        let new_string: String = str_builder.string().unwrap();
        let new_token: Token = Token::new(TokenType::Char(new_string.to_owned()), new_string.to_owned(), string_pos.0, string_pos.1);  
        ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(new_token));
    }

    fn parse_bool_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing a boolean expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing BooleanExpr")
        );

        match &token_stream[self.cur_token_index].token_type {
            // Long boolean expressions start with LParen
            TokenType::Symbol(Symbols::LParen) => self.long_bool_expression_helper(token_stream, ast),

            // The false and true keywords
            TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => {
                // Add the node for true and false and consume the token
                ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
                self.cur_token_index += 1;
            },

            // Invalid boolean expression, but parse should have already handled this
            _ => error!("Invalid boolean expression token [ {:?} ] at {:?}; Valid boolean expression beginning tokens are {:?}", token_stream[self.cur_token_index].token_type, token_stream[self.cur_token_index].position, vec![TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)])
        }
    }

    fn long_bool_expression_helper(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add 1 to the index for the left paren
        self.cur_token_index += 1;

        // Counter for the open parentheses we are seeing prior to the bool op
        let mut paren_count: i32 = 0;
        // Start with the second token because there is at least 1 before the bool op
        let mut cur_offset: usize = 1;
        // Flag for breaking out of the loop
        let mut bool_op_found: bool = false;


        while !bool_op_found {
            match &token_stream[self.cur_token_index + cur_offset].token_type {
                TokenType::Symbol(Symbols::EqOp) => {
                    if paren_count == 0 {
                        // Only add the operator to the ast if all prior parens are closed
                        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::IsEq));
                        bool_op_found = true;
                    }
                },
                TokenType::Symbol(Symbols::NeqOp) => {
                    if paren_count == 0 {
                        // Only add the operator to the ast if all prior parens are closed
                        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::NotEq));
                        bool_op_found = true;
                    }
                },
                TokenType::Symbol(Symbols::LParen) => {
                    // We found a paren, so have to add it to the count
                    paren_count += 1;
                },
                TokenType::Symbol(Symbols::RParen) => {
                    // The close paren should reduce the count
                    paren_count -= 1;
                },
                _ => {/* Do nothing if none of these symbols */}
            }
            cur_offset += 1;
        }
        
        // Then move on to the left side of the expression
        self.parse_expression(token_stream, ast);

        // Skip over the boolean operator because already took care of that
        self.cur_token_index += 1;

        // Next go through the other side of the expression
        self.parse_expression(token_stream, ast);

        // Increment the index for the close paren
        self.cur_token_index += 1;

        ast.move_up();
    }

    fn parse_identifier(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Log that we are parsing an identifier
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::SemanticAnalyzer,
            String::from("Parsing Id")
        );

        // Add the Id node
        ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
        
        // Increment the position because we consumed another token
        self.cur_token_index += 1;
    }

    fn peek_next_token(&mut self, token_stream: &Vec<Token>) -> Option<Token> {
        // Make sure we are in-bounds
        if self.cur_token_index < token_stream.len() {
            // Clone the token and return
            return Some(token_stream[self.cur_token_index].to_owned());
        } else {
            // If there are no more tokens, then we con return None
            return None;
        }
    }

    fn peek_and_match_next_token(&mut self, token_stream: &Vec<Token>,  expected_token: TokenType) -> bool {
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();
            match &next_token.token_type {
                TokenType::Identifier(_) => {
                    match expected_token {
                        // If next is an identifier, make sure expected is also an identifier
                        TokenType::Identifier(_) => return true,
                        _ => return false
                    }
                },
                TokenType::Keyword(actual_keyword) => {
                    match expected_token {
                        // If they are keywords, have to make sure it is the same keyword
                        TokenType::Keyword(expected_keyword) => {
                            if actual_keyword.eq(&expected_keyword) {
                                return true;
                            } else {
                                return false;
                            }
                        },
                        _ => return false
                    }
                },
                TokenType::Symbol(actual_symbol) => {
                    match expected_token {
                        // If they are symbols, have to make sure it is the same symbol
                        TokenType::Symbol(expected_symbol) => {
                            if actual_symbol.eq(&expected_symbol) {
                                return true;
                            } else {
                                return false;
                            }
                        },
                        _ => return false
                    }
                },
                TokenType::Char(_) => {
                    match expected_token {
                        // Check to make sure both are characters
                        TokenType::Char(_) => return true,
                        _ => return false
                    }
                },
                TokenType::Digit(_) => {
                    match expected_token {
                        // Make sure both are digits
                        TokenType::Digit(_) => return true,
                        _ => return false
                    }
                },
                _ => return false
            }
        } else {
            return false;
        }
    }

    pub fn analyze_program(&mut self, ast: &Ast) {
        self.num_errors = 0;
        self.num_warnings = 0;
        if (*ast).root.is_some() {
            self.analyze_dfs(ast, (*ast).root.unwrap());
            debug!("Symbol table: {:?}", self.symbol_table);
        }
    }

    fn analyze_dfs(&mut self, ast: &Ast, cur_index: usize) {
        // Start off by getting the children of the current node
        let neighbors: Vec<NodeIndex> = (*ast).graph.neighbors(NodeIndex::new(cur_index)).collect();

        match (*ast).graph.node_weight(NodeIndex::new(cur_index)).unwrap() {
            AstNode::NonTerminal(non_terminal) => {
                match non_terminal {
                    NonTerminals::Block => {
                        // Create a new scope for the block
                        self.symbol_table.new_scope();
                        
                        // Everything inside is a statement, so analyze each node
                        for neighbor_index in neighbors.into_iter().rev() {
                            self.analyze_dfs(ast, neighbor_index.index());
                        }

                        // This is the end of the current scope
                        self.symbol_table.end_cur_scope();
                    },
                    NonTerminals::VarDecl => {
                        // Index 0 should be the id token
                        let id_node: &AstNode = (*ast).graph.node_weight(neighbors[0]).unwrap();
                        let mut new_id: Option<String> = None;
                        let mut new_id_pos: (usize, usize) = (0, 0);

                        match id_node {
                            AstNode::Terminal(id_token) => {
                                match &id_token.token_type {
                                    TokenType::Identifier(id_name) => {
                                        new_id = Some(id_name.to_owned());
                                        new_id_pos = id_token.position.to_owned();
                                    },
                                    // Should also never be reached, this is an internal error
                                    _ => error!("Received {:?} at {:?}; Expected an identifier", id_token.token_type, id_token.position)
                                }
                            },
                            // Nonterminal should never be reached
                            AstNode::NonTerminal(_) => error!("Received a nonterminal as child to VarDecl")
                        }

                        // Index 1 should be the type token
                        let type_node: &AstNode = (*ast).graph.node_weight(neighbors[1]).unwrap();
                        // Assume the type node does not exist
                        let mut new_type: Option<Type> = None;

                        match type_node {
                            AstNode::Terminal(id_token) => {
                                match &id_token.token_type {
                                    TokenType::Keyword(keyword) => {
                                        match &keyword {
                                            // Set the appropriate type
                                            Keywords::String => new_type = Some(Type::String),
                                            Keywords::Int => new_type = Some(Type::Int),
                                            Keywords::Boolean => new_type = Some(Type::Boolean),

                                            // Should never be reached once again, but have to add
                                            _ => error!("Received {:?} at {:?}; Expected String, Int, or Boolean", id_token.token_type, id_token.position)
                                        }
                                    },
                                    // Should also never be reached, this is an internal error
                                    _ => error!("Received {:?} at {:?}; Expected a keyword", id_token.token_type, id_token.position)
                                }
                            },
                            // Nonterminal should never be reached
                            AstNode::NonTerminal(_) => error!("Received a nonterminal as child to VarDecl")
                        }

                        if new_id.is_some() && new_type.is_some() {
                            let new_id_res: bool = self.symbol_table.new_identifier(new_id.as_ref().unwrap().to_owned(), new_type.as_ref().unwrap().to_owned());
                            if new_id_res == false {
                                nexus_log::log(
                                    nexus_log::LogTypes::Error,
                                    nexus_log::LogSources::SemanticAnalyzer,
                                    format!("Error at {:?}, Id [ {} ] has already been declared within the current scope", new_id_pos, new_id.unwrap())
                                );
                                self.num_errors += 1;
                            }
                        }
                    },
                    _ => { debug!("Nonterminal: {}", non_terminal); }
                }
            },
            AstNode::Terminal(token) => {
                debug!("Terminal: {:?}", token);
            }
        }
    }
}
