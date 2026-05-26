/// One-based source range for parsed `.req` syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Span {
    pub fn line(line: usize, len: usize) -> Self {
        Self {
            start_line: line,
            start_column: 1,
            end_line: line,
            end_column: len + 1,
        }
    }

    pub fn between(start_line: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start_line,
            start_column: 1,
            end_line,
            end_column,
        }
    }
}
