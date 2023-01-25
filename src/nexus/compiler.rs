use log::info;

use crate::util::nexus_log;
use crate::nexus::{lexer, token::Token};

pub fn compile(source_code: String) {
    nexus_log::clear_logs();
    nexus_log::log(
        nexus_log::LogTypes::Info,
        nexus_log::Sources::Nexus,
        String::from("Compile called")
    );
    let lex_out: Result<(Vec<Token>, i32), (i32, i32)> = lexer::lex(&source_code);
    if lex_out.is_ok() {
        // Grab the token stream and number of warnings
        let (mut token_stream, num_warnings): (Vec<Token>, i32) = lex_out.unwrap();

        // Create the output string and log it
        let mut out_string: String = format!("Lexer completed with 0 errors and {} warning", num_warnings);
        if num_warnings == 1 {
            out_string.push_str(".");    
        } else {
            out_string.push_str("s.");
        }
        nexus_log::log(
            nexus_log::LogTypes::Info,
            nexus_log::Sources::Lexer,
            out_string
        );
        info!("{:?}", token_stream);
    } else {
        // Get the number of errors and warnings
        let (num_errors, num_warnings): (i32, i32) = lex_out.unwrap_err();

        // Generate the output string
        let mut out_string: String = format!("Lexer failed with {} error", num_errors);
        if num_errors == 1 {
            out_string.push_str(" and ");
        } else {
            out_string.push_str("s and ");
        }

        out_string.push_str(format!("{} warning", num_warnings).as_str());
        if num_warnings == 1 {
            out_string.push_str("");    
        } else {
            out_string.push_str("s.");
        }

        // Log the output string
        nexus_log::log(
            nexus_log::LogTypes::Error,
            nexus_log::Sources::Lexer,
            out_string
        );
    }
}