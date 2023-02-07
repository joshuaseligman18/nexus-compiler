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

        // A program consists of a block followed by an EOP marker
        // First will check block and then the token
        if self.parse_block(token_stream).is_err() || self.match_token(token_stream, TokenType::Symbol(Symbols::EOP)).is_err() {
            // Log that we are parsing the program
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                String::from("Parser failed")
            );
        }
    }

    fn parse_block(&mut self, token_stream: &Vec<Token>) -> Result<(), ()> {
        // Log that we are parsing a block
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Block")
        );

        // Check for left brace
        let lbrace_err: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::LBrace));
        if lbrace_err.is_err() {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                lbrace_err.unwrap_err()
            );
            return Err(());
        }

        self.parse_statement_list(token_stream);

        // Check for right brace
        let rbrace_err: Result<(), String> = self.match_token(token_stream, TokenType::Symbol(Symbols::RBrace));
        if rbrace_err.is_err() {
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                rbrace_err.unwrap_err()
            );
            return Err(());
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

    fn parse_statement_list(&mut self, token_stream: &Vec<Token>) {
        // Log that we are parsing a statement list
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing StatementList")
        );

        // Make sure that the statement list is not empty
        if !token_stream[self.cur_token_index].token_type.eq(&TokenType::Symbol(Symbols::RBrace)) {
            self.parse_statement(token_stream);
            if !token_stream[self.cur_token_index].token_type.eq(&TokenType::Symbol(Symbols::RBrace)) {
                self.parse_statement_list(token_stream);
            }
        }
    }

    fn parse_statement(&mut self, token_stream: &Vec<Token>) {
        // Log that we are parsing a statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing Statement")
        );
        match &token_stream[self.cur_token_index].token_type {
            TokenType::Keyword(Keywords::Print) => self.parse_print_statement(token_stream),
            TokenType::Identifier(_) => {}, // Parse assignment statement
            TokenType::Keyword(Keywords::Int) | TokenType::Keyword(Keywords::String) | TokenType::Keyword(Keywords::Boolean) => {}, // Parse var declaration
            TokenType::Keyword(Keywords::While) => {}, // Parse while statement
            TokenType::Keyword(Keywords::If) => {},// Parse if statement,
            TokenType::Symbol(Symbols::LBrace) => self.parse_block(token_stream).unwrap(),
            _ => {} // Figure out something to do to end the recursion
        }
    }

    fn parse_print_statement(&mut self, token_stream: &Vec<Token>) {//-> Result<bool, ()> {
        // Log that we are parsing a print statement
        nexus_log::log(
            nexus_log::LogTypes::Debug,
            nexus_log::LogSources::Parser,
            String::from("Parsing PrintStatement")
        );
        // If it is not a print statement, no error actually happened, just did not match
        if self.match_token(token_stream, TokenType::Keyword(Keywords::Print)).is_err() {
            // Return false
            // return Ok(false);
        } else {
            // Do all expression things
        }

        // return Ok(true);
    }
}