use log::{info, debug};
use regex::{Regex, Match};

use crate::util::nexus_log;
use crate::nexus::{lexer::Lexer, token::Token};

// Function to compile multiple programs
pub fn compile(source_code: &str) {
    let mut lexer: Lexer = Lexer::new(source_code);

    // Clean up the output area
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::LogSources::Nexus,
        String::from("Nexus compile called")
    );

    // Keep track of the number of programs
    let mut program_number: u32 = 1;

    // Go through each program
    while lexer.has_program_to_lex() {
        nexus_log::insert_empty_line();

        // Log the program we are on
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Compiling program {}", program_number)
        );
        program_number += 1;

        // Lex the program
        let lex_res: Result<Vec<Token>, ()> = lexer.lex_program();
        if lex_res.is_err() {
            // No need to move on if lex failed, so can go to next program
            continue;
        }
        let token_stream: Vec<Token> = lex_res.unwrap();
        debug!("{:?}; {}, {}", token_stream, token_stream.len(), token_stream.capacity());
    }
}