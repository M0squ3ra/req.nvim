/// Error returned when `.req` input cannot be converted into an executable request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable error message.
    pub message: String,
    /// One-based line number where the error happened.
    pub line: usize,
    /// One-based column number where the error happened.
    pub column: usize,
}
