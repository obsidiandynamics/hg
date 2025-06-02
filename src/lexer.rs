use std::collections::VecDeque;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::str::FromStr;
use crate::token::{Location, Token};

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

pub fn tokenise<R: Read>(mut reader: BufReader<R>) -> Result<VecDeque<Token>, Error> {
    let mut tokens = VecDeque::new();
    let mut line = String::new();
    let mut token = String::new();
    let mut mode = Mode::Whitespace;
    let mut location = Location::default();
    let mut bytes;
    loop {
        bytes = reader.read_line(&mut line)?;
        location.line += 1;

        let chars = if bytes != 0 {
            line.chars()
        } else {
            "\n".chars()
        };

        println!("line {}: '{line}'", location.line);
        for char in chars {
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
                            }
                            '(' => {
                                tokens.push_back(Token::LeftParen);
                            }
                            ')' => {
                                tokens.push_back(Token::RightParen);
                            }
                            '{' => {
                                tokens.push_back(Token::LeftBrace);
                            }
                            '}' => {
                                tokens.push_back(Token::RightBrace);
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
                                token.push(char);
                            }
                            '.' => {
                                mode = Mode::Decimal(0);
                            }
                            _ => {
                                mode = Mode::Ident;
                                token.push(char);
                            }
                        }
                    }
                    Mode::TextBody => {
                        match char {
                            '\\' => {
                                mode = Mode::TextEscape;
                            }
                            '"' => {
                                tokens.push_back(Token::Text(token.clone()));
                                token.clear();
                                mode = Mode::Whitespace;
                            }
                            '\n' => {
                                return Err(Error::UnterminatedLiteral(location))
                            }
                            _ => {
                                token.push(char);
                            }
                        }
                    }
                    Mode::CharacterEscape | Mode::TextEscape => {
                        match char {
                            '\\' | '"' | '\'' => {
                                token.push(char);
                            }
                            'n' => {
                                token.push('\n');
                            }
                            'r' => {
                                token.push('\r');
                            }
                            't' => {
                                token.push('\t');
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
                            }
                            '\'' => {
                                let mut chars = token.chars();
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
                                    token.push(char);
                                } else {
                                    return Err(Error::UnexpectedCharacter(char, location))
                                }
                            }
                        }
                    }
                    Mode::Integer => {
                        match char {
                            '_' => {}
                            '.' => {
                                match u128::from_str(&token) {
                                    Ok(int) => {
                                        mode = Mode::Decimal(int);
                                        token.clear()
                                    }
                                    Err(err) => {
                                        return Err(Error::UnparsableInteger(token.clone(), err, location))
                                    }
                                }
                            }
                            ')' | '}' | ':' | '-' | '\n' | '\t' | '\r' | ' ' => {
                                match u128::from_str(&token) {
                                    Ok(whole) => {
                                        tokens.push_back(Token::Integer(whole));
                                        token.clear();
                                        mode = Mode::Whitespace;
                                        continue 'matcher; // don't consume the char
                                    }
                                    Err(err) => {
                                        return Err(Error::UnparsableInteger(token.clone(), err, location))
                                    }
                                }
                            }
                            _ => {
                                token.push(char);
                            }
                        }
                    }
                    Mode::Decimal(whole) => {
                        match char {
                            '_' => {}
                            ')' | '}' | ':' | '-' | '\n' | '\t' | '\r' | ' ' => {
                                match u128::from_str(&token) {
                                    Ok(fractional) => {
                                        tokens.push_back(Token::Decimal(whole, fractional, token.len().try_into().expect("fractional part is too long")));
                                        token.clear();
                                        mode = Mode::Whitespace;
                                        continue 'matcher;  // don't consume the char
                                    }
                                    Err(err) => {
                                        return Err(Error::UnparsableDecimal(whole, token.clone(), err, location))
                                    }
                                }
                            }
                            _ => {
                                token.push(char)
                            }
                        }
                    }
                    Mode::Ident => {
                        match char {
                            ')' | '}' | ':' | '-' | '\n' | '\t' | '\r' | ' ' => {
                                match token.as_str() {
                                    "true" => {
                                        tokens.push_back(Token::Boolean(true));
                                    }
                                    "false" => {
                                        tokens.push_back(Token::Boolean(false));
                                    }
                                    _ => {
                                        tokens.push_back(Token::Ident(token.clone()));
                                    }
                                }
                                token.clear();
                                mode = Mode::Whitespace;
                                continue 'matcher;  // don't consume the char
                            }
                            _ => {
                                token.push(char);
                            }
                        }
                    }
                }
                break 'matcher;
            }
        }

        if bytes == 0 {
            break;
        } else {
            location.column = 0;
            line.clear();
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests;