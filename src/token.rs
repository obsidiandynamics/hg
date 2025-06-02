use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Text(String),
    Character(char),
    Integer(u128),
    Decimal(u128, u128, u8), // (whole part, fractional part, scale)
    Boolean(bool),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Dash,
    Colon,
    Comma,
    Ident(String),
    Newline,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Location {
    pub line: u32,
    pub column: u32
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}