use std::borrow::Cow;
use crate::char_buffer::CharBuffer;
use crate::newline_terminated_chars::NewlineTerminatedChars;
use crate::token::{ListDelimiter, Location, Token};
use std::io;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("i/o error {0}")]
    Io(#[from] io::Error),

    #[error("unexpected character '{0}' at {1}")]
    UnexpectedCharacter(char, Location),

    #[error("unterminated literal at {0}")]
    UnterminatedLiteral(Location),

    #[error("unknown escape sequence '{0}' at {1}")]
    UnknownEscapeSequence(char, Location),

    #[error("unparsable integer {0} ({1}) at {2}")]
    UnparsableInteger(Cow<'a, str>, ParseIntError, Location),

    #[error("unparsable decimal {0}.{1} ({2}) at {3}")]
    UnparsableDecimal(u128, Cow<'a, str>, ParseIntError, Location),

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

pub struct Tokeniser<'a> {
    bytes: &'a [u8],
    char_indexes: NewlineTerminatedChars<'a>,
    token: CharBuffer,
    mode: Mode,
    location: Location,
    stashed_char: Option<(usize, char)>,
    error: bool
}

impl<'a> Tokeniser<'a> {
    #[inline]
    pub fn new(str: &'a str) -> Self {
        Self {
            bytes: str.as_bytes(),
            char_indexes:  NewlineTerminatedChars::new(str.char_indices()),
            token: CharBuffer::default(),
            mode: Mode::Whitespace,
            location: Location { line: 1, column: 0 },
            stashed_char: None,
            error: false,
        }
    }

    #[inline(always)]
    fn next_char(&mut self) -> Option<(usize, char)> {
        match self.stashed_char.take() {
            None => self.char_indexes.next(),
            Some((index, char)) => Some((index, char))
        }
    }
}

impl<'a> Iterator for Tokeniser<'a> {
    type Item = Result<Token<'a>, Error<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.error {
            return None;
        }

        while let Some((index, char)) = self.next_char() {
            self.location.column += 1;
            let byte = char as u8;
            match self.mode {
                Mode::Whitespace => {
                    match byte {
                        b'\\' => {
                            self.error = true;
                            return Some(Err(Error::UnexpectedCharacter(char, self.location.clone())))
                        }
                        b'"' => {
                            self.mode = Mode::TextBody;
                        }
                        b'\'' => {
                            self.mode = Mode::CharacterBody;
                        }
                        b'\t' | b'\r' | b' ' => {}
                        b'\n' => {
                            self.location.line += 1;
                            self.location.column = 1;
                            return Some(Ok(Token::Newline))
                        }
                        b'(' => {
                            return Some(Ok(Token::Left(ListDelimiter::Paren)));
                        }
                        b')' => {
                            return Some(Ok(Token::Right(ListDelimiter::Paren)));
                        }
                        b'{' => {
                            return Some(Ok(Token::Left(ListDelimiter::Brace)));
                        }
                        b'}' => {
                            return Some(Ok(Token::Right(ListDelimiter::Brace)));
                        }
                        b'[' => {
                            return Some(Ok(Token::Left(ListDelimiter::Bracket)));
                        }
                        b']' => {
                            return Some(Ok(Token::Right(ListDelimiter::Bracket)));
                        }
                        b'-' => {
                            return Some(Ok(Token::Dash));
                        }
                        b':' => {
                            return Some(Ok(Token::Colon));
                        }
                        b',' => {
                            return Some(Ok(Token::Comma));
                        }
                        b'0'..=b'9' => {
                            self.mode = Mode::Integer;
                            self.token.push(index, char);
                        }
                        b'.' => {
                            self.mode = Mode::Decimal(0);
                        }
                        _ => {
                            self.mode = Mode::Ident;
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::TextBody => {
                    match byte {
                        b'\\' => {
                            self.token.copy(self.bytes);
                            self.mode = Mode::TextEscape;
                        }
                        b'"' => {
                            let token = Token::Text(self.token.string(self.bytes));
                            self.token.clear();
                            self.mode = Mode::Whitespace;
                            return Some(Ok(token))
                        }
                        b'\n' => {
                            self.error = true;
                            return Some(Err(Error::UnterminatedLiteral(self.location.clone())))
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::CharacterEscape | Mode::TextEscape => {
                    match byte {
                        b'\\' | b'"' | b'\'' => {
                            self.token.push(0, char);
                        }
                        b'n' => {
                            self.token.push(0, '\n');
                        }
                        b'r' => {
                            self.token.push(0, '\r');
                        }
                        b't' => {
                            self.token.push(0, '\t');
                        }
                        b'x' => {
                            //TODO handle hex (e.g., \x7F)
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(char, self.location.clone())))
                        }
                        b'u' => {
                            //TODO handle unicode (e.g., \u{7FFF})
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(char, self.location.clone())))
                        }
                        _ => {
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(char, self.location.clone())))
                        }
                    }
                    match self.mode {
                        Mode::TextEscape => {
                            self.mode = Mode::TextBody;
                        }
                        Mode::CharacterEscape => {
                            self.mode = Mode::CharacterBody;
                        }
                        _ => {
                            // by the encompassing pattern, mode must be one of the two variants above
                            unreachable!()
                        }
                    }
                }
                Mode::CharacterBody => {
                    match byte {
                        b'\\' => {
                            self.mode = Mode::CharacterEscape;
                            self.token.copy(self.bytes);
                        }
                        b'\'' => {
                            let mut chars = self.token.as_str(self.bytes).chars();
                            return match chars.next() {
                                None => {
                                    self.error = true;
                                    Some(Err(Error::EmptyCharacterLiteral(self.location.clone())))
                                }
                                Some(first_char) => {
                                    let token = Token::Character(first_char);
                                    self.token.clear();
                                    self.mode = Mode::Whitespace;
                                    Some(Ok(token))
                                }
                            }
                        }
                        b'\n' => {
                            self.error = true;
                            return Some(Err(Error::UnterminatedLiteral(self.location.clone())))
                        }
                        _ => {
                            if self.token.is_empty() {
                                self.token.push(index, char);
                            } else {
                                self.error = true;
                                return Some(Err(Error::UnexpectedCharacter(char, self.location.clone())))
                            }
                        }
                    }
                }
                Mode::Integer => {
                    match byte {
                        b'_' => {
                            self.token.copy(self.bytes);
                        }
                        b'.' => {
                            let str = self.token.as_str(self.bytes);
                            match u128::from_str(str) {
                                Ok(int) => {
                                    self.mode = Mode::Decimal(int);
                                    self.token.clear()
                                }
                                Err(err) => {
                                    self.error = true;
                                    return Some(Err(Error::UnparsableInteger(self.token.string(self.bytes), err, self.location.clone())))
                                }
                            }
                        }
                        b')' | b']' | b'}' | b':' | b'-' | b',' | b'\n' | b'\t' | b'\r' | b' ' => {
                            let str = self.token.as_str(self.bytes);
                            return match u128::from_str(str) {
                                Ok(whole) => {
                                    let token = Token::Integer(whole);
                                    self.token.clear();
                                    self.mode = Mode::Whitespace;
                                    self.stashed_char = Some((index, char)); // don't consume the char
                                    self.location.column -= 1;
                                    Some(Ok(token))
                                }
                                Err(err) => {
                                    Some(Err(Error::UnparsableInteger(self.token.string(self.bytes), err, self.location.clone())))
                                }
                            }
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::Decimal(whole) => {
                    match byte {
                        b'_' => {
                            self.token.copy(self.bytes);
                        }
                        b')' | b']' | b'}' | b':' | b'-' | b',' | b'\n' | b'\t' | b'\r' | b' ' => {
                            let str = self.token.as_str(self.bytes);
                            return match u128::from_str(str) {
                                Ok(fractional) => {
                                    let token = Token::Decimal(whole, fractional, self.token.len().try_into().expect("fractional part is too long"));
                                    self.token.clear();
                                    self.mode = Mode::Whitespace;
                                    self.stashed_char = Some((index, char)); // don't consume the char
                                    self.location.column -= 1;
                                    Some(Ok(token))
                                }
                                Err(err) => {
                                    Some(Err(Error::UnparsableDecimal(whole, self.token.string(self.bytes), err, self.location.clone())))
                                }
                            }
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::Ident => {
                    match byte {
                        b')' | b']' | b'}' | b':' | b'-'  | b',' | b'\n' | b'\t' | b'\r' | b' ' => {
                            let str = self.token.as_str(self.bytes);
                            let token = match str {
                                "true" => {
                                    Token::Boolean(true)
                                }
                                "false" => {
                                    Token::Boolean(false)
                                }
                                _ => {
                                    Token::Ident(self.token.string(self.bytes))
                                }
                            };
                            self.token.clear();
                            self.mode = Mode::Whitespace;
                            self.stashed_char = Some((index, char)); // don't consume the char
                            self.location.column -= 1;
                            return Some(Ok(token))
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;