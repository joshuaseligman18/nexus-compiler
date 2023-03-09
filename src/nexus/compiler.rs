use log::*;

use crate::util::nexus_log;
use crate::nexus::{lexer::Lexer, token::Token, parser::Parser, cst::Cst};

// Function to compile multiple programs
pub fn compile(source_code: &str) {
    let mut lexer: Lexer = Lexer::new(source_code);
    let mut parser: Parser = Parser::new();

    // Clean up the output area
    Cst::clear_display();
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

        nexus_log::insert_empty_line();

        if lex_res.is_err() {
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Parser,
                String::from("Parsing skipped due to lex failure")
            );

            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("CST display skipped due to lex failure")
            );

            // No need to move on if lex failed, so can go to next program
            continue;
        }

        // Log the program we are lexing
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Parser,
            format!("Parsing program {}", program_number)
        );

        let token_stream: Vec<Token> = lex_res.unwrap();
        let parse_res: Result<Cst, ()> = parser.parse_program(&token_stream);

        if parse_res.is_err() {
            nexus_log::insert_empty_line();

            // Do not show CST unless parse is successful
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::LogSources::Nexus,
                String::from("CST display skipped due to parse failure")
            );
            continue;
        }

        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("CST display for Program {} is below", program_number)
        );
        let cst: Cst = parse_res.unwrap();
        cst.display(&program_number);

        // nexus_log::insert_empty_line();
    }
}
