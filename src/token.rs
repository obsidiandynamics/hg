use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use crate::types::unqualified_type_name;

#[derive(PartialEq, Eq)]
pub struct Ascii(pub u8);

impl Debug for Ascii {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(b'{}')", unqualified_type_name::<Self>(), self.0 as char)
    }
}

#[derive(PartialEq, Eq)]
pub struct AsciiSlice<'a>(pub &'a [u8]);

impl Debug for AsciiSlice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::from("[");
        for (index, byte) in self.0.iter().enumerate() {
            buf.push_str(format!("b'{}'", *byte as char).as_str());
            if index < self.0.len() - 1 {
                buf.push_str(", ")
            }
        }
        write!(f, "{buf}]")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Text(Cow<'a, str>),
    Character(char),
    Integer(u128),
    Decimal(u128, u128, u8), // (whole part, fractional part, scale)
    Boolean(bool),
    Left(ListDelimiter),
    Right(ListDelimiter),
    Symbol(Ascii),
    ExtendedSymbol(AsciiSlice<'a>),
    Ident(Cow<'a, str>),
    Newline,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListDelimiter {
    Paren,
    Brace,
    Bracket,
    Angle
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub line: u32,
    pub column: u32
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use crate::token::Ascii;

    #[test]
    fn byte_debug() {
        let byte = Ascii(b'a');
        assert_eq!("Ascii(b'a')", format!("{byte:?}"));
    }
}