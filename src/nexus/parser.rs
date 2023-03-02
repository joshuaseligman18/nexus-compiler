use log::debug;

use crate::{nexus::token::{Token, TokenType, Symbols, Keywords}, util::nexus_log};

use crate::nexus::cst::{Cst};
use crate::nexus::cst_node::{CstNode, NonTerminals, CstNodeTypes};

pub struct Parser {
    cur_token_index: usize,
    num_warnings: i32
}

impl Parser {
    // Constructor for the parser
    pub fn new() -> Self {
        return Parser {
            cur_token_index: 0,
            num_warnings: 0
        };
    }
    // Calls for a program to be parsed
    pub fn parse_program(&mut self, token_stream: &Vec<Token>) -> Result<Cst, ()> {
        // Log that we are parsing the program
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Program")
        );

        // Reset the index to be 0 and clear the CST
        self.cur_token_index = 0;
        let mut cst: Cst = Cst::new();

        let mut success: bool = true;
        self.num_warnings = 0;

        // Add the program node
        cst.add_node(CstNodeTypes::Root, CstNode::NonTerminal(NonTerminals::Program));

        // A program consists of a block followed by an EOP marker
        // First will check block and then the token
        let program_block_res: Result<(), String> = self.parse_block(token_stream, &mut cst);
        if program_block_res.is_ok() {
            let eop_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::EOP), &mut cst);
            if eop_res.is_err() {
                success = false;
                nexus_log::log(
                    nexus_log::LogTypes::Error,
                    nexus_log::LogSources::Parser,
                    eop_res.unwrap_err()
                );
            }
        } else {
            success = false;
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                program_block_res.unwrap_err()
            );
        }

        let mut warnings_str: String = format!("{} warning", self.num_warnings);
        if self.num_warnings != 1 {
            warnings_str.push_str("s");
        }

        if !success {
            // Log that we are parsing the program
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                format!("Parser failed and had {}", warnings_str)
            );
            // Parse error
            return Err(());
        } else {
            nexus_log::log(
                nexus_log::LogTypes::Info,
                nexus_log::LogSources::Parser,
                format!("Parser completed successfully with {}", warnings_str)
            );
            // Parsing was successful
            return Ok(cst);
        }
    }

    fn parse_block(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a block
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Block")
        );

        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Block));

        // Check for left brace
        let lbrace_err: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LBrace), cst);
        if lbrace_err.is_err() {
            // Return the error message if the left brace does not exist
            return lbrace_err;
        }

        let statement_list_res: Result<(), String> = self.parse_statement_list(token_stream, cst);
        if statement_list_res.is_err() {
            return statement_list_res;
        }

        // Check for right brace
        let rbrace_err: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RBrace), cst);
        if rbrace_err.is_err() {
            // Return the error message if the right brace does not exist
            return rbrace_err;
        } else {
            // Check 2 tokens prior, which should be a left brace if empty block
            // No need to check for going out of bounds because both left and right brace will already have been consumed
            match &token_stream[self.cur_token_index - 2].token_type {
                TokenType::Symbol(Symbols::LBrace) => {
                    nexus_log::log(
                        nexus_log::LogTypes::Warning,
                        nexus_log::LogSources::Parser,
                        format!("Empty block found starting at {:?}", token_stream[self.cur_token_index - 2].position)
                    );
                    self.num_warnings += 1;
                },
                _ => { /* Do nothing because there is not an empty block */ }
            }
        }

        // Move up to the previous level
        cst.move_up();

        // Return ok if we have received everything that goes into a block
        return Ok(());
    }

    // Function to ensure the token is correct
    fn match_token(&mut self, token_stream: &Vec<Token>, expected_token: TokenType, cst: &mut Cst) -> Result<(), String> {
        // Get the next token
        let cur_token_res: Option<Token> = self.peek_next_token(token_stream);

        // Make sure we have a token
        if cur_token_res.is_some() {
            let cur_token: Token = cur_token_res.unwrap();
            match &cur_token.token_type {
                // Check the symbols
                TokenType::Symbol(_) => {
                    // Make sure it is equal
                    if cur_token.token_type.ne(&expected_token) {
                        // Return an error message if the expected token does not line up
                        match expected_token {
                            TokenType::Digit(_) => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [Digit(0-9)]", cur_token.token_type, cur_token.position)),
                            _ => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [{:?}]", cur_token.token_type, cur_token.position, expected_token))
                        }
                    } else {
                        // Add the node to the CST
                        cst.add_node(CstNodeTypes::Leaf, CstNode::Terminal(cur_token.to_owned()));
                    }
                },
                TokenType::Identifier(_) => {
                    match expected_token {
                        // Add the node to the cst
                        TokenType::Identifier(_) => cst.add_node(CstNodeTypes::Leaf, CstNode::Terminal(cur_token.to_owned())),
                        // Otherwise return an error
                        TokenType::Digit(_) => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [Digit(0-9)]", cur_token.token_type, cur_token.position)),
                        _ => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [{:?}]", cur_token.token_type, cur_token.position, expected_token)),
                    }
                },
                TokenType::Digit(_) => {
                    match expected_token {
                        // Add the new node to the cst
                        TokenType::Digit(_) => cst.add_node(CstNodeTypes::Leaf, CstNode::Terminal(cur_token.to_owned())),
                        // Otherwise return an error
                        _ => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [{:?}]", cur_token.token_type, cur_token.position, expected_token))
                    }
                },
                TokenType::Char(_) => {
                    match expected_token {
                        // Add the node to the cst
                        TokenType::Char(_) => cst.add_node(CstNodeTypes::Leaf, CstNode::Terminal(cur_token.to_owned())),
                        // Otherwise return an error
                        TokenType::Digit(_) => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [Digit(0-9)]", cur_token.token_type, cur_token.position)),
                        _ => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [{:?}]", cur_token.token_type, cur_token.position, expected_token))
                    }
                },
                TokenType::Keyword(keyword_actual) => {
                    match &expected_token {
                        // Check to make sure they are both keywords
                        TokenType::Keyword(keyword_expected) => {
                            // See if there is a discrepancy is the actual keywords
                            if keyword_actual.ne(&keyword_expected) {
                                return Err(format!("Invalid token at {:?}; Found {:?}, but expected [{:?}]", cur_token.position, cur_token.token_type, expected_token));
                            } else {
                                // Add the node to the cst
                                cst.add_node(CstNodeTypes::Leaf, CstNode::Terminal(cur_token.to_owned()));
                            }
                        },
                        TokenType::Digit(_) => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [Digit(0-9)]", cur_token.token_type, cur_token.position)),
                        _ => return Err(format!("Invalid token [ {:?} ] at {:?}; Expected [{:?}]", cur_token.token_type, cur_token.position, expected_token))
                    }
                },
                _ => {
                    // This should never be reached
                    return Err(format!("Unrecognized token [ {:?} ] at {:?}", cur_token.text, cur_token.position))
                }
            }
        } else {
            // Error if no more tokens and expected something
            return Err(format!("Missing token [{:?}] at end of program", expected_token));
        }

        // Consume the token if it is ok
        self.cur_token_index += 1;
        return Ok(());
    }

    fn match_token_collection(&mut self, token_stream: &Vec<Token>, expected_tokens: Vec<TokenType>, cst: &mut Cst) -> Result<(), String> {
        // Get the next token
        let cur_token_res: Option<Token> = self.peek_next_token(token_stream);

        // Make sure we have a token
        if cur_token_res.is_some() {
            let cur_token: Token = cur_token_res.unwrap();

            // Check to see if we are expecting the token
            if expected_tokens.contains(&cur_token.token_type) {
                // Consume the token if it is ok
                cst.add_node(CstNodeTypes::Leaf, CstNode::Terminal(cur_token.to_owned()));
                self.cur_token_index += 1;
                return Ok(());
            } else {
                return Err(format!("Invalid token [ {:?} ] at {:?}; Expected {:?}", cur_token.token_type, cur_token.position, expected_tokens));
            }
        } else {
            // Error if no more tokens and expected something
            return Err(format!("Missing token {:?} at end of program", expected_tokens));
        }
    }

    fn parse_statement_list(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Make sure that the statement list is not empty
        if !self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::RBrace)) {
            // Log that we are parsing a statement list
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::Parser,
                String::from("Parsing StatementList")
            );
            cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::StatementList));
            // Parse the statement
            let statement_res: Result<(), String> = self.parse_statement(token_stream, cst);
            if statement_res.is_err() {
                // There was an error so break here
                return statement_res;
            } else {
                // StatementList = Statement StatementList, so call parse on the next statement list
                let statement_list_res: Result<(), String> = self.parse_statement_list(token_stream, cst);
                if statement_list_res.is_ok() {
                    cst.move_up();
                }
                return statement_list_res;
            }

        } else {
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::Parser,
                String::from("Parsing StatementList (epsilon base case)")
            );
            // Do nothing here because we have an epsilon with the statement list
            return Ok(());
        }
    }

    fn parse_statement(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Statement")
        );

        // Add the Statement node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Statement));

        // Look ahead to the next token
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            // Assign a result object to statement_res based on the next token in the stream
            let statement_res: Result<(), String> = match next_token.token_type {
                // Print statements
                TokenType::Keyword(Keywords::Print) => self.parse_print_statement(token_stream, cst),

                // Assignment statements
                TokenType::Identifier(_) => self.parse_assignment_statement(token_stream, cst),

                // VarDecl statements
                TokenType::Keyword(Keywords::Int) | TokenType::Keyword(Keywords::String) | TokenType::Keyword(Keywords::Boolean) => self.parse_var_declaration(token_stream, cst),

                // While statements
                TokenType::Keyword(Keywords::While) => self.parse_while_statement(token_stream, cst), 

                // If statements
                TokenType::Keyword(Keywords::If) => self.parse_if_statement(token_stream, cst),

                // Block statements
                TokenType::Symbol(Symbols::LBrace) => self.parse_block(token_stream, cst),

                // Invalid statement starter tokens
                _ => Err(format!("Invalid statement token [ {:?} ] at {:?}; Valid statement beginning tokens are {:?}", next_token.token_type, next_token.position, vec![TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)]))
            };
            // We have parsed through the statement and can move up
            if statement_res.is_ok() {
                cst.move_up();
            }
            return statement_res;
        } else {
            // Return an error because there is no token for the statement
            return Err(format!("Missing statement token at end of program; Valid statement beginning tokens are {:?}", vec![TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)]));
        }
    }

    fn parse_print_statement(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing PrintStatement")
        );

        // Add the PrintStatement node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::PrintStatement));

        // Check for the print keyword
        let keyword_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::Print), cst);
        if keyword_res.is_err() {
            return keyword_res;
        }

        // Check for the left paren
        let lparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LParen), cst);
        if lparen_res.is_err() {
            return lparen_res;
        }

        // First make sure that we have tokens available for an expression
        if self.peek_next_token(token_stream).is_some() {
            // Check to make sure we have a valid expression to print
            let expr_res: Result<(), String> = self.parse_expression(token_stream, cst);
            if expr_res.is_err() {
                return expr_res;
            }
        }

        // Check for the right paren
        let rparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RParen), cst);
        if rparen_res.is_err() {
            return rparen_res;
        }

        // All good so we move up
        cst.move_up();
        return Ok(());
    }

    fn parse_assignment_statement(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing AssignmentStatement")
        );

        // Add the AssignmentStatement node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::AssignmentStatement));

        // Assignment statements begin with an identifier
        let id_res: Result<(), String> = self.parse_identifier(token_stream, cst);
        if id_res.is_err() {
            return id_res;
        }

        // Check for a =
        let assignment_op_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::AssignmentOp), cst);
        if assignment_op_res.is_err() {
            return assignment_op_res;
        }

        // The right hand side of the statement is an expression
        let expr_res: Result<(), String> = self.parse_expression(token_stream, cst);
        if expr_res.is_err() {
            return expr_res;
        }

        cst.move_up();
        return Ok(());
    }

    fn parse_var_declaration(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String>{
        // Log that we are parsing a variable declaration
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing VarDecl")
        );

        // Add the VarDecl node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::VarDecl));

        // Make sure we have a valid type
        let type_res: Result<(), String> = self.parse_type(token_stream, cst);
        if type_res.is_err() {
            return type_res;
        }

        // Then make sure there is a valid identifier
        let id_res: Result<(), String> = self.parse_identifier(token_stream, cst);
        if id_res.is_err() {
            return id_res;
        }

        cst.move_up();
        return Ok(());
    }

    fn parse_while_statement(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a while statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing WhileStatement")
        );

        // Add the WhileStatementNode
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::WhileStatement));

        // Make sure we have the while token
        let while_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::While), cst);
        if while_res.is_err() {
            return while_res;
        }

        // While has a boolean expression
        let bool_expr_res: Result<(), String> = self.parse_bool_expression(token_stream, cst);
        if bool_expr_res.is_err() {
            return bool_expr_res;
        }

        // The body of the loop is defined by a block
        let block_res: Result<(), String> = self.parse_block(token_stream, cst);
        if block_res.is_err() {
            return block_res;
        }

        cst.move_up();
        return Ok(());
    }

    fn parse_if_statement(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing an if statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing IfStatement")
        );

        // Add the IfStatement node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::IfStatement));

        // Make sure we have the if token
        let if_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::If), cst);
        if if_res.is_err() {
            return if_res;
        }

        // If has a boolean expression
        let bool_expr_res: Result<(), String> = self.parse_bool_expression(token_stream, cst);
        if bool_expr_res.is_err() {
            return bool_expr_res;
        }

        // The body of the if-statement is a block
        let block_res: Result<(), String> = self.parse_block(token_stream, cst);
        if block_res.is_err() {
            return block_res;
        }

        cst.move_up();
        return Ok(());
    }

    fn parse_expression(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing an expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Expr")
        );

        // Add the Expr node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Expr));

        // Look ahead to the next token
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            // Assign a result object to expression_res based on the next token in the stream
            let expression_res: Result<(), String> = match next_token.token_type {
                // IntExpr
                TokenType::Digit(_) => self.parse_int_expression(token_stream, cst),

                // StringExpr
                TokenType::Symbol(Symbols::Quote) => self.parse_string_expression(token_stream, cst),

                // BooleanExpr
                TokenType::Symbol(Symbols::LParen) | TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => self.parse_bool_expression(token_stream, cst),

                // Id
                TokenType::Identifier(_) => self.parse_identifier(token_stream, cst),

                _ => Err(format!("Invalid expression token [ {:?} ] at {:?}; Valid expression beginning tokens are [Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}]", next_token.token_type, next_token.position, TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z")))),
            };
    
            if expression_res.is_ok() {
                cst.move_up();
            }
            return expression_res;
        } else {
            // There are no more tokens to parse
            return Err(format!("Missing expression token at end of program; Valid expression beginning tokens are [Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}]", TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z"))));
        }
    }


    fn parse_int_expression(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing an integer expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing IntExpr")
        );

        // Add the IntExpr node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::IntExpr));

        // Parse the first digit and return error if needed
        let first_digit_res: Result<(), String> = self.parse_digit(token_stream, cst);
        if first_digit_res.is_err() {
            return first_digit_res;
        }

        // Check the integer operator
        if self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::AdditionOp)) {     
            let int_op_res: Result<(), String> = self.parse_int_op(token_stream, cst);
    
            if int_op_res.is_err() {
                return int_op_res;
            }

            // Get the second half of the expression if there is an integer operator and return the error if needed
            // Type check does not matter, so can parse 3 + "hello" for now and semantic analysis will catch it
            let second_half_res: Result<(), String> = self.parse_expression(token_stream, cst);
            if second_half_res.is_err() {
                return second_half_res;
            }
        }

        cst.move_up();
        return Ok(());
    }

    fn parse_string_expression(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a string expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing StringExpr")
        );

        // Add the StringExpr node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::StringExpr));

        // Check for the open quote
        let open_quote_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::Quote), cst);
        if open_quote_res.is_err() {
            return open_quote_res;
        }

        // Parse the string contents
        let char_list_res: Result<(), String> = self.parse_char_list(token_stream, cst);
        if char_list_res.is_err() {
            return char_list_res;
        }

        // Check for the close quote
        let close_quote_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::Quote), cst);
        if close_quote_res.is_err() {
            return close_quote_res;
        } else {
            // Check 2 tokens prior, which should be a quote if empty string
            // No need to check for going out of bounds because both quotes will already have been consumed
            match &token_stream[self.cur_token_index - 2].token_type {
                TokenType::Symbol(Symbols::Quote) => {
                    nexus_log::log(
                        nexus_log::LogTypes::Warning,
                        nexus_log::LogSources::Parser,
                        format!("Empty string found starting at {:?}", token_stream[self.cur_token_index - 2].position)
                    );
                    self.num_warnings += 1;
                },
                _ => { /* Do nothing because there is not an empty string */ }
            }
        }

        cst.move_up();
        return Ok(());
    }

    fn parse_bool_expression(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a boolean expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing BooleanExpr")
        );

        // Add BooleanExpr node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::BooleanExpr));

        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            let bool_expr_res: Result<(), String> = match next_token.token_type {
                // Long boolean expressions start with LParen
                TokenType::Symbol(Symbols::LParen) => self.long_bool_expression_helper(token_stream, cst),
    
                // The false and true keywords
                TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => self.parse_bool_val(token_stream, cst),
    
                // Invalid boolean expression
                _ => Err(format!("Invalid boolean expression token [ {:?} ] at {:?}; Valid boolean expression beginning tokens are {:?}", next_token.token_type, next_token.position, vec![TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)]))
            };
    
            if bool_expr_res.is_ok() {
                cst.move_up();
            }
            return bool_expr_res;
        } else {
            // There are no more tokens to parse
            return Err(format!("Missing boolean expression token at end of program; Valid boolean expression beginning tokens are {:?}", vec![TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)]));
        }
    }

    fn long_bool_expression_helper(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        let lparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LParen), cst);
        if lparen_res.is_err() {
            return lparen_res;
        }

        // Then move on to the left side of the expression
        let expr1_res: Result<(), String> = self.parse_expression(token_stream, cst);
        if expr1_res.is_err() {
            return expr1_res;
        }

        // Next check for a boolean operator
        let bool_op_res: Result<(), String> = self.parse_bool_op(token_stream, cst);
        if bool_op_res.is_err() {
            return bool_op_res;
        }

        // Next check for the other side of the expression
        let expr2_res: Result<(), String> = self.parse_expression(token_stream, cst);
        if expr2_res.is_err() {
            return expr2_res;
        }

        // Lastly close it with a paren
        let rparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RParen), cst);
        // Return the result regardless of error or ok
        return rparen_res;
    }

    fn parse_identifier(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing an identifier
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Id")
        );

        // Add the Id node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Id));

        // Match the id
        let id_res: Result<(), String> = self.match_token(token_stream, TokenType::Identifier(String::from("a-z")), cst);

        if id_res.is_ok() {
            cst.move_up();
        }
        return id_res;
    }

    fn parse_char_list(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Recursion base case
        // We have reached the end of the character list
        if self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::Quote)) {
            // Log that we are parsing a CharList
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::Parser,
                String::from("Parsing CharList (epsilon base case)")
            );
            // Do nothing here because we have reached the end of the string (epsilon case)
            return Ok(());
        } else {
            // Log that we are parsing a CharList
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::Parser,
                String::from("Parsing CharList")
            );
    
            // Add the CharList node
            cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::CharList));
            let char_res: Result<(), String> = self.parse_char(token_stream, cst);
            if char_res.is_err() {
                // Break from error
                return char_res;
            } else {
                // Otherwise continue for the rest of the string
                let char_list_res: Result<(), String> = self.parse_char_list(token_stream, cst);
                if char_list_res.is_ok() {
                    cst.move_up();
                }
                return char_list_res;
            }
        }
    }

    fn parse_type(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a type
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing type")
        );

        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Type));

        // Try to consume the int token
        let type_res: Result<(), String> = self.match_token_collection(token_stream, vec![TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean)], cst);
        
        if type_res.is_ok() {
            cst.move_up();
        }

        return type_res;
    }

    fn parse_digit(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log what we are doing
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing digit")
        );

        // Add the node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Digit));

        // Match the token with a digit
        let digit_res: Result<(), String> = self.match_token(token_stream, TokenType::Digit(0), cst);
        if digit_res.is_err() {
            return digit_res;
        } else {
            cst.move_up();
            return Ok(());
        }
    }

    fn parse_char(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Check for the next character's content to have the correct output (space vs char)
        let cur_token: Option<Token> = self.peek_next_token(token_stream);
        if cur_token.is_some() {
            match cur_token.unwrap().text.as_str() {
                " " => {
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::Parser,
                        String::from("Parsing space")
                    );

                    // Add the node
                    cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Space));
                },
                _ => {
                    // Log that we are parsing a Char
                    nexus_log::log(
                        nexus_log::LogTypes::Debug,
                        nexus_log::LogSources::Parser,
                        String::from("Parsing char")
                    );
                    cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::Char));
                }
            }
        }

        // Make sure we have a character token here
        let char_res: Result<(), String> = self.match_token(token_stream, TokenType::Char(String::from("a-z or space")), cst);

        if char_res.is_ok() {
            cst.move_up();
        }

        return char_res;
    }

    fn parse_bool_op(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a boolean operator
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing boolop")
        );

        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::BoolOp));

        // Try to consume the token
        let bool_op_res: Result<(), String> = self.match_token_collection(token_stream, vec![TokenType::Symbol(Symbols::EqOp), TokenType::Symbol(Symbols::NeqOp)], cst);

        if bool_op_res.is_ok() {
            cst.move_up();
        }
        
        return bool_op_res;
    }

    fn parse_bool_val(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing a boolean operator
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing boolval")
        );

        // Add the boolval node
        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::BoolVal));

        // Attempt to consume the token
        let bool_val_res: Result<(), String> = self.match_token_collection(token_stream, vec![TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)], cst);

        if bool_val_res.is_ok() {
            // Move up if appropriate to do so
            cst.move_up();
        }

        return bool_val_res;
    }

    fn parse_int_op(&mut self, token_stream: &Vec<Token>, cst: &mut Cst) -> Result<(), String> {
        // Log that we are parsing an integer operator
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing intop")
        );

        cst.add_node(CstNodeTypes::Branch, CstNode::NonTerminal(NonTerminals::IntOp));

        // Match the token or get the error
        let res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::AdditionOp), cst);

        // Move up
        if res.is_ok() {
            cst.move_up();
        }

        return res;
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
}