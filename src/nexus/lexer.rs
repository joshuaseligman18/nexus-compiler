use crate::{nexus::token::{Token, TokenType, Keywords, Symbols}, util::nexus_log};
use log::{debug, info, error};
use regex::{Regex, RegexSet};

// Struct to maintain the state of the line numbers when compiling multiple programs
pub struct Lexer {
    line_number: usize,
    col_number: usize
}

impl Lexer {
    // Creates the new lexer and initializes the starting position to be (1, 1)
    pub fn new() -> Self {
        return Lexer {
            line_number: 1,
            col_number: 1
        }
    }

    // Function to lex a program
    pub fn lex_program(&mut self, source_code: &str, starting_position: &mut usize) -> Result<Vec<Token>, ()> {
        let lex_out: Result<(Vec<Token>, i32), (i32, i32)> = self.lex(source_code, starting_position);
        if lex_out.is_ok() {
            // Grab the token stream and number of warnings
            let (token_stream, num_warnings): (Vec<Token>, i32) = lex_out.unwrap();

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

            // Return the token stream
            return Ok(token_stream);
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

            // Nothing has to be returned so just let the compiler know it failed
            return Err(());
        }
    }

    // Function to lex a program
    // Ok result: (token stream, number of warnings)
    // Err result: (number of errors, number of warnings)
    fn lex(&mut self, source_code: &str, starting_position: &mut usize) -> Result<(Vec<Token>, i32), (i32, i32)> {
        // Initialize the number of errors and warnings to 0
        let mut num_errors: i32 = 0;
        let mut num_warnings: i32 = 0;
        
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

        // The start and end indices in the source code string for the token
        // starting_position == best_end means that the token is empty (space or newline by itself)
        let mut best_end: usize = starting_position.to_owned();

        // The cur token type
        let mut cur_token_type: TokenType = TokenType::Unrecognized(String::from(""));

        // The current position in the source code
        let mut trailer: usize = starting_position.to_owned();

        // Initially not in a string
        let mut in_string: bool = false;

        // Initially not in a comment
        let mut in_comment: bool = false;
        let mut comment_position: (usize, usize) = (0, 0);
        let comment_regex: RegexSet = RegexSet::new(&[r"^/\*$", r"^\*/$"]).unwrap();

        let mut end_found: bool = false;

        // Iterate through the end of the string
        while !end_found && *starting_position < source_code.len() { 
            // If it is the start of a search and we have space for a comment (/* or */)
            if *starting_position == trailer && *starting_position < source_code.len() - 1 {
                // Get the next 2 characters
                let next_2: &str = &source_code[*starting_position..*starting_position + 2];

                let comment_matches = comment_regex.matches(next_2);
                // If it is a comment symbol
                if !in_comment && comment_matches.matched(0) || in_comment && comment_matches.matched(1) {
                    // Get the updated comment start position
                    if !in_comment {
                        comment_position = (self.line_number, self.col_number);
                    }

                    // Flip and skip both characters
                    in_comment = !in_comment;
                    *starting_position += 2;
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
                // Need to check the substring from starting_position
                // Get the current substring in question
                let cur_sub: &str = &source_code[*starting_position..trailer + 1];
                
                // Check to see if we need to upgrade the token
                if self.upgrade_token(cur_sub, &mut cur_token_type, &mut in_string) {
                    // Move the end to the character after the substring ends
                    best_end = trailer + 1;
                }
            } else {
                // Make sure we have something
                if best_end - *starting_position > 0 {
                    // Create the new token and add it to the stream
                    let new_token: Token = Token::new(cur_token_type.to_owned(), source_code[*starting_position..best_end].to_string(), self.line_number, self.col_number);
                    token_stream.push(new_token);

                    let new_token_ref: &Token = &token_stream[token_stream.len() - 1];
                    match &new_token_ref.token_type {
                        // Log the keyword information
                        TokenType::Keyword(keyword_type) => nexus_log::log(
                            nexus_log::LogTypes::Debug,
                            nexus_log::Sources::Lexer,
                            format!("Keyword - {:?} [ {} ] found at {:?}", keyword_type, new_token_ref.text, new_token_ref.position)
                        ),

                        // Log the identifier information
                        TokenType::Identifier(id) => nexus_log::log(
                            nexus_log::LogTypes::Debug, 
                            nexus_log::Sources::Lexer,
                            format!("Identifier [ {} ] found at {:?}", id, new_token_ref.position)
                        ),
                        
                        // Log the symbol information
                        TokenType::Symbol(symbol_type) => {
                            nexus_log::log(
                                nexus_log::LogTypes::Debug,
                                nexus_log::Sources::Lexer,
                                format!("Symbol - {:?} [ {} ] found at {:?}", symbol_type, new_token_ref.text, new_token_ref.position)
                            );

                            // Mark the end found if needed
                            match symbol_type {
                                Symbols::EOP => end_found = true,
                                _ => {}
                            }
                        },

                        // Log the digit information
                        TokenType::Digit(num) => nexus_log::log(
                            nexus_log::LogTypes::Debug,
                            nexus_log::Sources::Lexer,
                            format!("Digit [ {} ] found at {:?}", num, new_token_ref.position)
                        ),
                        
                        // Log the char information
                        TokenType::Char(char) => nexus_log::log(
                            nexus_log::LogTypes::Debug,
                            nexus_log::Sources::Lexer,
                            format!("Char [ {} ] found at {:?}", char, new_token_ref.position)
                        ),

                        // Unrecognized tokens throw errors
                        TokenType::Unrecognized(_) => {
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
                                nexus_log::log(
                                    nexus_log::LogTypes::Error,
                                    nexus_log::Sources::Lexer,
                                    format!("Error at {:?}; Unrecognized token '{}' in string starting at {:?}; Strings may only contain lowercase letters (a - z)", new_token_ref.position, new_token_ref.text, token_stream[i as usize].position)
                                )
                            } else {
                                nexus_log::log(
                                    nexus_log::LogTypes::Error,
                                    nexus_log::Sources::Lexer,
                                    format!("Error at {:?}; Unrecognized token '{}'", new_token_ref.position, new_token_ref.text)
                                );
                            }
                            num_errors += 1;
                        },
                    }

                    // Go back to an unrecognized empty token
                    cur_token_type = TokenType::Unrecognized(String::from(""));

                    // Update the column number to accommodate the length of the token
                    self.col_number += best_end - *starting_position;

                    // Move the trailer to the best end - 1 (will get incremented at the loop bottom)
                    trailer = best_end - 1;
                    // Move starting_position to the beginning of the next possible token
                    *starting_position = trailer + 1;
                } else {
                    // Token is empty
                    *starting_position += 1;
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
                            num_errors += 1;

                            // Will finish lexing, so reset in_string
                            in_string = false;
                        }
                        self.line_number += 1;
                        self.col_number = 1;
                    } else {
                        self.col_number += 1;
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
            num_warnings += 1;
        }

        // Check for the $ at the end of the program
        if token_stream.len() > 0 {
            match &token_stream[token_stream.len() - 1].token_type {
                // We are good if we have EOP
                TokenType::Symbol(Symbols::EOP) => {},
                // Otherwise log out the warning
                _ => {
                    nexus_log::log(
                        nexus_log::LogTypes::Warning,
                        nexus_log::Sources::Lexer,
                        String::from("Program did not end with EOP symbol [ $ ]")
                    );
                    num_warnings += 1;
                }
            }
        } else {
            // Empty programs by definition have no tokens and, thus, no EOP token
            nexus_log::log(
                nexus_log::LogTypes::Warning,
                nexus_log::Sources::Lexer,
                String::from("Program did not end with EOP symbol [ $ ]")
            );
            num_warnings += 1;
        }

        if num_errors == 0 {
            // Return the token stream and number of warnings if no errors
            return Ok((token_stream, num_warnings));
        } else {
            // Otherwise, we failed and should inform the user on the return of this function
            return Err((num_errors, num_warnings));
        }
    }

    // Function to upgrade a token based on new information
    fn upgrade_token(&self, substr: &str, best_token_type: &mut TokenType, in_string: &mut bool) -> bool {
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
            r"^\$$"
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
                        9 => *best_token_type = TokenType::Symbol(Symbols::EOP),
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
}