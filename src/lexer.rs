use crate::char_buffer::CharBuffer;
use crate::newline_terminated_chars::NewlineTerminatedChars;
use crate::token::{ListDelimiter, Location, Token};
use std::collections::VecDeque;
use std::io;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error {0}")]
    Io(#[from] io::Error),

    #[error("unexpected character '{0}' at {1}")]
    UnexpectedCharacter(char, Location),

    #[error("unterminated literal at {0}")]
    UnterminatedLiteral(Location),

    #[error("unknown escape sequence '{0}' at {1}")]
    UnknownEscapeSequence(char, Location),

    #[error("unparsable integer {0} ({1}) at {2}")]
    UnparsableInteger(String, ParseIntError, Location),

    #[error("unparsable decimal {0}.{1} ({2}) at {3}")]
    UnparsableDecimal(u128, String, ParseIntError, Location),

    #[error("empty character literal at {0}")]
    EmptyCharacterLiteral(Location),
}

enum Mode {
    Whitespace,
    TextBody,
    TextEscape,
    CharacterBody,
    CharacterEscape,
    Integer,
    Decimal(u128),
    Ident
}

pub fn tokenise(str: &str) -> Result<VecDeque<Token>, Error> {
    let bytes = str.as_bytes();
    let char_indexes = NewlineTerminatedChars::new(str.char_indices());
    
    let mut tokens = VecDeque::new();
    let mut token = CharBuffer::default();
    let mut mode = Mode::Whitespace;
    let mut location = Location { line: 1, column: 0 };
    for (index, char) in char_indexes {
        //println!("{char} at index {index}");
        location.column += 1;
        'matcher: loop {
            match mode {
                Mode::Whitespace => {
                    match char {
                        '\\' => {
                            return Err(Error::UnexpectedCharacter(char, location))
                        }
                        '"' => {
                            mode = Mode::TextBody;
                        }
                        '\'' => {
                            mode = Mode::CharacterBody;
                        }
                        '\t' | '\r' | ' ' => {}
                        '\n' => {
                            tokens.push_back(Token::Newline);
                            location.line += 1;
                            location.column = 1;
                        }
                        '(' => {
                            tokens.push_back(Token::Left(ListDelimiter::Paren));
                        }
                        ')' => {
                            tokens.push_back(Token::Right(ListDelimiter::Paren));
                        }
                        '{' => {
                            tokens.push_back(Token::Left(ListDelimiter::Brace));
                        }
                        '}' => {
                            tokens.push_back(Token::Right(ListDelimiter::Brace));
                        }
                        '[' => {
                            tokens.push_back(Token::Left(ListDelimiter::Bracket));
                        }
                        ']' => {
                            tokens.push_back(Token::Right(ListDelimiter::Bracket));
                        }
                        '-' => {
                            tokens.push_back(Token::Dash);
                        }
                        ':' => {
                            tokens.push_back(Token::Colon);
                        }
                        ',' => {
                            tokens.push_back(Token::Comma);
                        }
                        '0'..='9' => {
                            mode = Mode::Integer;
                            token.push(index, char);
                        }
                        '.' => {
                            mode = Mode::Decimal(0);
                        }
                        _ => {
                            mode = Mode::Ident;
                            token.push(index, char);
                        }
                    }
                }
                Mode::TextBody => {
                    match char {
                        '\\' => {
                            token.copy(bytes);
                            mode = Mode::TextEscape;
                        }
                        '"' => {
                            tokens.push_back(Token::Text(token.string(bytes).to_string())); //TODO
                            token.clear();
                            mode = Mode::Whitespace;
                        }
                        '\n' => {
                            return Err(Error::UnterminatedLiteral(location))
                        }
                        _ => {
                            token.push(index, char);
                        }
                    }
                }
                Mode::CharacterEscape | Mode::TextEscape => {
                    match char {
                        '\\' | '"' | '\'' => {
                            token.push(0, char);
                        }
                        'n' => {
                            token.push(0, '\n');
                        }
                        'r' => {
                            token.push(0, '\r');
                        }
                        't' => {
                            token.push(0, '\t');
                        }
                        'x' => {
                            //TODO handle hex (e.g., \x7F)
                            return Err(Error::UnknownEscapeSequence(char, location))
                        }
                        'u' => {
                            //TODO handle unicode (e.g., \u{7FFF})
                            return Err(Error::UnknownEscapeSequence(char, location))
                        }
                        _ => {
                            return Err(Error::UnknownEscapeSequence(char, location))
                        }
                    }
                    match mode {
                        Mode::TextEscape => {
                            mode = Mode::TextBody;
                        }
                        Mode::CharacterEscape => {
                            mode = Mode::CharacterBody;
                        }
                        _ => {
                            // by the encompassing pattern, mode must be one of the two variants above
                            unreachable!()
                        }
                    }
                }
                Mode::CharacterBody => {
                    match char {
                        '\\' => {
                            mode = Mode::CharacterEscape;
                            token.copy(bytes);
                        }
                        '\'' => {
                            let mut chars = token.as_str(bytes).chars();
                            match chars.next() {
                                None => {
                                    return Err(Error::EmptyCharacterLiteral(location))
                                }
                                Some(first_char) => {
                                    tokens.push_back(Token::Character(first_char));
                                    token.clear();
                                    mode = Mode::Whitespace;
                                }
                            }
                        }
                        '\n' => {
                            return Err(Error::UnterminatedLiteral(location))
                        }
                        _ => {
                            if token.is_empty() {
                                token.push(index, char);
                            } else {
                                return Err(Error::UnexpectedCharacter(char, location))
                            }
                        }
                    }
                }
                Mode::Integer => {
                    match char {
                        '_' => {
                            token.copy(bytes);
                        }
                        '.' => {
                            let str = token.as_str(bytes);
                            match u128::from_str(str) {
                                Ok(int) => {
                                    mode = Mode::Decimal(int);
                                    token.clear()
                                }
                                Err(err) => {
                                    return Err(Error::UnparsableInteger(str.to_string(), err, location))
                                }
                            }
                        }
                        ')' | '}' | ':' | '-' | ',' | '\n' | '\t' | '\r' | ' ' => {
                            let str = token.as_str(bytes);
                            match u128::from_str(str) {
                                Ok(whole) => {
                                    tokens.push_back(Token::Integer(whole));
                                    token.clear();
                                    mode = Mode::Whitespace;
                                    continue 'matcher; // don't consume the char
                                }
                                Err(err) => {
                                    return Err(Error::UnparsableInteger(str.to_string(), err, location))
                                }
                            }
                        }
                        _ => {
                            token.push(index, char);
                        }
                    }
                }
                Mode::Decimal(whole) => {
                    match char {
                        '_' => {
                            token.copy(bytes);
                        }
                        ')' | '}' | ':' | '-' | ',' | '\n' | '\t' | '\r' | ' ' => {
                            let str = token.as_str(bytes);
                            match u128::from_str(str) {
                                Ok(fractional) => {
                                    tokens.push_back(Token::Decimal(whole, fractional, token.len().try_into().expect("fractional part is too long")));
                                    token.clear();
                                    mode = Mode::Whitespace;
                                    continue 'matcher;  // don't consume the char
                                }
                                Err(err) => {
                                    return Err(Error::UnparsableDecimal(whole, str.to_string(), err, location))
                                }
                            }
                        }
                        _ => {
                            token.push(index, char);
                        }
                    }
                }
                Mode::Ident => {
                    match char {
                        ')' | '}' | ':' | '-'  | ',' | '\n' | '\t' | '\r' | ' ' => {
                            let str = token.as_str(bytes);
                            match str {
                                "true" => {
                                    tokens.push_back(Token::Boolean(true));
                                }
                                "false" => {
                                    tokens.push_back(Token::Boolean(false));
                                }
                                _ => {
                                    tokens.push_back(Token::Ident(str.to_string())); //TODO
                                }
                            }
                            token.clear();
                            mode = Mode::Whitespace;
                            continue 'matcher;  // don't consume the char
                        }
                        _ => {
                            token.push(index, char);
                        }
                    }
                }
            }
            break 'matcher;
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests;