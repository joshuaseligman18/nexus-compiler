use log::{info, debug};
use regex::{Regex, Match};

use crate::util::nexus_log;
use crate::nexus::{lexer::Lexer, token::Token};

pub fn compile(source_code: String) {
    let mut lexer: Lexer = Lexer::new();

    // Clean up the output area
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::Sources::Nexus,
        String::from("Compile called")
    );

    // Keep track of the starting position of the current program
    let mut program_start: usize = 0;

    // Keep track of the number of programs
    let mut program_number: u32 = 1;

    // Go through each program
    while program_start < source_code.len() {
        // Log the program we are on
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::Sources::Nexus,
            format!("Compiling program {}", program_number)
        );

        // Lex the program
        let lex_res: Result<Vec<Token>, ()> = lexer.lex_program(&source_code, &mut program_start);
        if lex_res.is_err() {
            // No need to move on if lex failed, so can go to next program
            continue;
        }
        debug!("{:?}", lex_res.unwrap());
        program_number += 1;
    }
}