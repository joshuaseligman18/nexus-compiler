use log::debug;

use crate::{nexus::token::{Token, TokenType, Symbols}, util::nexus_log};

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
}