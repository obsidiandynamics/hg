use hg::metadata::{Location, Metadata};

pub fn metadata_bounds(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Metadata {
    debug_assert!(start_line <= end_line);
    debug_assert!(
        start_line == end_line && start_column <= end_column || start_line + 1 == end_line
    );
    Metadata {
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
