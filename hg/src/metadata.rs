use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub line: u32,
    pub column: u32,
}

impl Location {
    #[inline]
    pub fn before_start() -> Self {
        Self { line: 1, column: 0 }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub start: Option<Location>,
    pub end: Option<Location>,
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.start, &self.end) {
            (None, None) => {
                write!(f, "unspecified location")
            }
            (Some(location), None) => {
                write!(f, "region after {location}")
            }
            (None, Some(location)) => {
                write!(f, "region before {location}")
            }
            (Some(start), Some(end)) => {
                if start.line == end.line {
                    if start.column == end.column {
                        write!(
                            f,
                            "line {}, column {}",
                            start.line, start.column
                        )
                    } else {
                        write!(
                            f,
                            "line {}, columns {} to {}",
                            start.line, start.column, end.column
                        )
                    }
                } else {
                    write!(f, "{start} to {end}")
                }
            }
        }
    }
}

impl Metadata {
    pub const fn unspecified() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn bounds(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        debug_assert!(start_line <= end_line);
        debug_assert!(
            start_line == end_line && start_column <= end_column || start_line + 1 == end_line
        );
        Self {
            start: Some(Location {
                line: start_line,
                column: start_column,
            }),
            end: Some(Location {
                line: end_line,
                column: end_column,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::{Location, Metadata};

    #[test]
    fn location_display() {
        assert_eq!(
            "line 2, column 3",
            Location { line: 2, column: 3 }.to_string()
        );
    }

    #[test]
    fn metadata_display() {
        for (input, expect) in [
            (Metadata::unspecified(), "unspecified location"),
            (
                Metadata {
                    start: Some(Location { line: 2, column: 3 }),
                    end: None,
                },
                "region after line 2, column 3",
            ),
            (
                Metadata {
                    start: None,
                    end: Some(Location { line: 2, column: 3 }),
                },
                "region before line 2, column 3",
            ),
            (
                Metadata {
                    start: Some(Location { line: 2, column: 3 }),
                    end: Some(Location { line: 2, column: 5 }),
                },
                "line 2, columns 3 to 5",
            ),
            (
                Metadata {
                    start: Some(Location { line: 2, column: 5 }),
                    end: Some(Location { line: 2, column: 5 }),
                },
                "line 2, column 5",
            ),
            (
                Metadata {
                    start: Some(Location { line: 2, column: 3 }),
                    end: Some(Location { line: 3, column: 1 }),
                },
                "line 2, column 3 to line 3, column 1",
            ),
        ] {
            assert_eq!(expect, input.to_string(), "for input {input:?}")
        }
    }
}
