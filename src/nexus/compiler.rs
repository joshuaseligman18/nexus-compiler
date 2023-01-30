use log::{info, debug};
use regex::{Regex, Match};

use crate::util::nexus_log;
use crate::nexus::{lexer::Lexer, token::Token};

// Function to compile multiple programs
pub fn compile(source_code: String) {
    let mut lexer: Lexer = Lexer::new();

    // Clean up the output area
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::LogSources::Nexus,
        String::from("Nexus compile called")
    );

    // Keep track of the starting position of the current program
    let mut program_start: usize = 0;

    // Keep track of the number of programs
    let mut program_number: u32 = 1;

    // Go through each program
    while program_start < source_code.len() && has_content(&source_code, &program_start) {
        nexus_log::insert_empty_line();

        // Log the program we are on
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::LogSources::Nexus,
            format!("Compiling program {}", program_number)
        );
        program_number += 1;

        // Lex the program
        let lex_res: Result<Vec<Token>, ()> = lexer.lex_program(&source_code, &mut program_start);
        if lex_res.is_err() {
            // No need to move on if lex failed, so can go to next program
            continue;
        }
        debug!("{:?}", lex_res.unwrap());
    }
}

// Function to make sure there is still content to go through
fn has_content(source_code: &str, starting_position: &usize) -> bool {
    // String only has whitespace
    let whitespace_regex: Regex = Regex::new(r"^\s*$").unwrap();

    // Determine if it is only whitespace or if there is content
    if whitespace_regex.is_match(&source_code[*starting_position..]) {
        return false;
    } else {
        return true;
    }
}