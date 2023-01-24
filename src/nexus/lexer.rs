use crate::{nexus::token::{Token, Keywords, Symbols}, util::nexus_log};
use log::{debug, info, error};
use regex::{Regex, RegexSet, SetMatches};

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
            *best_token_type = Token::Char(String::from(substr));
            return true;
        } else if substr.eq("\"") {
            // " is the end of the string
            *best_token_type = Token::Symbol(Symbols::Quote);
            *in_string = false;
            return true;
        } else if substr.len() == 1 {
            // Invalid token
            *best_token_type = Token::Unrecognized(String::from(substr));
            return true;
        }
    } else {
        if keywords.is_match(substr) {
            // Get the possible keyword matches
            let keyword_matches: Vec<usize> = keywords.matches(substr).into_iter().collect();
            debug!("{:?}", keyword_matches);
            if keyword_matches.len() > 0 {
                // The order here matches the order in which they are defined in the constructor
                match keyword_matches[0] {
                    0 => *best_token_type = Token::Keyword(Keywords::If),
                    1 => *best_token_type = Token::Keyword(Keywords::While),
                    2 => *best_token_type = Token::Keyword(Keywords::Print),
                    3 => *best_token_type = Token::Keyword(Keywords::String),
                    4 => *best_token_type = Token::Keyword(Keywords::Int),
                    5 => *best_token_type = Token::Keyword(Keywords::Boolean),
                    6 => *best_token_type = Token::Keyword(Keywords::True),
                    7 => *best_token_type = Token::Keyword(Keywords::False),
                    // Should never be reached
                    _ => panic!("Invalid regex found for keywords")
                }
                return true;
            }
        } else if characters.is_match(substr) {
            // Otherwise it may be an identifier, digit, symbol, or unrecognized
            // We have an identifier
            *best_token_type = Token::Identifier(String::from(substr));
            return true;
        } else if symbols.is_match(substr) {
            // Get the possible symbol matches
            let symbol_matches: Vec<usize> = symbols.matches(substr).into_iter().collect();
            debug!("{:?}", symbol_matches);
            if symbol_matches.len() > 0 {
                // The order here matches the order in which they are defined in the constructor
                match symbol_matches[0] {
                    0 => *best_token_type = Token::Symbol(Symbols::L_Paren),
                    1 => *best_token_type = Token::Symbol(Symbols::R_Paren),
                    2 => *best_token_type = Token::Symbol(Symbols::L_Brace),
                    3 => *best_token_type = Token::Symbol(Symbols::R_Brace),
                    4 => *best_token_type = Token::Symbol(Symbols::Addition_Op),
                    5 => *best_token_type = Token::Symbol(Symbols::Eq_Op),
                    6 => *best_token_type = Token::Symbol(Symbols::Neq_Op),
                    7 => *best_token_type = Token::Symbol(Symbols::Assignment_Op),
                    8 => {
                        *best_token_type = Token::Symbol(Symbols::Quote);
                        *in_string = true;
                    },
                    // Should never be reached
                    _ => panic!("Invalid regex found for symbols")
                }
                return true;
            }
        } else if digits.is_match(substr) {
            // We have a digit
            *best_token_type = Token::Digit(substr.parse::<u32>().unwrap());
            return true;
        } else if substr.len() == 1 {
            // We have an unrecognized symbol
            *best_token_type = Token::Unrecognized(String::from(substr));
            return true;
        }
    }
    // No upgrade
    return false;
}