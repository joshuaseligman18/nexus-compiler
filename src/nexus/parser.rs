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
            nexus_log::LogTypes::Info,
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
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Parser,
            String::from("Parsing Block")
        );

        // Check for left brace
        if self.match_token(token_stream, TokenType::Symbol(Symbols::LBrace)).is_err() {
            return Err(());
        }

        // Check for right brace
        if self.match_token(token_stream, TokenType::Symbol(Symbols::RBrace)).is_err() {
            return Err(());
        }

        // Return ok if we have received everything
        return Ok(());
    }

    // Function to ensure the token is correct
    fn match_token(&mut self, token_stream: &Vec<Token>, expected_token: TokenType) -> Result<(), ()> {
        // Check for an index out of range error
        if self.cur_token_index < token_stream.len() {
            // Consume the next token
            let cur_token: &Token = &token_stream[self.cur_token_index];
            self.cur_token_index += 1;

            match &cur_token.token_type {
                // Check the symbols
                TokenType::Symbol(symbol) => {
                    // Make sure it is equal
                    if cur_token.token_type.eq(&expected_token) {
                        return Ok(());
                    } else {
                        // Otherwise print an error message
                        nexus_log::log(
                            nexus_log::LogTypes::Error,
                            nexus_log::LogSources::Parser,
                            format!("Invalid token at {:?}; Found {:?}, but expected {:?}", cur_token.position, cur_token.token_type, expected_token)
                        );
                    }
                },
                _ => {

                }
            }
        } else {
            // Error if no more tokens and expected something else
            nexus_log::log(
                nexus_log::LogTypes::Error,
                nexus_log::LogSources::Parser,
                format!("Missing token {:?} ", expected_token)
            );
        }

        return Ok(());
    }
}