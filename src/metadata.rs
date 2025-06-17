use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub line: u32,
    pub column: u32
}

impl Location {
    #[inline]
    pub fn before_start() -> Self {
        Self {
            line: 1, column: 0
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub struct Metadata {
    pub start: Location,
    pub end: Location
}

impl Metadata {
    #[cfg(test)]
    pub fn new(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        debug_assert!(start_line <= end_line);
        debug_assert!(start_line == end_line && start_column <= end_column || start_line + 1 == end_line);
        Self {
            start: Location {
                line: start_line,
                column: start_column,
            },
            end: Location {
                line: end_line,
                column: end_column,
            },
        }
    }
}