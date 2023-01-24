use crate::{nexus::token::Token, util::nexus_log};
use log::{debug, info, error};
use regex::{Regex, RegexSet};

pub fn lex(source_code: &str) -> Vec<Token> {
    
    // This represents all possible terminal characters for which to mark the end of the current search
    let terminal_chars = Regex::new(r"^\s$").unwrap();

    // Worst case is that we have source_code length minus amount of whitespace number of tokens, so allocate that much space to prevent copying of the vector
    let mut char_count: usize = 0;
    for i in 0..source_code.len() {
        if !terminal_chars.is_match(&source_code[i..i+1]) {
            char_count += 1;
        }
    }
    let mut token_stream: Vec<Token> = Vec::with_capacity(char_count);
    
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

    // Initially not in a comment
    let mut in_comment: bool = false;
    let comment_regex: RegexSet = RegexSet::new(&[r"^/\*$", r"^\*/$"]).unwrap();

    // Iterate through the end of the string
    while cur_start < source_code.len() { 
        // If it is the start of a search and we have space for a comment (/* or */)
        if cur_start == trailer && cur_start < source_code.len() - 1 {
            // Get the next 2 characters
            let next_2: &str = &source_code[cur_start..cur_start + 2];
            // If it is a comment symbol
            if comment_regex.is_match(next_2) {
                // Flip and skip both characters
                in_comment = !in_comment;
                cur_start += 2;
                best_end += 2;
                trailer += 2;
            }
        }
        
        // Get the current character if legal
        let mut cur_char: &str = "";
        if trailer < source_code.len() {
            cur_char = &source_code[trailer..trailer + 1];
        }

        // Check if it is a terminal character or if we are in string and the character is not a \n
        // If \n when in string, then we have an unclosed string and should throw an error in the else block
        if !in_comment && !cur_char.is_empty() && (!terminal_chars.is_match(cur_char) || (in_string && !cur_char.eq("\n"))) {
            // Need to check the substring from cur_start
            // Get the current substring in question
            let cur_sub: &str = &source_code[cur_start..trailer + 1];
            
            // Check to see if we need to upgrade the token
            if upgrade_token(cur_sub, &mut cur_token, &mut in_string) {
                // Move the end to the character after the substring ends
                best_end = trailer + 1;
            }
        } else {
            // Make sure we have something
            if best_end - cur_start > 0 {
                match cur_token {
                    // Unrecognized tokens throw errors
                    Token::Unrecognized(_) => nexus_log::error(String::from("LEXER"), format!("{:?} at ({}, {})", cur_token, line_number, col_number)),
                    // Everything else is valid and is printed out
                    _ => nexus_log::info(String::from("LEXER"), format!("{:?} at ({}, {})", cur_token, line_number, col_number)),
                }

                // Update the column number to accommodate the length of the token
                col_number += best_end - cur_start;

                // Move the trailer to the best end - 1 (will get incremented at the loop bottom)
                trailer = best_end - 1;
                // Move cur_start to the beginning of the next possible token
                cur_start = trailer + 1;

                token_stream.push(cur_token.to_owned());

                // Go back to an unrecognized empty token
                cur_token = Token::Unrecognized(String::from(""));
            } else {
                // Token is empty
                cur_start += 1;
                best_end += 1;

                // New line should update the line and column numbers
                if cur_char.eq("\n") {
                    if in_string {
                        // The string was not closed, so throw an error
                        nexus_log::error(String::from("LEXER"), String::from("Unclosed string"));
                        // Will finish lexing, so reset in_string
                        in_string = false;
                    }
                    line_number += 1;
                    col_number = 1;
                } else {
                    col_number += 1;
                }
            }
        }

        trailer += 1;
    }

    // If comment is still open at end of program, the user should be warned
    if in_comment {
        nexus_log::warning(String::from("LEXER"), String::from("Unclosed comment"));
    }

    return token_stream;
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
        }
    } else {
        if substr.len() > 1 {
            // Being longer than 1 means we may have a keyword
            if keywords.is_match(substr) {
                // We have a keyword
                *best_token_type = Token::Keyword(String::from(substr));
                return true;
            } 
        } else {
            // Otherwise it may be an identifier, digit, symbol, or unrecognized
            if characters.is_match(substr) {
                // We have an identifier
                *best_token_type = Token::Identifier(String::from(substr));
            } else if symbols.is_match(substr) {
                // We have a symbol
                *best_token_type = Token::Symbol(String::from(substr));
                // We found the start of a string
                if substr.eq("\"") {
                    *in_string = true;
                }
            } else if digits.is_match(substr) {
                // We have a digit
                *best_token_type = Token::Digit(String::from(substr));
            } else {
                // We have an unrecognized symbol
                *best_token_type = Token::Unrecognized(String::from(substr));
            }
            // Length of 1 always generates a token of some kind
            return true;
        }
    }
    // No upgrade
    return false;
}