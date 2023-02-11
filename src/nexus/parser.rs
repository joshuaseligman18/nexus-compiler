use log::debug;

use crate::{nexus::token::{Token, TokenType, Symbols, Keywords}, util::nexus_log};

use crate::nexus::cst::{Cst, CstNodeTypes};

pub struct Parser {
    cur_token_index: usize,
    cst: Cst
}

impl Parser {
    // Constructor for the parser
    pub fn new() -> Self {
        return Parser {
            cur_token_index: 0,
            cst: Cst::new(),
        };
    }

    // Calls for a program to be parsed
    pub fn parse_program(&mut self, token_stream: &Vec<Token>) {
        // Log that we are parsing the program
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Program")
        );

        // Reset the index to be 0
        self.cur_token_index = 0;

        let mut success: bool = true;

        // Add the program node
        self.cst.add_node(CstNodeTypes::Root, String::from("Program"));

        // A program consists of a block followed by an EOP marker
        // First will check block and then the token
        let program_block_res: Result<(), String> = self.parse_block(token_stream);
        if program_block_res.is_ok() {
            let eop_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::EOP));
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


        if !success {
            // Log that we are parsing the program
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                String::from("Parser failed")
            );
        } else {
            // Move up (make current None)
            self.cst.move_up();
            debug!("{:?}", self.cst.graph);
            self.cst.create_image();
        }
    }

    fn parse_block(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a block
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Block")
        );

        self.cst.add_node(CstNodeTypes::Branch, String::from("Block"));

        // Check for left brace
        let lbrace_err: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LBrace));
        if lbrace_err.is_err() {
            // Return the error message if the left brace does not exist
            return lbrace_err;
        }

        let statement_list_res: Result<(), String> = self.parse_statement_list(token_stream);
        if statement_list_res.is_err() {
            return statement_list_res;
        }

        // Check for right brace
        let rbrace_err: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RBrace));
        if rbrace_err.is_err() {
            // Return the error message if the right brace does not exist
            return rbrace_err;
        }

        // Move up to the previous level
        self.cst.move_up();

        // Return ok if we have received everything that goes into a block
        return Ok(());
    }

    // Function to ensure the token is correct
    fn match_token(&mut self, token_stream: &Vec<Token>, expected_token: TokenType) -> Result<(), String> {
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
                            TokenType::Digit(_) => return Err(format!("Invalid token at {:?}; Found {:?}, but expected Digit(0-9)", cur_token.position, cur_token.token_type)),
                            _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token))
                        }
                    } else {
                        // Add the node to the CST
                        self.cst.add_node(CstNodeTypes::Leaf, cur_token.text);
                    }
                },
                TokenType::Identifier(_) => {
                    match expected_token {
                        // Add the node to the cst
                        TokenType::Identifier(_) => self.cst.add_node(CstNodeTypes::Leaf, cur_token.text),
                        // Otherwise return an error
                        TokenType::Digit(_) => return Err(format!("Invalid token at {:?}; Found {:?}, but expected Digit(0-9)", cur_token.position, cur_token.token_type)),
                        _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token)),
                    }
                },
                TokenType::Digit(_) => {
                    match expected_token {
                        // Add the new node to the cst
                        TokenType::Digit(_) => self.cst.add_node(CstNodeTypes::Leaf, cur_token.text),
                        // Otherwise return an error
                        _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token))
                    }
                },
                TokenType::Char(_) => {
                    match expected_token {
                        // Add the node to the cst
                        TokenType::Char(_) => self.cst.add_node(CstNodeTypes::Leaf, cur_token.text),
                        // Otherwise return an error
                        TokenType::Digit(_) => return Err(format!("Invalid token at {:?}; Found {:?}, but expected Digit(0-9)", cur_token.position, cur_token.token_type)),
                        _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token))
                    }
                },
                TokenType::Keyword(keyword_actual) => {
                    match &expected_token {
                        // Check to make sure they are both keywords
                        TokenType::Keyword(keyword_expected) => {
                            // See if there is a discrepancy is the actual keywords
                            if keyword_actual.ne(&keyword_expected) {
                                return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token));
                            } else {
                                // Add the node to the cst
                                self.cst.add_node(CstNodeTypes::Leaf, cur_token.text);
                            }
                        },
                        TokenType::Digit(_) => return Err(format!("Invalid token at {:?}; Found {:?}, but expected Digit(0-9)", cur_token.position, cur_token.token_type)),
                        _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token))
                    }
                },
                _ => {
                    // This should never be reached
                    return Err(format!("Unrecognized token [ {:?} ] at {:?}", cur_token.text, cur_token.position))
                }
            }
        } else {
            // Error if no more tokens and expected something
            return Err(format!("Missing token {:?} ", expected_token));
        }

        // Consume the token if it is ok
        self.cur_token_index += 1;
        return Ok(());
    }

    fn parse_statement_list(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a statement list
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing StatementList")
        );

        self.cst.add_node(CstNodeTypes::Branch, String::from("StatementList"));

        // Make sure that the statement list is not empty (for programs that are {}$)
        if !self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::RBrace)) {
            // Parse the statement
            let statement_res: Result<(), String> = self.parse_statement(token_stream);
            if statement_res.is_err() {
                return statement_res;
            } else if !self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::RBrace)) {
                // StatementList = Statement StatementList, so call parse on the next statement list
                let statement_list_res: Result<(), String> = self.parse_statement_list(token_stream);
                if statement_list_res.is_ok() {
                    self.cst.move_up();
                }
                return statement_list_res;
            }  else {
                // There is no more to print, so return
                self.cst.move_up();
                return Ok(());
            }
        } else {
            self.cst.move_up();
            return Ok(());
        }
    }

    fn parse_statement(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Statement")
        );

        // Add the Statement node
        self.cst.add_node(CstNodeTypes::Branch, String::from("Statement"));

        // Look ahead to the next token
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            // Assign a result object to statement_res based on the next token in the stream
            let statement_res: Result<(), String> = 
                match next_token.token_type {
                    // Print statements
                    TokenType::Keyword(Keywords::Print) => self.parse_print_statement(token_stream),

                    // Assignment statements
                    TokenType::Identifier(_) => self.parse_assignment_statement(token_stream),

                    // VarDecl statements
                    TokenType::Keyword(Keywords::Int) | TokenType::Keyword(Keywords::String) | TokenType::Keyword(Keywords::Boolean) => self.parse_var_declaration(token_stream),

                    // While statements
                    TokenType::Keyword(Keywords::While) => self.parse_while_statement(token_stream), 

                    // If statements
                    TokenType::Keyword(Keywords::If) => self.parse_if_statement(token_stream),

                    // Block statements
                    TokenType::Symbol(Symbols::LBrace) => self.parse_block(token_stream),

                    // Invalid statement starter tokens
                    _ => Err(format!("Invalid statement token [ {:?} ] at {:?}; Valid statement beginning tokens are {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}", next_token.token_type, next_token.position, TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)))
                };
            // We have parsed through the statement and can move up
            if statement_res.is_ok() {
                self.cst.move_up();
            }
            return statement_res;
        } else {
            // Return an error because there is no token for the statement
            return Err(format!("Missing statement token; Valid statement beginning tokens are {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}", TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)));
        }
    }

    fn parse_print_statement(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing PrintStatement")
        );

        // Add the PrintStatement node
        self.cst.add_node(CstNodeTypes::Branch, String::from("PrintStatement"));

        // Check for the print keyword
        let keyword_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::Print));
        if keyword_res.is_err() {
            return keyword_res;
        }

        // Check for the left paren
        let lparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LParen));
        if lparen_res.is_err() {
            return lparen_res;
        }

        // First make sure that we have tokens available for an expression
        if self.peek_next_token(token_stream).is_some() {
            // Check to make sure we have a valid expression to print
            let expr_res: Result<(), String> = self.parse_expression(token_stream);
            if expr_res.is_err() {
                return expr_res;
            }
        }

        // Check for the right paren
        let rparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RParen));
        if rparen_res.is_err() {
            return rparen_res;
        }

        // All good so we move up
        self.cst.move_up();
        return Ok(());
    }

    fn parse_assignment_statement(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing AssignmentStatement")
        );

        // Add the AssignmentStatement node
        self.cst.add_node(CstNodeTypes::Branch, String::from("AssignmentStatement"));

        // Assignment statements begin with an identifier
        let id_res: Result<(), String> = self.parse_identifier(token_stream);
        if id_res.is_err() {
            return id_res;
        }

        // Check for a =
        let assignment_op_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::AssignmentOp));
        if assignment_op_res.is_err() {
            return assignment_op_res;
        }

        // The right hand side of the statement is an expression
        let expr_res: Result<(), String> = self.parse_expression(token_stream);
        if expr_res.is_err() {
            return expr_res;
        }

        self.cst.move_up();
        return Ok(());
    }

    fn parse_var_declaration(&mut self, token_stream: &Vec<Token>) -> Result<(), String>{
        // Log that we are parsing a variable declaration
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing VarDecl")
        );

        // Add the VarDecl node
        self.cst.add_node(CstNodeTypes::Branch, String::from("VarDecl"));

        // Make sure we have a valid type
        let type_res: Result<(), String> = self.parse_type(token_stream);
        if type_res.is_err() {
            return type_res;
        }

        // Then make sure there is a valid identifier
        let id_res: Result<(), String> = self.parse_identifier(token_stream);
        if id_res.is_err() {
            return id_res;
        }

        self.cst.move_up();
        return Ok(());
    }

    fn parse_while_statement(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a while statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing WhileStatement")
        );

        // Add the WhileStatementNode
        self.cst.add_node(CstNodeTypes::Branch, String::from("WhileStatement"));

        // Make sure we have the while token
        let while_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::While));
        if while_res.is_err() {
            return while_res;
        }

        // While has a boolean expression
        let bool_expr_res: Result<(), String> = self.parse_bool_expression(token_stream);
        if bool_expr_res.is_err() {
            return bool_expr_res;
        }

        // The body of the loop is defined by a block
        let block_res: Result<(), String> = self.parse_block(token_stream);
        if block_res.is_err() {
            return block_res;
        }

        self.cst.move_up();
        return Ok(());
    }

    fn parse_if_statement(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an if statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing IfStatement")
        );

        // Add the IfStatement node
        self.cst.add_node(CstNodeTypes::Branch, String::from("IfStatement"));

        // Make sure we have the if token
        let if_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::If));
        if if_res.is_err() {
            return if_res;
        }

        // If has a boolean expression
        let bool_expr_res: Result<(), String> = self.parse_bool_expression(token_stream);
        if bool_expr_res.is_err() {
            return bool_expr_res;
        }

        // The body of the if-statement is a block
        let block_res: Result<(), String> = self.parse_block(token_stream);
        if block_res.is_err() {
            return block_res;
        }

        self.cst.move_up();
        return Ok(());
    }

    fn parse_expression(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Expr")
        );

        // Add the Expr node
        self.cst.add_node(CstNodeTypes::Branch, String::from("Expr"));

        // Look ahead to the next token
        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            // Assign a result object to expression_res based on the next token in the stream
            let expression_res: Result<(), String> =
                match next_token.token_type {
                    // IntExpr
                    TokenType::Digit(_) => self.parse_int_expression(token_stream),
    
                    // StringExpr
                    TokenType::Symbol(Symbols::Quote) => self.parse_string_expression(token_stream),
    
                    // BooleanExpr
                    TokenType::Symbol(Symbols::LParen) | TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => self.parse_bool_expression(token_stream),
    
                    // Id
                    TokenType::Identifier(_) => self.parse_identifier(token_stream),
    
                    _ => Err(format!("Invalid expression token [ {:?} ] at {:?}; Valid expression beginning tokens are Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}", next_token.token_type, next_token.position, TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z")))),
                };
    
            if expression_res.is_ok() {
                self.cst.move_up();
            }
            return expression_res;
        } else {
            // There are no more tokens to parse
            return Err(format!("Missing expression token; Valid expression beginning tokens are Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}", TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z"))));
        }
    }


    fn parse_int_expression(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an integer expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing IntExpr")
        );

        // Add the IntExpr node
        self.cst.add_node(CstNodeTypes::Branch, String::from("IntExpr"));

        let first_digit_res: Result<(), String> = self.match_token(token_stream, TokenType::Digit(0));
        if first_digit_res.is_err() {
            return first_digit_res;
        }

        // Check the integer operator
        let int_op_res: Result<(), String> = self.parse_int_op(token_stream);

        // Only continue if it is ok
        // We do not need to throw an error here as we are assuming that the next token was not related
        // The token that was checked was not consumed so the next match_token call will check that token
        if int_op_res.is_ok() {
            // Get the second half of the expression if there is an integer operator and return the error if needed
            // Type check does not matter, so can parse 3 + "hello" for now and semantic analysis will catch it
            let second_half_res: Result<(), String> = self.parse_expression(token_stream);
            if second_half_res.is_err() {
                return second_half_res;
            }
        }

        self.cst.move_up();
        return Ok(());
    }

    fn parse_string_expression(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a string expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing StringExpr")
        );

        // Add the StringExpr node
        self.cst.add_node(CstNodeTypes::Branch, String::from("StringExpr"));

        // Check for the open quote
        let open_quote_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::Quote));
        if open_quote_res.is_err() {
            return open_quote_res;
        }

        // Parse the string contents
        let char_list_res: Result<(), String> = self.parse_char_list(token_stream);
        if char_list_res.is_err() {
            return char_list_res;
        }

        // Check for the close quote
        let close_quote_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::Quote));
        if close_quote_res.is_err() {
            return close_quote_res;
        }

        self.cst.move_up();
        return Ok(());
    }

    fn parse_bool_expression(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a boolean expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing BooleanExpr")
        );

        // Add BooleanExpr node
        self.cst.add_node(CstNodeTypes::Branch, String::from("BooleanExpr"));

        let next_token_peek: Option<Token> = self.peek_next_token(token_stream);
        if next_token_peek.is_some() {
            let next_token: Token = next_token_peek.unwrap();

            let mut bool_expr_res: Result<(), String> = Ok(());
    
            match next_token.token_type {
                TokenType::Symbol(Symbols::LParen) => {
                    // Start with a left paren
                    let lparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LParen));
                    if lparen_res.is_err() {
                        bool_expr_res = lparen_res;
                    } else {
                        // Then move on to the left side of the expression
                        let expr1_res: Result<(), String> = self.parse_expression(token_stream);
                        if expr1_res.is_err() {
                            bool_expr_res = expr1_res;
                        } else {
                            // Next check for a boolean operator
                            let bool_op_res: Result<(), String> = self.parse_bool_op(token_stream);
                            if bool_op_res.is_err() {
                                bool_expr_res = bool_op_res;
                            } else {
                                // Next check for the other side of the expression
                                let expr2_res: Result<(), String> = self.parse_expression(token_stream);
                                if expr2_res.is_err() {
                                    bool_expr_res = expr2_res;
                                } else {
                                    // Lastly close it with a paren
                                    let rparen_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RParen));
                                    bool_expr_res = rparen_res;
                                }
                            }
                        }
                    }
                },
    
                // The false and true keywords
                TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => bool_expr_res = self.parse_bool_val(token_stream),
    
                // Invalid boolean expression
                _ => bool_expr_res = Err(format!("Invalid boolean expression token [ {:?} ] at {:?}; Valid boolean expression beginning tokens are {:?}, {:?}, {:?}", next_token.token_type, next_token.position, TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)))
            }
    
            if bool_expr_res.is_ok() {
                self.cst.move_up();
            }
            return bool_expr_res;
        } else {
            // There are no more tokens to parse
            return Err(format!("Missing boolean expression token; Valid boolean expression beginning tokens are {:?}, {:?}, {:?}", TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)));
        }
    }

    fn parse_identifier(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an identifier
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Id")
        );

        // Add the Id node
        self.cst.add_node(CstNodeTypes::Branch, String::from("Id"));

        // Match the id
        let id_res: Result<(), String> = self.match_token(token_stream, TokenType::Identifier(String::from("a-z")));

        if id_res.is_ok() {
            self.cst.move_up();
        }
        return id_res;
    }

    fn parse_char_list(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a CharList
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing CharList")
        );

        // Add the CharList node
        self.cst.add_node(CstNodeTypes::Branch, String::from("CharList"));

        // Recursion base case
        // We have reached the end of the character list
        if self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::Quote)) {
            self.cst.move_up();
            return Ok(());
        } else {
            let char_res: Result<(), String> = self.parse_char(token_stream);
            if char_res.is_err() {
                // Break from error
                return char_res;
            } else {
                if self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::Quote)) {
                    self.cst.move_up();
                    return Ok(());
                } else {
                    // Otherwise continue for the rest of the string
                    let char_list_res: Result<(), String> = self.parse_char_list(token_stream);
                    if char_list_res.is_ok() {
                        self.cst.move_up();
                    }
                    return char_list_res;
                }
            }
        }
    }

    fn parse_type(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a type
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing type")
        );

        self.cst.add_node(CstNodeTypes::Branch, String::from("type"));

        // Try to consume the int token
        let int_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::Int));

        // If int was bad, then try again with string
        if int_res.is_err() && int_res.as_ref().unwrap_err().starts_with("Invalid") {
            let string_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::String));
 
            // If string was bad, then try again with boolean
            if string_res.is_err() && string_res.as_ref().unwrap_err().starts_with("Invalid") {
                let bool_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::Boolean));

                if bool_res.is_err() && bool_res.as_ref().unwrap_err().starts_with("Invalid") {
                    // Return a better error if a bool val was not found
                    let cur_token: &Token = &token_stream[self.cur_token_index];
                    return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}, {:?}, or {:?}", cur_token.position, cur_token.token_type, TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean)));
                } else {
                    if bool_res.is_ok() {
                        self.cst.move_up();
                    }
                    return bool_res;
                }
            } else {
                // Otherwise we can just return the result
                if string_res.is_ok() {
                    self.cst.move_up();
                }
                return string_res;
            }
        } else {
            if int_res.is_ok() {
                self.cst.move_up();
            }
            return int_res;
        }
    }

    fn parse_char(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a Char
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing char/space")
        );

        // Add the node
        self.cst.add_node(CstNodeTypes::Branch, String::from("char/space"));

        // Make sure we have a character token here
        let char_res: Result<(), String> = self.match_token(token_stream, TokenType::Char(String::from("a-z or space")));

        if char_res.is_ok() {
            self.cst.move_up();
        }

        return char_res;
    }

    fn parse_bool_op(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a boolean operator
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing boolop")
        );

        self.cst.add_node(CstNodeTypes::Branch, String::from("boolop"));

        // Try to consume the == token
        let eq_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::EqOp));

        // If == was bad, then try again with !=
        if eq_res.is_err() && eq_res.as_ref().unwrap_err().starts_with("Invalid") {
            let neq_res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::NeqOp));
 
            // Check to see if the error was an invalid error
            if neq_res.is_err() && neq_res.as_ref().unwrap_err().starts_with("Invalid") {
                // Return a better error if a bool val was not found
                let cur_token: &Token = &token_stream[self.cur_token_index];
                return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?} or {:?}", cur_token.position, cur_token.token_type, TokenType::Symbol(Symbols::EqOp), TokenType::Symbol(Symbols::NeqOp)));
            } else {
                // Otherwise we can just return the result
                if neq_res.is_ok() {
                    self.cst.move_up();
                }
                return neq_res;
            }
        } else {
            if eq_res.is_ok() {
                self.cst.move_up();
            }
            return eq_res;
        }
    }

    fn parse_bool_val(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a boolean operator
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing boolval")
        );

        // Add the boolval node
        self.cst.add_node(CstNodeTypes::Branch, String::from("boolval"));

        // Try to consume the false token
        let false_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::False));

        // If false was bad, then try again with true
        if false_res.is_err() && false_res.as_ref().unwrap_err().starts_with("Invalid") {
            let true_res: Result<(), String> = self.match_token(token_stream, TokenType::Keyword(Keywords::True));

            // Check to see if the error was an invalid error
            if true_res.is_err() && true_res.as_ref().unwrap_err().starts_with("Invalid") {
                // Return a better error if a bool val was not found
                let cur_token: &Token = &token_stream[self.cur_token_index];
                return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?} or {:?}", cur_token.position, cur_token, TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True)));
            } else {
                if true_res.is_ok() {
                    self.cst.move_up();
                }
                // Otherwise we can just return the result
                return true_res;
            }
        } else {
            if false_res.is_ok() {
                self.cst.move_up();
            }
            return false_res;
        }
    }

    fn parse_int_op(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        let has_int_op_next: bool = self.peek_and_match_next_token(token_stream, TokenType::Symbol(Symbols::AdditionOp));

        // Only print if the token was consumed because it is being checked
        // more as a peek rather than an actual expectation
        if has_int_op_next {
            // Log that we are parsing an integer operator
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::Parser,
                String::from("Parsing intop")
            );

            self.cst.add_node(CstNodeTypes::Branch, String::from("intop"));
        }

        // Match the token or get the error
        let res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::AdditionOp));

        // Move up
        if res.is_ok() {
            self.cst.move_up();
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