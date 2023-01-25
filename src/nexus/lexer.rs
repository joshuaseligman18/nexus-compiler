use crate::{nexus::token::{Token, TokenType, Keywords, Symbols}, util::nexus_log};
use log::{debug, info, error};
use regex::{Regex, RegexSet};

use super::token;

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

    // The cur token type
    let mut cur_token_type: TokenType = TokenType::Unrecognized(String::from(""));

    // The current position in the source code
    let mut trailer: usize = 0;

    // Initially not in a string
    let mut in_string: bool = false;

    // Initially not in a comment
    let mut in_comment: bool = false;
    let mut comment_position: (usize, usize) = (0, 0);
    let comment_regex: RegexSet = RegexSet::new(&[r"^/\*$", r"^\*/$"]).unwrap();

    // Iterate through the end of the string
    while cur_start < source_code.len() { 
        // If it is the start of a search and we have space for a comment (/* or */)
        if cur_start == trailer && cur_start < source_code.len() - 1 {
            // Get the next 2 characters
            let next_2: &str = &source_code[cur_start..cur_start + 2];
            // If it is a comment symbol
            if comment_regex.is_match(next_2) {
                // Get the updated comment start position
                if !in_comment {
                    comment_position = (line_number, col_number);
                }

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
            if upgrade_token(cur_sub, &mut cur_token_type, &mut in_string) {
                // Move the end to the character after the substring ends
                best_end = trailer + 1;
            }
        } else {
            // Make sure we have something
            if best_end - cur_start > 0 {
                // Create the new token and add it to the stream
                let new_token: Token = Token::new(cur_token_type.to_owned(), source_code[cur_start..best_end].to_string(), line_number, col_number);
                token_stream.push(new_token);

                let new_token_ref: &Token = &token_stream[token_stream.len() - 1];
                match &new_token_ref.token_type {
                    // Log the keyword information
                    TokenType::Keyword(keyword_type) => nexus_log::log(
                        nexus_log::LogTypes::Info,
                        nexus_log::Sources::Lexer,
                        format!("Keyword - {:?} [ {} ] found at position {:?}", keyword_type, new_token_ref.text, new_token_ref.position)
                    ),

                    // Log the identifier information
                    TokenType::Identifier(id) => nexus_log::log(
                        nexus_log::LogTypes::Info, 
                        nexus_log::Sources::Lexer,
                        format!("Identifier [ {} ] found at position {:?}", id, new_token_ref.position)
                    ),
                    
                    // Log the symbol information
                    TokenType::Symbol(symbol_type) => nexus_log::log(
                        nexus_log::LogTypes::Info,
                        nexus_log::Sources::Lexer,
                        format!("Symbol - {:?} [ {} ] found at position {:?}", symbol_type, new_token_ref.text, new_token_ref.position)
                    ),

                    // Log the digit information
                    TokenType::Digit(num) => nexus_log::log(
                        nexus_log::LogTypes::Info,
                        nexus_log::Sources::Lexer,
                        format!("Digit [ {} ] found at position {:?}", num, new_token_ref.position)
                    ),
                    
                    // Log the char information
                    TokenType::Char(char) => nexus_log::log(
                        nexus_log::LogTypes::Info,
                        nexus_log::Sources::Lexer,
                        format!("Char [ {} ] found at position {:?}", char, new_token_ref.position)
                    ),

                    // Unrecognized tokens throw errors
                    TokenType::Unrecognized(_) => nexus_log::log(
                        nexus_log::LogTypes::Error,
                        nexus_log::Sources::Lexer,
                        format!("Error at {:?}; Unrecognized symbol '{}'", new_token_ref.position, new_token_ref.text)
                    ),
                }    

                // Go back to an unrecognized empty token
                cur_token_type = TokenType::Unrecognized(String::from(""));

                // Update the column number to accommodate the length of the token
                col_number += best_end - cur_start;

                // Move the trailer to the best end - 1 (will get incremented at the loop bottom)
                trailer = best_end - 1;
                // Move cur_start to the beginning of the next possible token
                cur_start = trailer + 1;
            } else {
                // Token is empty
                cur_start += 1;
                best_end += 1;

                // New line should update the line and column numbers
                if cur_char.eq("\n") {
                    if in_string {
                        // Get the index of the open quote token by doing a backwards linear search
                        let mut i: i32 = token_stream.len() as i32 - 1;
                        while i >= 0 {
                            match &token_stream[i as usize].token_type {
                                // Can break upon finding the token
                                TokenType::Symbol(Symbols::Quote) => break,
                                _ => i -= 1,
                            };
                        }
                        // The string was not closed, so throw an error
                        nexus_log::log(
                            nexus_log::LogTypes::Error,
                            nexus_log::Sources::Lexer,
                            format!("Unclosed string starting at {:?}", token_stream[i as usize].position)
                        );
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
        nexus_log::log(
            nexus_log::LogTypes::Warning,
            nexus_log::Sources::Lexer,
            format!("Unclosed comment starting at {:?}", comment_position)
        );
    }

    return token_stream;
}

fn upgrade_token(substr: &str, best_token_type: &mut TokenType, in_string: &mut bool) -> bool {
    // Create the keywords
    let keywords: RegexSet = RegexSet::new(&[
        r"^if$",
        r"^while$",
        r"^print$",
        r"^string$",
        r"^int$",
        r"^boolean$",
        r"^true$",
        r"^false$",
    ]).unwrap();

    // Characters are a-z all lowercase and only 1 character
    let characters: Regex = Regex::new(r"^[a-z]$").unwrap();

    // Symbols can be (, ), {, }, ==, =, +, ", or !=
    let symbols: RegexSet = RegexSet::new(&[
        r"^\($",
        r"^\)$",
        r"^\{$",
        r"^\}$",
        r"^\+$",
        r"^==$",
        r"^!=$",
        r"^=$",
        r#"^"$"#,
    ]).unwrap();

    // Digits are 0-9
    let digits: Regex = Regex::new(r"^[0-9]$").unwrap();
    

    // See if we are in a string
    if *in_string {
        // Spaces and characters are valid
        if characters.is_match(substr) || substr.eq(" ") {
            *best_token_type = TokenType::Char(String::from(substr));
            return true;
        } else if substr.eq("\"") {
            // " is the end of the string
            *best_token_type = TokenType::Symbol(Symbols::Quote);
            *in_string = false;
            return true;
        } else if substr.len() == 1 {
            // Invalid token
            *best_token_type = TokenType::Unrecognized(String::from(substr));
            return true;
        }
    } else {
        if keywords.is_match(substr) {
            // Get the possible keyword matches
            let keyword_matches: Vec<usize> = keywords.matches(substr).into_iter().collect();
            if keyword_matches.len() > 0 {
                // The order here matches the order in which they are defined in the constructor
                match keyword_matches[0] {
                    0 => *best_token_type = TokenType::Keyword(Keywords::If),
                    1 => *best_token_type = TokenType::Keyword(Keywords::While),
                    2 => *best_token_type = TokenType::Keyword(Keywords::Print),
                    3 => *best_token_type = TokenType::Keyword(Keywords::String),
                    4 => *best_token_type = TokenType::Keyword(Keywords::Int),
                    5 => *best_token_type = TokenType::Keyword(Keywords::Boolean),
                    6 => *best_token_type = TokenType::Keyword(Keywords::True),
                    7 => *best_token_type = TokenType::Keyword(Keywords::False),
                    // Should never be reached
                    _ => panic!("Invalid regex found for keywords")
                }
                return true;
            }
        } else if characters.is_match(substr) {
            // Otherwise it may be an identifier, digit, symbol, or unrecognized
            // We have an identifier
            *best_token_type = TokenType::Identifier(String::from(substr));
            return true;
        } else if symbols.is_match(substr) {
            // Get the possible symbol matches
            let symbol_matches: Vec<usize> = symbols.matches(substr).into_iter().collect();
            if symbol_matches.len() > 0 {
                // The order here matches the order in which they are defined in the constructor
                match symbol_matches[0] {
                    0 => *best_token_type = TokenType::Symbol(Symbols::LParen),
                    1 => *best_token_type = TokenType::Symbol(Symbols::RParen),
                    2 => *best_token_type = TokenType::Symbol(Symbols::LBrace),
                    3 => *best_token_type = TokenType::Symbol(Symbols::RBrace),
                    4 => *best_token_type = TokenType::Symbol(Symbols::AdditionOp),
                    5 => *best_token_type = TokenType::Symbol(Symbols::EqOp),
                    6 => *best_token_type = TokenType::Symbol(Symbols::NeqOp),
                    7 => *best_token_type = TokenType::Symbol(Symbols::AssignmentOp),
                    8 => {
                        *best_token_type = TokenType::Symbol(Symbols::Quote);
                        *in_string = true;
                    },
                    // Should never be reached
                    _ => panic!("Invalid regex found for symbols")
                }
                return true;
            }
        } else if digits.is_match(substr) {
            // We have a digit
            *best_token_type = TokenType::Digit(substr.parse::<u32>().unwrap());
            return true;
        } else if substr.len() == 1 {
            // We have an unrecognized symbol
            *best_token_type = TokenType::Unrecognized(String::from(substr));
            return true;
        }
    }
    // No upgrade
    return false;
}