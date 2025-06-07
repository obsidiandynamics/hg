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
            match self.mode {
                Mode::Whitespace => {
                    match char {
                        '\\' => {
                            self.error = true;
                            return Some(Err(Error::UnexpectedCharacter(char, self.location.clone())))
                        }
                        '"' => {
                            self.mode = Mode::TextBody;
                        }
                        '\'' => {
                            self.mode = Mode::CharacterBody;
                        }
                        '\t' | '\r' | ' ' => {}
                        '\n' => {
                            self.location.line += 1;
                            self.location.column = 1;
                            return Some(Ok(Token::Newline))
                        }
                        '(' => {
                            return Some(Ok(Token::Left(ListDelimiter::Paren)));
                        }
                        ')' => {
                            return Some(Ok(Token::Right(ListDelimiter::Paren)));
                        }
                        '{' => {
                            return Some(Ok(Token::Left(ListDelimiter::Brace)));
                        }
                        '}' => {
                            return Some(Ok(Token::Right(ListDelimiter::Brace)));
                        }
                        '[' => {
                            return Some(Ok(Token::Left(ListDelimiter::Bracket)));
                        }
                        ']' => {
                            return Some(Ok(Token::Right(ListDelimiter::Bracket)));
                        }
                        '-' => {
                            return Some(Ok(Token::Dash));
                        }
                        ':' => {
                            return Some(Ok(Token::Colon));
                        }
                        ',' => {
                            return Some(Ok(Token::Comma));
                        }
                        '0'..='9' => {
                            self.mode = Mode::Integer;
                            self.token.push(index, char);
                        }
                        '.' => {
                            self.mode = Mode::Decimal(0);
                        }
                        _ => {
                            self.mode = Mode::Ident;
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::TextBody => {
                    match char {
                        '\\' => {
                            self.token.copy(self.bytes);
                            self.mode = Mode::TextEscape;
                        }
                        '"' => {
                            let token = Token::Text(self.token.string(self.bytes));
                            self.token.clear();
                            self.mode = Mode::Whitespace;
                            return Some(Ok(token))
                        }
                        '\n' => {
                            self.error = true;
                            return Some(Err(Error::UnterminatedLiteral(self.location.clone())))
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::CharacterEscape | Mode::TextEscape => {
                    match char {
                        '\\' | '"' | '\'' => {
                            self.token.push(0, char);
                        }
                        'n' => {
                            self.token.push(0, '\n');
                        }
                        'r' => {
                            self.token.push(0, '\r');
                        }
                        't' => {
                            self.token.push(0, '\t');
                        }
                        'x' => {
                            //TODO handle hex (e.g., \x7F)
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(char, self.location.clone())))
                        }
                        'u' => {
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
                    match char {
                        '\\' => {
                            self.mode = Mode::CharacterEscape;
                            self.token.copy(self.bytes);
                        }
                        '\'' => {
                            let mut chars = self.token.as_str(self.bytes).chars();
                            match chars.next() {
                                None => {
                                    self.error = true;
                                    return Some(Err(Error::EmptyCharacterLiteral(self.location.clone())))
                                }
                                Some(first_char) => {
                                    let token = Token::Character(first_char);
                                    self.token.clear();
                                    self.mode = Mode::Whitespace;
                                    return Some(Ok(token))
                                }
                            }
                        }
                        '\n' => {
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
                    match char {
                        '_' => {
                            self.token.copy(self.bytes);
                        }
                        '.' => {
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
                        ')' | ']' | '}' | ':' | '-' | ',' | '\n' | '\t' | '\r' | ' ' => {
                            let str = self.token.as_str(self.bytes);
                            match u128::from_str(str) {
                                Ok(whole) => {
                                    let token = Token::Integer(whole);
                                    self.token.clear();
                                    self.mode = Mode::Whitespace;
                                    self.stashed_char = Some((index, char)); // don't consume the char
                                    self.location.column -= 1;
                                    return Some(Ok(token));
                                }
                                Err(err) => {
                                    return Some(Err(Error::UnparsableInteger(self.token.string(self.bytes), err, self.location.clone())))
                                }
                            }
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::Decimal(whole) => {
                    match char {
                        '_' => {
                            self.token.copy(self.bytes);
                        }
                        ')' | ']' | '}' | ':' | '-' | ',' | '\n' | '\t' | '\r' | ' ' => {
                            let str = self.token.as_str(self.bytes);
                            match u128::from_str(str) {
                                Ok(fractional) => {
                                    let token = Token::Decimal(whole, fractional, self.token.len().try_into().expect("fractional part is too long"));
                                    self.token.clear();
                                    self.mode = Mode::Whitespace;
                                    self.stashed_char = Some((index, char)); // don't consume the char
                                    self.location.column -= 1;
                                    return Some(Ok(token))
                                }
                                Err(err) => {
                                    return Some(Err(Error::UnparsableDecimal(whole, self.token.string(self.bytes), err, self.location.clone())))
                                }
                            }
                        }
                        _ => {
                            self.token.push(index, char);
                        }
                    }
                }
                Mode::Ident => {
                    match char {
                        ')' | ']' | '}' | ':' | '-'  | ',' | '\n' | '\t' | '\r' | ' ' => {
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

// pub fn tokenise(str: &str) -> Result<VecDeque<Token>, Error> {
//     let bytes = str.as_bytes();
//     let char_indexes = NewlineTerminatedChars::new(str.char_indices());
//     
//     let mut tokens = VecDeque::new();
//     let mut token = CharBuffer::default();
//     let mut mode = Mode::Whitespace;
//     let mut location = Location { line: 1, column: 0 };
//     for (index, char) in char_indexes {
//         //println!("{char} at index {index}");
//         location.column += 1;
//         'matcher: loop {
//             match mode {
//                 Mode::Whitespace => {
//                     match char {
//                         '\\' => {
//                             return Err(Error::UnexpectedCharacter(char, location))
//                         }
//                         '"' => {
//                             mode = Mode::TextBody;
//                         }
//                         '\'' => {
//                             mode = Mode::CharacterBody;
//                         }
//                         '\t' | '\r' | ' ' => {}
//                         '\n' => {
//                             tokens.push_back(Token::Newline);
//                             location.line += 1;
//                             location.column = 1;
//                         }
//                         '(' => {
//                             tokens.push_back(Token::Left(ListDelimiter::Paren));
//                         }
//                         ')' => {
//                             tokens.push_back(Token::Right(ListDelimiter::Paren));
//                         }
//                         '{' => {
//                             tokens.push_back(Token::Left(ListDelimiter::Brace));
//                         }
//                         '}' => {
//                             tokens.push_back(Token::Right(ListDelimiter::Brace));
//                         }
//                         '[' => {
//                             tokens.push_back(Token::Left(ListDelimiter::Bracket));
//                         }
//                         ']' => {
//                             tokens.push_back(Token::Right(ListDelimiter::Bracket));
//                         }
//                         '-' => {
//                             tokens.push_back(Token::Dash);
//                         }
//                         ':' => {
//                             tokens.push_back(Token::Colon);
//                         }
//                         ',' => {
//                             tokens.push_back(Token::Comma);
//                         }
//                         '0'..='9' => {
//                             mode = Mode::Integer;
//                             token.push(index, char);
//                         }
//                         '.' => {
//                             mode = Mode::Decimal(0);
//                         }
//                         _ => {
//                             mode = Mode::Ident;
//                             token.push(index, char);
//                         }
//                     }
//                 }
//                 Mode::TextBody => {
//                     match char {
//                         '\\' => {
//                             token.copy(bytes);
//                             mode = Mode::TextEscape;
//                         }
//                         '"' => {
//                             tokens.push_back(Token::Text(token.string(bytes)));
//                             token.clear();
//                             mode = Mode::Whitespace;
//                         }
//                         '\n' => {
//                             return Err(Error::UnterminatedLiteral(location))
//                         }
//                         _ => {
//                             token.push(index, char);
//                         }
//                     }
//                 }
//                 Mode::CharacterEscape | Mode::TextEscape => {
//                     match char {
//                         '\\' | '"' | '\'' => {
//                             token.push(0, char);
//                         }
//                         'n' => {
//                             token.push(0, '\n');
//                         }
//                         'r' => {
//                             token.push(0, '\r');
//                         }
//                         't' => {
//                             token.push(0, '\t');
//                         }
//                         'x' => {
//                             //TODO handle hex (e.g., \x7F)
//                             return Err(Error::UnknownEscapeSequence(char, location))
//                         }
//                         'u' => {
//                             //TODO handle unicode (e.g., \u{7FFF})
//                             return Err(Error::UnknownEscapeSequence(char, location))
//                         }
//                         _ => {
//                             return Err(Error::UnknownEscapeSequence(char, location))
//                         }
//                     }
//                     match mode {
//                         Mode::TextEscape => {
//                             mode = Mode::TextBody;
//                         }
//                         Mode::CharacterEscape => {
//                             mode = Mode::CharacterBody;
//                         }
//                         _ => {
//                             // by the encompassing pattern, mode must be one of the two variants above
//                             unreachable!()
//                         }
//                     }
//                 }
//                 Mode::CharacterBody => {
//                     match char {
//                         '\\' => {
//                             mode = Mode::CharacterEscape;
//                             token.copy(bytes);
//                         }
//                         '\'' => {
//                             let mut chars = token.as_str(bytes).chars();
//                             match chars.next() {
//                                 None => {
//                                     return Err(Error::EmptyCharacterLiteral(location))
//                                 }
//                                 Some(first_char) => {
//                                     tokens.push_back(Token::Character(first_char));
//                                     token.clear();
//                                     mode = Mode::Whitespace;
//                                 }
//                             }
//                         }
//                         '\n' => {
//                             return Err(Error::UnterminatedLiteral(location))
//                         }
//                         _ => {
//                             if token.is_empty() {
//                                 token.push(index, char);
//                             } else {
//                                 return Err(Error::UnexpectedCharacter(char, location))
//                             }
//                         }
//                     }
//                 }
//                 Mode::Integer => {
//                     match char {
//                         '_' => {
//                             token.copy(bytes);
//                         }
//                         '.' => {
//                             let str = token.as_str(bytes);
//                             match u128::from_str(str) {
//                                 Ok(int) => {
//                                     mode = Mode::Decimal(int);
//                                     token.clear()
//                                 }
//                                 Err(err) => {
//                                     return Err(Error::UnparsableInteger(token.string(bytes), err, location))
//                                 }
//                             }
//                         }
//                         ')' | '}' | ':' | '-' | ',' | '\n' | '\t' | '\r' | ' ' => {
//                             let str = token.as_str(bytes);
//                             match u128::from_str(str) {
//                                 Ok(whole) => {
//                                     tokens.push_back(Token::Integer(whole));
//                                     token.clear();
//                                     mode = Mode::Whitespace;
//                                     continue 'matcher; // don't consume the char
//                                 }
//                                 Err(err) => {
//                                     return Err(Error::UnparsableInteger(token.string(bytes), err, location))
//                                 }
//                             }
//                         }
//                         _ => {
//                             token.push(index, char);
//                         }
//                     }
//                 }
//                 Mode::Decimal(whole) => {
//                     match char {
//                         '_' => {
//                             token.copy(bytes);
//                         }
//                         ')' | '}' | ':' | '-' | ',' | '\n' | '\t' | '\r' | ' ' => {
//                             let str = token.as_str(bytes);
//                             match u128::from_str(str) {
//                                 Ok(fractional) => {
//                                     tokens.push_back(Token::Decimal(whole, fractional, token.len().try_into().expect("fractional part is too long")));
//                                     token.clear();
//                                     mode = Mode::Whitespace;
//                                     continue 'matcher;  // don't consume the char
//                                 }
//                                 Err(err) => {
//                                     return Err(Error::UnparsableDecimal(whole, token.string(bytes), err, location))
//                                 }
//                             }
//                         }
//                         _ => {
//                             token.push(index, char);
//                         }
//                     }
//                 }
//                 Mode::Ident => {
//                     match char {
//                         ')' | '}' | ':' | '-'  | ',' | '\n' | '\t' | '\r' | ' ' => {
//                             let str = token.as_str(bytes);
//                             match str {
//                                 "true" => {
//                                     tokens.push_back(Token::Boolean(true));
//                                 }
//                                 "false" => {
//                                     tokens.push_back(Token::Boolean(false));
//                                 }
//                                 _ => {
//                                     tokens.push_back(Token::Ident(token.string(bytes)));
//                                 }
//                             }
//                             token.clear();
//                             mode = Mode::Whitespace;
//                             continue 'matcher;  // don't consume the char
//                         }
//                         _ => {
//                             token.push(index, char);
//                         }
//                     }
//                 }
//             }
//             break 'matcher;
//         }
//     }
//     Ok(tokens)
// }

#[cfg(test)]
mod tests;