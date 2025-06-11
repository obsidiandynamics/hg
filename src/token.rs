use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use crate::types::unqualified_type_name;

#[derive(PartialEq, Eq)]
pub struct Byte(pub u8);

impl Debug for Byte {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(b'{}')", unqualified_type_name::<Self>(), self.0 as char)
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
    Symbol(Byte),
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
    use crate::token::Byte;

    #[test]
    fn byte_debug() {
        let byte = Byte(b'a');
        assert_eq!("Byte(b'a')", format!("{byte:?}"));
    }
}