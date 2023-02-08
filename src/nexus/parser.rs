use log::debug;

use crate::{nexus::token::{Token, TokenType, Symbols, Keywords}, util::nexus_log};

pub struct Parser {
    cur_token_index: usize
}

impl Parser {
    // Constructor for the parser
    pub fn new() -> Self {
        return Parser {
            cur_token_index: 0
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
        }
    }

    fn parse_block(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a block
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Block")
        );

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

        // Return ok if we have received everything that goes into a block
        return Ok(());
    }

    // Function to ensure the token is correct
    fn match_token(&mut self, token_stream: &Vec<Token>, expected_token: TokenType) -> Result<(), String> {
        // Check for an index out of range error
        if self.cur_token_index < token_stream.len() {
            // Get the next token
            let cur_token: &Token = &token_stream[self.cur_token_index];

            match &cur_token.token_type {
                // Check the symbols
                TokenType::Symbol(_) => {
                    // Make sure it is equal
                    if cur_token.token_type.ne(&expected_token) {
                        // Return an error message if the expected token does not line up
                        return Err(format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token));
                    }
                },
                TokenType::Identifier(_) => {
                    match expected_token {
                        // Do nothing because we have a match
                        TokenType::Identifier(_) => {},
                        _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected Identifier(\"a-z\")", cur_token.position, cur_token.token_type))

                    }
                },
                TokenType::Digit(_) => {
                    match expected_token {
                        // Do nothing because we have a match
                        TokenType::Digit(_) => {},
                        _ => return Err(format!("Invalid token at {:?}; Found {:?}, but expected Digit(0-9)", cur_token.position, cur_token.token_type))
                    }
                },
                _ => {

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

        // Make sure that the statement list is not empty (for programs that are {}$)
        if self.cur_token_index < token_stream.len() && !token_stream[self.cur_token_index].token_type.eq(&TokenType::Symbol(Symbols::RBrace)) {
            // Parse the statement
            let statement_res: Result<(), String> = self.parse_statement(token_stream);
            if statement_res.is_err() {
                return statement_res;
            } else if self.cur_token_index < token_stream.len() && !token_stream[self.cur_token_index].token_type.eq(&TokenType::Symbol(Symbols::RBrace)) {
                // StatementList = Statement StatementList, so call parse on the next statement list
                return self.parse_statement_list(token_stream);
            }  else {
                // There is no more to print, so 
                return Ok(());
            }
        } else {
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

        // Assume we are ok
        let mut statement_res: Result<(), String> = Ok(());

        match &token_stream[self.cur_token_index].token_type {
            // Print statements
            TokenType::Keyword(Keywords::Print) => statement_res = self.parse_print_statement(token_stream),

            // Assignment statements
            TokenType::Identifier(_) => {}, // Parse assignment statement

            // VarDecl statements
            TokenType::Keyword(Keywords::Int) | TokenType::Keyword(Keywords::String) | TokenType::Keyword(Keywords::Boolean) => {}, // Parse var declaration

            // While statements
            TokenType::Keyword(Keywords::While) => {}, // Parse while statement

            // If statements
            TokenType::Keyword(Keywords::If) => {},// Parse if statement,

            // Block statements
            TokenType::Symbol(Symbols::LBrace) => statement_res = self.parse_block(token_stream),

            // Invalid statement starter tokens
            _ => {
                statement_res = Err(format!("Invalid statement token [ {:?} ] at {:?}; Valid statement beginning tokens are {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}", token_stream[self.cur_token_index].token_type, token_stream[self.cur_token_index].position, TokenType::Keyword(Keywords::Print), TokenType::Identifier(String::from("a-z")), TokenType::Keyword(Keywords::Int), TokenType::Keyword(Keywords::String), TokenType::Keyword(Keywords::Boolean), TokenType::Keyword(Keywords::While), TokenType::Keyword(Keywords::If), TokenType::Symbol(Symbols::LBrace)));
            }
        }
        return statement_res;
    }

    fn parse_print_statement(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing PrintStatement")
        );

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
        if self.cur_token_index < token_stream.len() {
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

        return Ok(());
    }

    fn parse_expression(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Expr")
        );

        let mut expression_res: Result<(), String> = Ok(());

        match &token_stream[self.cur_token_index].token_type {
            // IntExpr
            TokenType::Digit(_) => expression_res = self.parse_int_expression(token_stream),

            // StringExpr
            TokenType::Symbol(Symbols::Quote) => {},

            // BooleanExpr
            TokenType::Symbol(Symbols::LParen) | TokenType::Keyword(Keywords::False) | TokenType::Keyword(Keywords::True) => {},

            // Id
            TokenType::Identifier(_) => expression_res = self.parse_identifier(token_stream),

            _ => expression_res = Err(format!("Invalid expression token [ {:?} ] at {:?}; Valid expression beginning tokens are Digit(0-9), {:?}, {:?}, {:?}, {:?}, {:?}", token_stream[self.cur_token_index].token_type, token_stream[self.cur_token_index].position, TokenType::Symbol(Symbols::Quote), TokenType::Symbol(Symbols::LParen), TokenType::Keyword(Keywords::False), TokenType::Keyword(Keywords::True), TokenType::Identifier(String::from("a-z")))),
        }

        return expression_res;
    }

    fn parse_int_expression(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an integer expression
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing IntExpr")
        );

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
            // Type check does not matter, so can parse 3 + "007" for now and semantic analysis will catch it
            let second_half_res: Result<(), String> = self.parse_expression(token_stream);
            if second_half_res.is_err() {
                return second_half_res;
            }
        }

        return Ok(());
    }

    fn parse_identifier(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        // Log that we are parsing an identifier
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Id")
        );
        return self.match_token(token_stream, TokenType::Identifier(String::from("a-z")));
    }

    fn parse_int_op(&mut self, token_stream: &Vec<Token>) -> Result<(), String> {
        let res: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::AdditionOp));

        // Only print if the token was consumed because it is being checked
        // more as a peek rather than an actual expectation
        if res.is_ok() {
            // Log that we are parsing an integer operator
            nexus_log::log(
                nexus_log::LogTypes::Debug,
                nexus_log::LogSources::Parser,
                String::from("Parsing intop")
            );
        }

        return res;
    }
}