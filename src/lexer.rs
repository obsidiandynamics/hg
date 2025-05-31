use std::io;
use std::io::{BufRead, BufReader, Read};
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
    UnknownEscapeSequence(char, Location)
}

enum Mode {
    Whitespace,
    TextBody,
    TextEscape,
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
            match char {
                '\\' => {
                    match mode {
                        Mode::Whitespace => {
                            return Err(Error::UnexpectedChar(char, location))
                        }
                        Mode::TextBody => {
                            mode = Mode::TextEscape
                        }
                        Mode::TextEscape => {
                            token.push('\\');
                            mode = Mode::TextBody
                        }
                    }
                }
                '"' => {
                    match mode {
                        Mode::Whitespace => mode = Mode::TextBody,
                        Mode::TextBody => {
                            tokens.push(Token::Text(token.clone()));
                            token.clear();
                            mode = Mode::Whitespace
                        },
                        Mode::TextEscape => { 
                            token.push('"');
                            mode = Mode::TextBody
                        }
                    }
                },
                '\t' | '\r' | ' ' => {
                    match mode {
                        Mode::Whitespace => {}
                        Mode::TextBody => {
                            token.push(char)
                        },
                        Mode::TextEscape => todo!()
                    }
                }
                '\n' => {
                    match mode {
                        Mode::Whitespace => {
                            tokens.push(Token::Newline)
                        }
                        Mode::TextBody => {
                            return Err(Error::UnterminatedText(location))
                        },
                        Mode::TextEscape => todo!()
                    }
                }
                _ => {
                    match mode {
                        Mode::Whitespace => {
                            //TODO start ident
                        }
                        Mode::TextBody => {
                            token.push(char)
                        },
                        Mode::TextEscape => { 
                            match char {
                                '"' => { 
                                    token.push('"');
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
                                _ => {
                                    return Err(Error::UnknownEscapeSequence(char, location))
                                }
                            }
                            mode = Mode::TextBody
                        }
                    }
                }
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