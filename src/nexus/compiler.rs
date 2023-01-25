use log::{info, debug};
use regex::{Regex, Match};

use crate::util::nexus_log;
use crate::nexus::{lexer, token::Token};

pub fn compile(source_code: String) {
    // Get the programs
    let programs: Vec<&str> = get_individual_programs(&source_code);

    // Clean up the output area
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::Sources::Nexus,
        String::from("Compile called")
    );

    // Go through each program
    for (i, program) in programs.iter().enumerate() {
        // Log the program we are on
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::Sources::Nexus,
            format!("Compiling program {} of {}", i + 1, programs.len())
        );

        // Lex the program
        let lex_res: Result<Vec<Token>, ()> = lexer::lex_program(program);
        if lex_res.is_err() {
            // No need to move on if lex failed, so can go to next program
            continue;
        }
        debug!("{:?}", lex_res.unwrap());
    }
}

// Function that separates multiple inputted programs
fn get_individual_programs(source_code: &str) -> Vec<&str> {
    // This is a vector of all the individual program source code
    let mut programs: Vec<&str> = Vec::new();

    // Split programs on the $
    let end_program_regex: Regex = Regex::new(r"\$").unwrap();

    // The start index of the current program
    let mut cur_program_start: usize = 0;

    // We need to find all of the $ in the code
    for eop_match in end_program_regex.find_iter(source_code) {
        // Extract the program code and update the start index
        let program: &str = &source_code[cur_program_start..eop_match.end()];
        programs.push(program);
        cur_program_start = eop_match.end();
    }

    return programs;
}