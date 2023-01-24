use crate::{nexus::token::Token, util::nexus_log};
use log::{debug, info, error};
use regex::{Regex, RegexSet};

pub fn lex(source_code: String) {//-> Vec<Token> {
    // This represents all possible terminal characters for which to mark the end of the current search
    let terminal_chars = Regex::new(r"^\s$").unwrap();

    // The line and column numbers in the file
    let mut line_number: usize = 1;
    let mut col_number: usize = 1;

    // The start and end indices in the source code string for the token
    // cur_start == best_end means that the token is empty (space or newline by itself)
    let mut cur_start: usize = 0;
    let mut best_end: usize = 0;

    // The cur token (implementation may change)
    let mut cur_token: Token = Token::Unrecognized(String::from(""));

    // The current position in the source code
    let mut trailer: usize = 0;

    // Initially not in a string
    let mut in_string: bool = false;

    // Iterate through the end of the string
    while cur_start < source_code.len() {
        debug!("{}", format!("trailer: {}, cur_start: {}, best_end: {}", trailer, cur_start, best_end));

        // Get the current character if legal
        let mut cur_char: &str = "";
        if trailer < source_code.len() {
            cur_char = &source_code[trailer..trailer + 1];
        }

        // Check if it is a terminal character
        if !cur_char.is_empty() && (!terminal_chars.is_match(cur_char) || in_string) {
            // Need to check the substring from cur_start
            // Get the current substring in question
            let cur_sub: &str = &source_code[cur_start..trailer + 1];
            
            if upgrade_token(cur_sub, &mut cur_token, &mut in_string) {
                best_end = trailer + 1;
            }
        } else {
            if best_end - cur_start > 0 {
                match cur_token {
                    Token::Unrecognized(_) => nexus_log::error(String::from("LEXER"), format!("{:?} at ({}, {})", cur_token, line_number, col_number)),
                    _ => nexus_log::info(String::from("LEXER"), format!("{:?} at ({}, {})", cur_token, line_number, col_number)),
                }

                // nexus_log::info(String::from("LEXER"), format!("{:?} at ({}, {})", cur_token, line_number, col_number));

                // Update the column number to accommodate the length of the token
                col_number += best_end - cur_start;

                // Move the trailer to the best end - 1 (will get incremented at the loop bottom)
                trailer = best_end - 1;
                // Move cur_start to the beginning of the next possible token
                cur_start = trailer + 1;

                cur_token = Token::Unrecognized(String::from(""));
            } else {
                // Token is empty
                cur_start += 1;
                best_end += 1;

                // New line should update the line and column numbers
                if cur_char.eq("\n") {
                    line_number += 1;
                    col_number = 1;
                } else {
                    col_number += 1;
                }
            }
        }

        trailer += 1;
    }
}

fn upgrade_token(substr: &str, best_token_type: &mut Token, in_string: &mut bool) -> bool {
    // Create the keywords
    let keywords: RegexSet = RegexSet::new(&[
        r"^if$",
        r"^while$",
        r"^print$",
        r"^string$",
        r"^int$",
        r"^boolean$"
    ]).unwrap();

    // Characters are a-z all lowercase and only 1 character
    let characters: Regex = Regex::new(r"^[a-z]$").unwrap();

    // Symbols can be (, ), {, }, =, +, ", or !
    let symbols: Regex = Regex::new(r#"^[\(\){}=\+"!]$"#).unwrap();

    // Digits are 0-9
    let digits: Regex = Regex::new(r"^[0-9]$").unwrap();
    

    // See if we are in a string
    if *in_string {
        // Spaces and characters are valid
        if characters.is_match(substr) || substr.eq(" ") {
            *best_token_type = Token::Char(String::from(substr));
            return true;
        } else if substr.eq("\"") {
            // " is the end of the string
            *best_token_type = Token::Symbol(String::from(substr));
            *in_string = false;
            return true;
        } else if substr.len() == 1 {
            // Invalid token
            *best_token_type = Token::Unrecognized(String::from(substr));
            return true;
        } else {
            // No upgrade
            return false;
        }
    } else {
        match best_token_type {
            // Keyword is the best and they are all mutually exclusive, so no need to check
            Token::Keyword(_) => return false,
            _ => {
                if keywords.is_match(substr) {
                    *best_token_type = Token::Keyword(String::from(substr));
                    return true;
                } else if characters.is_match(substr) {
                    *best_token_type = Token::Identifier(String::from(substr));
                    return true;
                } else if symbols.is_match(substr) {
                    *best_token_type = Token::Symbol(String::from(substr));
                    // We found the start of a string
                    if substr.eq("\"") {
                        *in_string = true;
                    }
                    return true;
                }  else if digits.is_match(substr) {
                    *best_token_type = Token::Digit(String::from(substr));
                    return true;
                } else {
                    return false;
                }
            }
        }
    }       
}