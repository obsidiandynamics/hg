use std::borrow::Cow;
use crate::char_buffer::CharBuffer;
use crate::token::{Ascii, AsciiSlice, ListDelimiter, Token};
use std::io;
use std::num::ParseIntError;
use std::str::FromStr;
use crate::graphemes::Grapheme;
use crate::metadata::{Location, Metadata};
use crate::newline_terminated_bytes::NewlineTerminatedBytes;
use crate::symbols::{is_symbol, SymbolString, SymbolTable};

#[derive(Debug, thiserror::Error)]
#[error("codepoint out of range")]
pub struct CodepointOutOfRange;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error {0}")]
    Io(#[from] io::Error),

    #[error("unexpected character '{0}' at {1}")]
    UnexpectedCharacter(char, Location),

    #[error("unterminated literal at {0}")]
    UnterminatedLiteral(Location),

    #[error("unknown escape sequence \"{0}\" at {1}")]
    UnknownEscapeSequence(String, Location),

    #[error("invalid codepoint \"{0}\" ({1}) at {2}")]
    InvalidCodepoint(String, Box<dyn std::error::Error>, Location),

    #[error("unparsable integer {0} ({1}) at {2}")]
    UnparsableInteger(String, ParseIntError, Location),

    #[error("unparsable decimal {0}.{1} ({2}) at {3}")]
    UnparsableDecimal(u128, String, ParseIntError, Location),

    #[error("empty character literal at {0}")]
    EmptyCharacterLiteral(Location),
}

enum Mode {
    Whitespace,
    Text,
    Character,
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
    start: Location,
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
            start: Location::before_start(),
            location: Location::before_start(),
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
            } else if self.token.len() == 1 && self.token.first_byte(self.bytes) == b'.' && byte.is_ascii_digit() {
                self.token.clear();
                self.stashed_byte = Some((index, byte)); // don't consume the char
                self.mode = Mode::Decimal(0);
                self.start = self.location.clone();
                return None
            } else {
                self.stashed_byte = Some((index, byte)); // don't consume the char
                return Some(self.make_symbol())
            }
        }
        unreachable!() // since '\n' is guaranteed to terminate the stream (handled in the loop above)
    }

    #[inline]
    fn parse_escape(&mut self) -> Result<char, Box<Error>> {
        enum EscapeState {
            Single,
            Hex,
            UnicodeFixed,
            UnicodeVariable
        }

        let mut buf = String::new();
        let mut state = EscapeState::Single;
        while let Some((_, byte)) = self.next_byte() {
            self.location.column += 1;
            if byte == b'\n' {
                self.error = true;
                let str = unsafe { String::from_utf8_unchecked(vec![byte]) };
                return Err(Error::UnknownEscapeSequence(str, self.location.clone()).into())
            } else if byte < 0x80 {
                match state {
                    EscapeState::Single => {
                        match byte {
                            b'\\' | b'"' | b'\'' => {
                                return Ok(byte as char)
                            }
                            b'n' => {
                                return Ok('\n')
                            }
                            b'r' => {
                                return Ok('\r')
                            }
                            b't' => {
                                return Ok('\t')
                            }
                            b'0' => {
                                return Ok('\0')
                            }
                            b'x' => {
                                state = EscapeState::Hex
                            }
                            b'u' => {
                                state = EscapeState::UnicodeFixed
                            }
                            _ => {
                                self.error = true;
                                let str = unsafe { String::from_utf8_unchecked(vec![byte]) };
                                return Err(Error::UnknownEscapeSequence(str, self.location.clone()).into())
                            }
                        }
                    }
                    EscapeState::Hex => {
                        buf.push(byte as char);
                        if buf.len() == 2 {
                            return self.make_unicode(&buf)
                        }
                    }
                    EscapeState::UnicodeFixed => {
                        if buf.is_empty() && byte == b'{' {
                            state = EscapeState::UnicodeVariable;
                        } else {
                            buf.push(byte as char);
                            if buf.len() == 4 {
                                return self.make_unicode(&buf)
                            }
                        }
                    }
                    EscapeState::UnicodeVariable => {
                        if byte == b'}' {
                            return self.make_unicode(&buf)
                        } else {
                            buf.push(byte as char);
                        }
                    }
                }
            } else {
                self.error = true;
                let grapheme = read_grapheme(byte, &mut self.byte_indexes);
                buf.push(char::from(grapheme));
                return Err(Error::UnknownEscapeSequence(buf, self.location.clone()).into())
            }
        }
        unreachable!() // since '\n' is guaranteed to terminate the stream (handled in the loop above)
    }

    #[inline]
    fn make_unicode(&mut self, buf: &str) -> Result<char, Box<Error>> {
        match u32::from_str_radix(buf, 16) {
            Ok(hex) => {
                match char::from_u32(hex) {
                    None => {
                        self.error = true;
                        Err(Error::InvalidCodepoint(buf.to_string(), Box::new(CodepointOutOfRange), self.location.clone()).into())
                    }
                    Some(char) => Ok(char)
                }
            }
            Err(err) => {
                self.error = true;
                Err(Error::InvalidCodepoint(buf.to_string(), Box::new(err), self.location.clone()).into())
            }
        }
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
                self.frame_token(token)
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
                self.frame_token(token)
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
        self.frame_token(token)
    }

    fn frame_token(&mut self, token: Token<'a>) -> Option<Fragment<'a>> {
        let start = Some(self.start.clone());
        self.start = self.location.clone();
        self.start.column += 1;
        let end = Some(self.location.clone());
        Some(Ok((token, Metadata { start, end })))
    }
}

pub type Fragment<'a> = Result<(Token<'a>, Metadata), Box<Error>>;

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
                            self.start = self.location.clone();
                            self.mode = Mode::Text;
                        }
                        b'\'' => {
                            self.start = self.location.clone();
                            self.mode = Mode::Character;
                        }
                        b'\t' | b'\r' | b' ' => {}
                        b'\n' => {
                            self.location.line += 1;
                            self.location.column = 0;
                            return self.frame_token(Token::Newline)
                        }
                        b'(' => {
                            self.start = self.location.clone();
                            return self.frame_token(Token::Left(ListDelimiter::Paren));
                        }
                        b')' => {
                            self.start = self.location.clone();
                            return self.frame_token(Token::Right(ListDelimiter::Paren));
                        }
                        b'{' => {
                            self.start = self.location.clone();
                            return self.frame_token(Token::Left(ListDelimiter::Brace));
                        }
                        b'}' => {
                            self.start = self.location.clone();
                            return self.frame_token(Token::Right(ListDelimiter::Brace));
                        }
                        b'[' => {
                            self.start = self.location.clone();
                            return self.frame_token(Token::Left(ListDelimiter::Bracket));
                        }
                        b']' => {
                            self.start = self.location.clone();
                            return self.frame_token(Token::Right(ListDelimiter::Bracket));
                        }
                        b'0'..=b'9' => {
                            self.mode = Mode::Integer;
                            self.start = self.location.clone();
                            self.token.push_byte(index, byte);
                        }
                        _ => {
                            if is_symbol(byte) {
                                self.start = self.location.clone();
                                self.token.push_byte(index, byte);
                                match self.parse_symbol() {
                                    None => {}
                                    Some(token) => {
                                        return self.frame_token(token)
                                    }
                                }
                            } else {
                                self.start = self.location.clone();
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
                Mode::Text => {
                    match byte {
                        b'\\' => {
                            match self.parse_escape() {
                                Ok(char) => {
                                    self.token.copy(self.bytes);
                                    self.token.push_char(0, char);
                                }
                                Err(err) => {
                                    return Some(Err(err))
                                }
                            }
                        }
                        b'"' => {
                            let token = Token::Text(self.token.string(self.bytes));
                            self.token.clear();
                            self.mode = Mode::Whitespace;
                            return self.frame_token(token)
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
                Mode::Character => {
                    match byte {
                        b'\\' => {
                            match self.parse_escape() {
                                Ok(char) => {
                                    self.token.copy(self.bytes);
                                    self.token.push_char(0, char);
                                }
                                Err(err) => {
                                    return Some(Err(err))
                                }
                            }
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
                                    self.frame_token(token)
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