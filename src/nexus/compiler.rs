use log::{info, debug};
use regex::{Regex, Match};

use crate::util::nexus_log;
use crate::nexus::{lexer::Lexer, token::Token, parser::Parser};

// Function to compile multiple programs
pub fn compile(source_code: &str) {
    let mut lexer: Lexer = Lexer::new(source_code);
    let mut parser: Parser = Parser::new();

    // Clean up the output area
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::LogSources::Nexus,
        String::from("Nexus compile called")
    );

    // Keep track of the number of programs
    let mut program_number: u32 = 0;

    // Go through each program
    while lexer.has_program_to_lex() {
        program_number += 1;

        nexus_log::insert_empty_line();

        // Log the program we are on
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Compiling program {}", program_number)
        );
        nexus_log::insert_empty_line();

        // Log the program we are lexing
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Lexer,
            format!("Lexing program {}", program_number)
        );

        // Lex the program
        let lex_res: Result<Vec<Token>, ()> = lexer.lex_program();
        if lex_res.is_err() {
            // No need to move on if lex failed, so can go to next program
            continue;
        }

        nexus_log::insert_empty_line();

        // Log the program we are lexing
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Parser,
            format!("Parsing program {}", program_number)
        );

        let token_stream: Vec<Token> = lex_res.unwrap();
        parser.parse_program(&token_stream);
    }
}