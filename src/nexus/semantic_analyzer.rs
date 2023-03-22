use log::*;
use crate::{nexus::token::{Token, TokenType, Symbols, Keywords}, util::nexus_log};

use crate::nexus::ast::{Ast};
use crate::nexus::ast_node::{AstNode, NonTerminals, AstNodeTypes};
use crate::nexus::symbol_table::{SymbolTable, Type, SymbolTableEntry, SymbolTableEntryField};

use petgraph::graph::NodeIndex;

use string_builder::Builder;

pub struct SemanticAnalyzer {
    cur_token_index: usize,
    num_errors: i32,
    num_warnings: i32,
    pub symbol_table: SymbolTable
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
        self.parse_ast_block(token_stream, &mut ast);

        // Return the AST
        return ast;
    }

    fn parse_ast_block(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Block));

        // Advance a token for the left brace
        self.cur_token_index += 1;

        // Parse all of the content inside of the block
        self.parse_ast_statement_list(token_stream, ast);

        // Advance a token for the right brace
        self.cur_token_index += 1;

        // Move up to the previous level
        ast.move_up();
    }

    fn parse_ast_statement_list(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Make sure that the statement list is not empty
        if token_stream[self.cur_token_index].token_type.ne(&TokenType::Symbol(Symbols::RBrace)) {
            // Parse the statement
            self.parse_ast_statement(token_stream, ast);
            self.parse_ast_statement_list(token_stream, ast);
        } else {
            // Nothing to do here (epsilon base case)
        }
    }

    fn parse_ast_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        let next_token: &Token = &token_stream[self.cur_token_index];
        // Parse the next section in the stream based on the next token 
        match &next_token.token_type {
            // Print statements
            TokenType::Keyword(Keywords::Print) => self.parse_ast_print_statement(token_stream, ast),

            // Assignment statements
            TokenType::Identifier(_) => self.parse_ast_assignment_statement(token_stream, ast),

            // VarDecl statements
            TokenType::Keyword(Keywords::Int) | TokenType::Keyword(Keywords::String) | TokenType::Keyword(Keywords::Boolean) => self.parse_ast_var_declaration(token_stream, ast),

            // While statements
            TokenType::Keyword(Keywords::While) => self.parse_ast_while_statement(token_stream, ast), 

            // If statements
            TokenType::Keyword(Keywords::If) => self.parse_ast_if_statement(token_stream, ast),

            // Block statements
            TokenType::Symbol(Symbols::LBrace) => self.parse_ast_block(token_stream, ast),

            // Invalid statement starter tokens
            _ => error!("Invalid statement token [ {:?} ] at {:?}; Valid statement beginning tokens are {:?}", next_token.token_type, next_token.position, vec![TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)])
        }
    }

    fn parse_ast_print_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add the PrintStatement node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Print));

        // Increment the token index by 1 for the print keyword
        self.cur_token_index += 1;

        // Increment the token index by 1 for the left paren
        self.cur_token_index += 1;

        // Parse the expression inside the print statement
        self.parse_ast_expression(token_stream, ast);
        
        // Increment the token index by 1 for the right paren
        self.cur_token_index += 1;

        // All good so we move up
        ast.move_up();
    }

    fn parse_ast_assignment_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add the AssignmentStatement node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Assign));

        // Assignment statements begin with an identifier
        self.parse_ast_identifier(token_stream, ast);
        
        // Increment the index for the = sign that parse checked
        self.cur_token_index += 1;

        // The right hand side of the statement is an expression
        self.parse_ast_expression(token_stream, ast);
       
        // Move back up to the level of the statements
        ast.move_up();
    }

    fn parse_ast_var_declaration(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add the VarDecl node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::VarDecl));

        // Add the type to the AST
        ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
        self.cur_token_index += 1;

        // Then make sure there is a valid identifier
        self.parse_ast_identifier(token_stream, ast);

        ast.move_up();
    }

    fn parse_ast_while_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add the node for a while statement
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::While));
        self.cur_token_index += 1;
        
        // While has a boolean expression
        self.parse_ast_bool_expression(token_stream, ast);
        
        // The body of the loop is defined by a block
        self.parse_ast_block(token_stream, ast);
       
        // Move up out of the while
        ast.move_up();
    }

    fn parse_ast_if_statement(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add the IfStatement node
        ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::If));
        self.cur_token_index += 1;

        // If has a boolean expression
        self.parse_ast_bool_expression(token_stream, ast);
        
        // The body of the if-statement is a block
        self.parse_ast_block(token_stream, ast);

        ast.move_up();
    }

    fn parse_ast_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Look ahead to the next token
        let next_token: &Token = &token_stream[self.cur_token_index];
        // Generate AST based on the next token to determine what type of expression to work with
        match &next_token.token_type {
            // IntExpr
            TokenType::Digit(_) => self.parse_ast_int_expression(token_stream, ast),

            // StringExpr
            TokenType::Symbol(Symbols::Quote) => self.parse_ast_string_expression(token_stream, ast),

            // BooleanExpr
            TokenType::Symbol(Symbols::LParen) | TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => self.parse_ast_bool_expression(token_stream, ast),

            // Id
            TokenType::Identifier(_) => self.parse_ast_identifier(token_stream, ast),

            // Parse already ensured correctness, but have to include this case
            _ => error!("Invalid expression token [ {:?} ] at {:?}; Valid expression beginning tokens are [Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}]", next_token.token_type, next_token.position, TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z"))),
        }
    }

    fn parse_ast_int_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        match &token_stream[self.cur_token_index + 1].token_type {
            TokenType::Symbol(Symbols::AdditionOp) => {
                // Add the addition nonterminal
                ast.add_node(AstNodeTypes::Branch, AstNode::NonTerminal(NonTerminals::Add));
                // Add the first digit
                ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
                self.cur_token_index += 2;
                
                self.parse_ast_expression(token_stream, ast);
                ast.move_up();
            },
            _ => {
                // It is just the digit, so we can just add the digit (current token) to the ast
                ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
                self.cur_token_index += 1;
            }
        }
      }

    fn parse_ast_string_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
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

    fn parse_ast_bool_expression(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
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
        self.parse_ast_expression(token_stream, ast);

        // Skip over the boolean operator because already took care of that
        self.cur_token_index += 1;

        // Next go through the other side of the expression
        self.parse_ast_expression(token_stream, ast);

        // Increment the index for the close paren
        self.cur_token_index += 1;

        ast.move_up();
    }

    fn parse_ast_identifier(&mut self, token_stream: &Vec<Token>, ast: &mut Ast) {
        // Add the Id node
        ast.add_node(AstNodeTypes::Leaf, AstNode::Terminal(token_stream[self.cur_token_index].to_owned()));
        
        // Increment the position because we consumed another token
        self.cur_token_index += 1;
    }

    pub fn analyze_program(&mut self, ast: &Ast, program_number: &u32) -> bool {
        self.num_errors = 0;
        self.num_warnings = 0;
        self.symbol_table.reset();
        if (*ast).root.is_some() {
            self.analyze_dfs(ast, (*ast).root.unwrap());
            debug!("Symbol table: {:?}", self.symbol_table);

            self.num_warnings += self.symbol_table.mass_warnings();

            // We need to determine final string that gets printed
            // and format it nicely based on the number of errors and warnings
            let mut output_string: String = format!("Semantic analysis for program {} ", *program_number);
            if self.num_errors == 0 {
                output_string.push_str("completed with 0 errors and ");
            } else {
                output_string.push_str(format!("failed with {} error", self.num_errors).as_str());
                if self.num_errors != 1 {
                    output_string.push_str("s");
                }
                output_string.push_str(" and ");
            }

            output_string.push_str(format!("{} warning", self.num_warnings).as_str());
            if self.num_warnings != 1 {
                output_string.push_str("s");
            }

            if self.num_errors == 0 {
                nexus_log::log(
                    nexus_log::LogTypes::Info,
                    nexus_log::LogSources::SemanticAnalyzer,
                    output_string
                );
                return true;
            } else {
                nexus_log::log(
                    nexus_log::LogTypes::Error,
                    nexus_log::LogSources::SemanticAnalyzer,
                    output_string
                );
                return false;
            }
        }
        return false;
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
                        nexus_log::log(
                            nexus_log::LogTypes::Debug,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Entering new scope {}", self.symbol_table.cur_scope.unwrap())
                        );

                        // Everything inside is a statement, so analyze each node
                        for neighbor_index in neighbors.into_iter().rev() {
                            self.analyze_dfs(ast, neighbor_index.index());
                        }

                        nexus_log::log(
                            nexus_log::LogTypes::Debug,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Exiting scope {}", self.symbol_table.cur_scope.unwrap())
                        );
                        // This is the end of the current scope
                        self.symbol_table.end_cur_scope();
                    },
                    NonTerminals::VarDecl => self.analyze_var_decl(ast, &neighbors),
                    NonTerminals::Assign => self.analyze_assignment(ast, &neighbors),
                    NonTerminals::Print => {
                        // Only have to make sure that the types are ok, but don't
                        // care what is inside because that was taken care of in parse
                        self.derive_type(ast, neighbors[0]);
                    },
                    NonTerminals::If | NonTerminals::While => {
                        // A condition_type of None means there was an error in the analysis
                        // Parse guarantees that it is either true, false, or a boolean
                        // expression, so do not need to make sure that it is a boolean because
                        // it always will return as such if no errors
                        self.derive_type(ast, neighbors[1]);

                        // This is the block, so can perform DFS on it
                        self.analyze_dfs(ast, neighbors[0].index());
                    },
                    _ => error!("Cannot analyze {:?} through DFS", non_terminal)
                }
            },
            AstNode::Terminal(_) => error!("Cannot analyze a terminal as part of the DFS, only nonterminals can be analyzed")
        }
    }

    // Function to derive the type of a node and returns the left-most token position
    fn derive_type(&mut self, ast: &Ast, node_index: NodeIndex) -> Option<(Type, (usize, usize))> {
        let ast_node: &AstNode = (*ast).graph.node_weight(node_index).unwrap();

        let mut output: Option<(Type, (usize, usize))> = None;

        match ast_node {
            AstNode::Terminal(token) => {
                match &token.token_type {
                    // Digits are integer types
                    TokenType::Digit(_) => output = Some((Type::Int, token.position.to_owned())),
                    // The AST combined CharLists into a single Char token, so this is a string
                    TokenType::Char(_) => output = Some((Type::String, token.position.to_owned())),
                    TokenType::Identifier(id_name) => {
                        // Get the identifier from the symbol table
                        let symbol_table_entry: Option<&SymbolTableEntry> = self.get_identifier(&token);
                        if symbol_table_entry.is_some() {
                            // Make clones of a these fields to prevent the rust borrow checker
                            // from going crazy
                            let symbol_table_entry_type: Type = symbol_table_entry.unwrap().symbol_type.to_owned();
                            let symbol_table_entry_position: (usize, usize) = symbol_table_entry.unwrap().position.to_owned();
                            let symbol_table_entry_is_initialized: bool = symbol_table_entry.unwrap().is_initialized.to_owned();
                            let symbol_table_entry_is_used: bool = symbol_table_entry.unwrap().is_used.to_owned();
                            let symbol_table_entry_scope: usize = symbol_table_entry.unwrap().scope.to_owned();

                            nexus_log::log(
                                nexus_log::LogTypes::Debug,
                                nexus_log::LogSources::SemanticAnalyzer,
                                format!("Id [ {} ] declared in scope {} at position {:?} is valid and has been used at {:?} in scope {}",
                                        id_name, symbol_table_entry_scope, symbol_table_entry_position, token.position, self.symbol_table.cur_scope.unwrap())
                            );

                            if !symbol_table_entry_is_initialized {
                                // Throw a warning for using an uninitialized variable
                                nexus_log::log(
                                    nexus_log::LogTypes::Warning,
                                    nexus_log::LogSources::SemanticAnalyzer,
                                    format!("Warning at {:?}; Use of uninitialized variable [ {} ] that was declared at {:?}",
                                            token.position, id_name, symbol_table_entry_position)
                                );
                                self.num_warnings += 1;
                            }

                            // Make sure the variable is marked as used
                            if !symbol_table_entry_is_used {
                                self.symbol_table.set_entry_field(id_name, SymbolTableEntryField::Used);
                            }

                            // Return the type and position of the identifier being used
                            output = Some((symbol_table_entry_type, token.position.to_owned()));
                        }
                    },
                    TokenType::Keyword(keyword) => {
                        match &keyword {
                            // True and false keywords are booleans
                            Keywords::True | Keywords::False => output = Some((Type::Boolean, token.position.to_owned())),
                            _ => error!("Cannot derive type of keyword {:?}, only true and false", keyword)
                        }
                    },
                    _ => error!("Cannot derive type of terminal {:?}, only Digit, Char, Identifier, and Keyword", token)
                }
            },
            AstNode::NonTerminal(non_terminal) => {
                // Get the children nodes for the nonterminal node
                let non_term_neighbors: Vec<NodeIndex> = (*ast).graph.neighbors(node_index).collect();
                match &non_terminal {
                    // Analyze the addition statement
                    NonTerminals::Add => output = self.analyze_add(ast, &non_term_neighbors),
                    // Analyze the boolean expression
                    NonTerminals::IsEq | NonTerminals::NotEq => output = self.analyze_eq_neq(ast, &non_term_neighbors),
                    _ => error!("Cannot derive type of nonterminal {:?}, only Add, IsEq, and NotEq", non_terminal)
                }
            }
        }

        return output;
    }

    fn analyze_var_decl(&mut self, ast: &Ast, neighbors: &Vec<NodeIndex>) {
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
        let mut type_pos: (usize, usize) = (0, 0);

        match type_node {
            AstNode::Terminal(id_token) => {
                type_pos = id_token.position.to_owned();
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

        // Check to make sure that there weren't any internal errors (should never happen if AST
        // was properly generated
        if new_id.is_some() && new_type.is_some() {
            let cur_scope = self.symbol_table.cur_scope.unwrap().to_owned();
            // Attempt to add the new id to the symbol table
            let new_id_res: bool = self.symbol_table.new_identifier(new_id.as_ref().unwrap().to_owned(), new_type.as_ref().unwrap().to_owned(), new_id_pos);
            
            // Throw an error if the id wasn't added to the symbol table
            if new_id_res == false {
                nexus_log::log(
                    nexus_log::LogTypes::Error,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Error at {:?}; Id [ {} ] has already been declared within the current scope", new_id_pos, new_id.unwrap())
                );
                self.num_errors += 1;
            } else {
                nexus_log::log(
                    nexus_log::LogTypes::Debug,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Id [ {} ] of type {:?} has been declared at {:?} in scope {}", new_id.unwrap(), new_type.unwrap(), type_pos, cur_scope)
                );
            }
        }
    }

    fn analyze_assignment(&mut self, ast: &Ast, neighbors: &Vec<NodeIndex>) {
        // Index 1 should be the id token
        let id_node: &AstNode = (*ast).graph.node_weight(neighbors[1]).unwrap();
        let mut id_info: Option<(Type, String, bool, bool, (usize, usize), (usize, usize))> = None;

        match id_node {
            // We assume this is an identifier because of the grammar and the AST
            // should be correct
            AstNode::Terminal(id_token) => {
                let cur_scope: usize = self.symbol_table.cur_scope.unwrap().to_owned();
                // Get the id result
                let id_res: Option<&SymbolTableEntry> = self.get_identifier(&id_token);
                if id_res.is_some() {
                    // Collect copies of a bunch of information to prevent rust borrow errors
                    id_info = Some((id_res.unwrap().symbol_type.to_owned(), id_token.text.to_owned(),
                                    id_res.unwrap().is_initialized.to_owned(), id_res.unwrap().is_used.to_owned(),
                                    id_res.unwrap().position.to_owned(), id_token.position.to_owned()));
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::SemanticAnalyzer,
                        format!("Id [ {} ] declared in scope {} at position {:?} is valid at {:?} in scope {}",
                                id_token.text, id_res.unwrap().scope, id_info.as_ref().unwrap().4, id_token.position, cur_scope)
                    );

                }
            },
            // Nonterminal should never be reached
            AstNode::NonTerminal(_) => error!("Received a nonterminal when expecting a terminal to Assign")
        }

        // Index 0 is the value being assigned
        let right_entry = self.derive_type(ast, neighbors[0]);

        // If both sides check out, then we can compare types
        if id_info.is_some() && right_entry.is_some() {
            let id_info_real: (Type, String, bool, bool, (usize, usize), (usize, usize)) = id_info.unwrap();
            let right_entry_real: (Type, (usize, usize)) = right_entry.unwrap();
            
            // Compare the types and throw and error if they do not line up
            if id_info_real.0.ne(&right_entry_real.0) {
                nexus_log::log(
                    nexus_log::LogTypes::Error,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Mismatched types at {:?}; Expected {:?} for the assignment type, but received {:?}", right_entry_real.1, id_info_real.0, right_entry_real.0)
                );
                self.num_errors += 1;
            } else {
                // The variable has now been assigned a value, so make sure it is
                // updated in the symbol table if it has not been done so already
                if id_info_real.2 == false {
                    self.symbol_table.set_entry_field(&id_info_real.1, SymbolTableEntryField::Initialized);
               
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::SemanticAnalyzer,
                        format!("Id [ {} ] declared at {:?} of type {:?} has been initialized with a value of type {:?} at position {:?}",
                                id_info_real.1, id_info_real.4, id_info_real.0, right_entry_real.0, id_info_real.5)
                    );

                    // Throw a warning for the variable being initialized here because
                    // it was already used
                    if id_info_real.3 == true {
                        nexus_log::log(
                            nexus_log::LogTypes::Warning,
                            nexus_log::LogSources::SemanticAnalyzer,
                            format!("Warning at {:?}; Id [ {} ] declared at {:?} is being initialized after already being used",
                                    id_info_real.5, id_info_real.1, id_info_real.4)
                        );
                        self.num_warnings += 1;
                    }
                } else {
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::SemanticAnalyzer,
                        format!("Id [ {} ] declared at {:?} of type {:?} has been assigned a value of type {:?} at position {:?}",
                                id_info_real.1, id_info_real.4, id_info_real.0, right_entry_real.0, id_info_real.5)
                    );
                }
            }
        }
    }

    // Gets a symbol table entry for an identifier, or None if it does not exist
    fn get_identifier(&mut self, id_token: &Token) -> Option<&SymbolTableEntry> {
        let symbol_table_entry: Option<&SymbolTableEntry> = self.symbol_table.get_symbol(&id_token.text);

        if symbol_table_entry.is_none() {
            // Throw an error from the undeclared identifier
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::SemanticAnalyzer,
                format!("Error at {:?}; Id [ {} ] has not been declared", id_token.position, id_token.text)
            );
            self.num_errors += 1;
        }
        return symbol_table_entry;
    }

    // Function that analyzes an add statement
    fn analyze_add(&mut self, ast: &Ast, neighbors: &Vec<NodeIndex>) -> Option<(Type, (usize, usize))> {
        // Index 1 will always be a digit, so that is by default an Int
        // Only have to check index 0 of neighbors, which can be a nonterminal
    
        // Get the type of the right hand side, which can be any expression
        let right_res: Option<(Type, (usize, usize))> = self.derive_type(ast, neighbors[0]);

        if right_res.is_some() {
            let right_res_real: (Type, (usize, usize)) = right_res.unwrap();

            // Since the left is already an int, we have to make sure the right is an int too
            if right_res_real.0.ne(&Type::Int) {
                nexus_log::log(
                    nexus_log::LogTypes::Error,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Error at {:?}; Expected {:?} for the addition expression, but received {:?}", right_res_real.1, Type::Int, right_res_real.0)
                );
                self.num_errors += 1;
                return None;
            } else {
                nexus_log::log(
                    nexus_log::LogTypes::Debug,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Correctly received expression of type {:?} for right side of addition operator at position {:?}",
                            right_res_real.0, right_res_real.1)
                );

                // Get the left side node of the addition for its position
                let left_node: &AstNode = (*ast).graph.node_weight(neighbors[1]).unwrap();
                let mut left_position: (usize, usize) = (0, 0);

                match &left_node {
                    AstNode::Terminal(token) => {
                        // Grab the position of the token
                        // Parse already made sure it is a digit
                        left_position = token.position.to_owned();
                    },
                    AstNode::NonTerminal(non_terminal) => error!("Received [ {:?} ] as a value for addition; Expected a terminal", non_terminal)
                }

                return Some((right_res_real.0, left_position));
            }
        } else {
            return None;
        }
    }

    pub fn analyze_eq_neq(&mut self, ast: &Ast, neighbors: &Vec<NodeIndex>) -> Option<(Type, (usize, usize))>{
        // Get the type for the left side of the boolean operator
        let left_entry: Option<(Type, (usize, usize))> = self.derive_type(ast, neighbors[1]);

        // Get the type for the right side of the boolean operator
        let right_entry: Option<(Type, (usize, usize))> = self.derive_type(ast, neighbors[0]);

        if left_entry.is_some() && right_entry.is_some() {
            // Unwrap both entries
            let left_entry_real: (Type, (usize, usize)) = left_entry.unwrap();
            let right_entry_real: (Type, (usize, usize)) = right_entry.unwrap();

            if left_entry_real.0.ne(&right_entry_real.0) {
                // Throw an error if the types do not match
                nexus_log::log(
                    nexus_log::LogTypes::Error,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Error at {:?}; Mismatched types for boolean expression; Received {:?} on the left side and {:?} on the right side",
                            left_entry_real.1, left_entry_real.0, right_entry_real.0)
                );
                self.num_errors += 1;
                return None;
            } else {
                nexus_log::log(
                    nexus_log::LogTypes::Debug,
                    nexus_log::LogSources::SemanticAnalyzer,
                    format!("Comparing expressions of type {:?} (position {:?}) and type {:?} (position {:?})",
                            left_entry_real.0, left_entry_real.1, right_entry_real.0, right_entry_real.1)
                );
                // Otherwise, we have a boolean result from the expression
                return Some((Type::Boolean, left_entry_real.1));
            }
        } else {
            return None;
        }
    }
}
