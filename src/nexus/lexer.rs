use crate::{nexus::token::Token, util::nexus_log};
use regex::{Regex, RegexSet};

pub fn lex(source_code: String) {//-> Vec<Token> {
    // This represents all possible terminal characters for which to mark the end of the current search
    let terminal_chars = Regex::new(r"\s").unwrap();

    let mut line_number: usize = 1;

    let mut cur_start: usize = 0;
    let mut best_end: usize = 0;

    let mut cur_token: Token = Token::Unrecognized(String::from(""));

    // Iterate through the end of the string
    for mut i in 0..source_code.len() {
        // Get the current character
        let cur_char: &str = &source_code[i..i+1];

        // Check if it is a terminal character
        if !terminal_chars.is_match(cur_char) {
            // Need to check the substring from cur_start
            // Get the current substring in question
            let cur_sub: &str = &source_code[cur_start..i + 1];
            
            if check_for_improvement(cur_sub, &mut cur_token) {
                best_end = i;
            }
        } else {
            if cur_char == "\n" {
                line_number += 1;
            }

            nexus_log::log(String::from("LEXER"), format!("Found {:?} at ({}, {})", cur_token, line_number, cur_start + 1));
            i = best_end + 1;
            cur_start = i;
        }
    }
}

fn check_for_improvement(substr: &str, best_token_type: &mut Token) -> bool {
    // Create the keywords
    let keywords: RegexSet = RegexSet::new(&[
        r"if",
        r"while",
        r"print",
        r"string",
        r"int",
        r"boolean"
    ]).unwrap();

    let identifiers = Regex::new(r"^[a-z]$").unwrap();

    let symbols = Regex::new(r"").unwrap();
    
    match best_token_type {
        // Keyword is the best and they are all mutually exclusive, so no need to check
        Token::Keyword(_) => return false,
        _ => {
            if keywords.is_match(substr) {
                *best_token_type = Token::Keyword(String::from(substr));
                return true;
            } else if identifiers.is_match(substr) {
                *best_token_type = Token::Identifier(String::from(substr));
                return true;
            } else {
                return false;
            }
        }
    }       
}