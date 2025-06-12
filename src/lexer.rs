use std::borrow::Cow;
use crate::char_buffer::CharBuffer;
use crate::token::{Ascii, AsciiSlice, ListDelimiter, Location, Token};
use std::io;
use std::num::ParseIntError;
use std::str::FromStr;
use crate::graphemes::Grapheme;
use crate::newline_terminated_bytes::NewlineTerminatedBytes;
use crate::symbols::{is_symbol, SymbolString, SymbolTable};

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

pub struct Tokeniser<'a, 's> {
    symbol_table: SymbolTable<'s>,
    bytes: &'a [u8],
    byte_indexes: NewlineTerminatedBytes<'a>,
    token: CharBuffer,
    mode: Mode,
    location: Location,
    stashed_byte: Option<(usize, u8)>,
    error: bool
}

impl<'a, 's> Tokeniser<'a, 's> {
    #[inline]
    pub fn new(str: &'a str, symbol_table: SymbolTable<'s>) -> Self {
        Self {
            symbol_table,
            bytes: str.as_bytes(),
            byte_indexes:  NewlineTerminatedBytes::new(str.bytes()),
            token: CharBuffer::default(),
            mode: Mode::Whitespace,
            location: Location { line: 1, column: 0 },
            stashed_byte: None,
            error: false,
        }
    }

    #[inline(always)]
    fn next_byte(&mut self) -> Option<(usize, u8)> {
        self.stashed_byte.take().or_else(|| self.byte_indexes.next())
    }

    #[inline(always)]
    fn make_symbol(&mut self) -> Token<'a> {
        //println!("making symbol with string \"{}\"", self.token.string(self.bytes));
        let token = if self.token.len() == 1 {
            Token::Symbol(Ascii(self.token.first_byte(self.bytes)))
        } else {
            Token::ExtendedSymbol(AsciiSlice(self.token.make_byte_slice(self.bytes)))
        };
        self.token.clear();
        token
    }
    
    #[inline(always)]
    fn parse_symbol(&mut self) -> Option<Token<'a>> {
        while let Some((index, byte)) = self.next_byte() {
            //println!("read  b'{}'", byte as char);
            if is_symbol(byte) {
                let bytes = &self.bytes[self.token.offset()..index + 1];
                if self.symbol_table.contains(&SymbolString(Cow::Borrowed(bytes))) {
                    self.location.column += 1;
                    self.token.push_byte(index, byte);
                } else {
                    self.stashed_byte = Some((index, byte)); // don't consume the char
                    return Some(self.make_symbol())
                }
            } else if self.token.len() == 1 && self.token.first_byte(&self.bytes) == b'.' && byte.is_ascii_digit() {
                self.token.clear();
                self.stashed_byte = Some((index, byte)); // don't consume the char
                self.mode = Mode::Decimal(0);
                return None
            } else {
                self.stashed_byte = Some((index, byte)); // don't consume the char
                return Some(self.make_symbol())
            }
        }
        unreachable!() // since '\n' is guaranteed to terminate the stream, which is handled in the loop above
    }
    
    #[inline]
    fn make_integer(&mut self) -> Option<Fragment<'a>> {
        let str = self.token.as_str(self.bytes);
        match u128::from_str(str) {
            Ok(whole) => {
                let token = Token::Integer(whole);
                self.token.clear();
                self.mode = Mode::Whitespace;
                self.location.column -= 1;
                Some(Ok(token))
            }
            Err(err) => {
                self.error = true;
                Some(Err(Error::UnparsableInteger(str.to_string(), err, self.location.clone()).into()))
            }
        }
    }
    
    #[inline]
    fn make_decimal(&mut self, whole: u128) -> Option<Fragment<'a>> {
        let str = self.token.as_str(self.bytes);
        match u128::from_str(str) {
            Ok(fractional) => {
                let token = Token::Decimal(whole, fractional, self.token.len().try_into().expect("fractional part is too long"));
                self.token.clear();
                self.mode = Mode::Whitespace;
                self.location.column -= 1;
                Some(Ok(token))
            }
            Err(err) => {
                self.error = true;
                Some(Err(Error::UnparsableDecimal(whole, str.to_string(), err, self.location.clone()).into()))
            }
        }
    }
    
    #[inline]
    fn make_ident(&mut self) -> Option<Fragment<'a>> {
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
        self.location.column -= 1;
        Some(Ok(token))
    }
}

pub type Fragment<'a> = Result<Token<'a>, Box<Error>>;

impl<'a> Iterator for Tokeniser<'a, '_> {
    type Item = Fragment<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.error {
            return None;
        }

        while let Some((index, byte)) = self.next_byte() {
            self.location.column += 1;
            match self.mode {
                Mode::Whitespace => {
                    match byte {
                        b'\\' => {
                            self.error = true;
                            return Some(Err(Error::UnexpectedCharacter(byte as char, self.location.clone()).into()))
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
                        b'0'..=b'9' => {
                            self.mode = Mode::Integer;
                            self.token.push_byte(index, byte);
                        }
                        _ => {
                            if is_symbol(byte) {
                                self.token.push_byte(index, byte);
                                match self.parse_symbol() {
                                    None => {}
                                    Some(token) => {
                                        return Some(Ok(token))
                                    }
                                }
                            } else {
                                self.mode = Mode::Ident;
                                if byte < 0x80 {
                                    self.token.push_byte(index, byte);
                                } else {
                                    self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                                }
                            }
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
                            return Some(Err(Error::UnterminatedLiteral(self.location.clone()).into()))
                        }
                        _ => {
                            if byte < 0x80 {
                                self.token.push_byte(index, byte);
                            } else {
                                self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                            }
                        }
                    }
                }
                Mode::CharacterEscape | Mode::TextEscape => {
                    match byte {
                        b'\\' | b'"' | b'\'' => {
                            self.token.push_byte(0, byte);
                        }
                        b'n' => {
                            self.token.push_char(0, '\n');
                        }
                        b'r' => {
                            self.token.push_char(0, '\r');
                        }
                        b't' => {
                            self.token.push_char(0, '\t');
                        }
                        b'x' => {
                            //TODO handle hex (e.g., \x7F)
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(byte as char, self.location.clone()).into()))
                        }
                        b'u' => {
                            //TODO handle unicode (e.g., \u{7FFF})
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(byte as char, self.location.clone()).into()))
                        }
                        _ => {
                            self.error = true;
                            return Some(Err(Error::UnknownEscapeSequence(byte as char, self.location.clone()).into()))
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
                                    Some(Err(Error::EmptyCharacterLiteral(self.location.clone()).into()))
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
                            return Some(Err(Error::UnterminatedLiteral(self.location.clone()).into()))
                        }
                        _ => {
                            if self.token.is_empty() {
                                if byte < 0x80 {
                                    self.token.push_byte(index, byte);
                                } else {
                                    self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                                }
                            } else {
                                self.error = true;
                                return Some(Err(Error::UnexpectedCharacter(byte as char, self.location.clone()).into()))
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
                                    return Some(Err(Error::UnparsableInteger(str.to_string(), err, self.location.clone()).into()))
                                }
                            }
                        }
                        b')' | b']' | b'}' | b'\n' | b'\t' | b'\r' | b' ' => {
                            self.stashed_byte = Some((index, byte)); // don't consume the char
                            return self.make_integer();
                        }
                        _ => {
                            if byte < 0x80 {
                                if is_symbol(byte) {
                                    self.stashed_byte = Some((index, byte)); // don't consume the char
                                    return self.make_integer(); 
                                } else {
                                    self.token.push_byte(index, byte);
                                }
                            } else {
                                self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                            }
                        }
                    }
                }
                Mode::Decimal(whole) => {
                    match byte {
                        b'_' => {
                            self.token.copy(self.bytes);
                        }
                        b')' | b']' | b'}' | b'\n' | b'\t' | b'\r' | b' ' => {
                            self.stashed_byte = Some((index, byte)); // don't consume the char
                            return self.make_decimal(whole)
                        }
                        _ => {
                            if byte < 0x80 {
                                if is_symbol(byte) {
                                    self.stashed_byte = Some((index, byte)); // don't consume the char
                                    return self.make_decimal(whole)
                                } else {
                                    self.token.push_byte(index, byte);
                                }
                            } else {
                                self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                            }
                        }
                    }
                }
                Mode::Ident => {
                    match byte {
                        b')' | b']' | b'}' | b'\n' | b'\t' | b'\r' | b' ' => {
                            self.stashed_byte = Some((index, byte)); // don't consume the char
                            return self.make_ident()
                        }
                        _ => {
                            if byte < 0x80 {
                                if is_symbol(byte) {
                                    self.stashed_byte = Some((index, byte)); // don't consume the char
                                    return self.make_ident()
                                } else {
                                    self.token.push_byte(index, byte);
                                }
                            } else {
                                self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                            }

                            // if is_symbol(byte) {
                            //     self.stashed_byte = Some((index, byte)); // don't consume the char
                            //     return self.make_ident()
                            // } else if byte < 0x80 {
                            //     self.token.push_byte(index, byte);
                            // } else {
                            //     self.token.push_grapheme(index, read_grapheme(byte, &mut self.byte_indexes))
                            // }
                        }
                    }
                }
            }
        }
        None
    }
}

#[inline(never)]
pub fn read_grapheme(b0: u8, bytes: &mut NewlineTerminatedBytes) -> Grapheme {
    __read_grapheme(b0, bytes).unwrap()
}

#[inline(always)]
fn __read_grapheme(b0: u8, bytes: &mut NewlineTerminatedBytes) -> Option<Grapheme> {
    let (_, b1) = bytes.next()?;
    if b0 >= 0xE0 {
        let (_, b2) = bytes.next()?;
        if b0 >= 0xF0 {
            let (_, b3) = bytes.next()?;
            Some(Grapheme([b0, b1, b2, b3]))
        } else {
            Some(Grapheme([b0, b1, b2, 0]))
        }
    } else {
        Some(Grapheme([b0, b1, 0, 0]))
    }
}

#[cfg(test)]
mod tests;