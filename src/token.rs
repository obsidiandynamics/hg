use std::borrow::Cow;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Text(Cow<'a, str>),
    Character(char),
    Integer(u128),
    Decimal(u128, u128, u8), // (whole part, fractional part, scale)
    Boolean(bool),
    Left(ListDelimiter),
    Right(ListDelimiter),
    Dash,
    Colon,
    Comma,
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