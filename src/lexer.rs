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
    UnexpectedChar(char, Location),

    #[error("unterminated text literal at {0}")]
    UnterminatedText(Location),

    #[error("unknown escape sequence '{0}' at {1}")]
    UnknownEscapeSequence(char, Location),

    #[error("unparsable integer {0} ({1}) at {2}")]
    UnparsableInteger(String, ParseIntError, Location),

    #[error("unparsable decimal {0}.{1} ({2}) at {3}")]
    UnparsableDecimal(u128, String, ParseIntError, Location),
}

enum Mode {
    Whitespace,
    TextBody,
    TextEscape,
    Integer,
    Decimal(u128)
}

pub fn tokenise<R: Read>(mut reader: BufReader<R>) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
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
                                return Err(Error::UnexpectedChar(char, location))
                            }
                            '"' => {
                                mode = Mode::TextBody
                            }
                            '\t' | '\r' | ' ' => {}
                            '\n' => {
                                tokens.push(Token::Newline)
                            }
                            '(' => {
                                tokens.push(Token::LeftParen)
                            }
                            ')' => {
                                tokens.push(Token::RightParen)
                            }
                            '{' => {
                                tokens.push(Token::LeftBrace)
                            }
                            '}' => {
                                tokens.push(Token::RightBrace)
                            }
                            '-' => {
                                tokens.push(Token::Dash)
                            }
                            ':' => {
                                tokens.push(Token::Colon)
                            }
                            '0'..='9' => {
                                mode = Mode::Integer;
                                token.push(char)
                            }
                            '.' => {
                                mode = Mode::Decimal(0);
                            }
                            _ => {
                                //TODO start ident
                            }
                        }
                    }
                    Mode::TextBody => {
                        match char {
                            '\\' => {
                                mode = Mode::TextEscape
                            }
                            '"' => {
                                tokens.push(Token::Text(token.clone()));
                                token.clear();
                                mode = Mode::Whitespace
                            }
                            '\n' => {
                                return Err(Error::UnterminatedText(location))
                            }
                            _ => {
                                token.push(char)
                            }
                        }
                    }
                    Mode::TextEscape => {
                        match char {
                            '\\' | '"' => {
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
                        mode = Mode::TextBody
                    }
                    Mode::Integer => {
                        match char {
                            // '0'..='9' => {
                            //     token.push(char)
                            // }
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
                            ')' | '}' | ':' | '\n' | '\t' | '\r' | ' ' => {
                                match u128::from_str(&token) {
                                    Ok(whole) => {
                                        tokens.push(Token::Integer(whole));
                                        token.clear();
                                        mode = Mode::Whitespace;
                                        continue 'matcher;
                                    }
                                    Err(err) => {
                                        return Err(Error::UnparsableInteger(token.clone(), err, location))
                                    }
                                }
                            }
                            _ => {
                                token.push(char);
                                // let err = u128::from_str(&token).unwrap_err();
                                // return Err(Error::UnparsableInteger(token, err, location))
                            }
                        }
                    }
                    Mode::Decimal(whole) => {
                        match char {
                            '0'..='9' => {
                                token.push(char)
                            }
                            '_' => {}
                            _ => {
                                match u128::from_str(&token) {
                                    Ok(fractional) => {
                                        tokens.push(Token::Decimal(whole, fractional, token.len().try_into().expect("fractional part is too long")));
                                        token.clear();
                                        mode = Mode::Whitespace;
                                        continue 'matcher;
                                    }
                                    Err(err) => {
                                        return Err(Error::UnparsableDecimal(whole, token.clone(), err, location))
                                    }
                                }
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