use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Text(String),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
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