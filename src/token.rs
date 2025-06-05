use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Text(String),
    Character(char),
    Integer(u128),
    Decimal(u128, u128, u8), // (whole part, fractional part, scale)
    Boolean(bool),
    Left(ListDelimiter),
    Right(ListDelimiter),
    Dash,
    Colon,
    Comma,
    Ident(String),
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

// impl Location {
//     #[inline(always)]
//     pub fn next_line(&mut self) {
//         self.line += 1;
//         self.column = 0;
//     }
// }

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}