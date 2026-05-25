/// Parsed representation of a complete `.req` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqDocument {
    /// Request blocks found in the document.
    pub requests: Vec<ReqBlock>,
}

/// A single request block, optionally introduced by a `###` marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqBlock {
    /// Optional request name parsed from `### Name`.
    pub name: Option<String>,
    /// One-based line number where the block content starts.
    pub start_line: usize,
    /// Classified lines that belong to this block.
    pub lines: Vec<ReqLine>,
}

/// A classified line inside a request block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReqLine {
    /// An empty line.
    Empty,
    /// A parser-supported directive such as `@env dev`.
    Directive(Directive),
    /// Any line that is not empty and not a supported directive.
    Raw(String),
}

/// Parser-supported `.req` directives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    /// Selects an environment group for the request.
    Env(String),
    /// Defines a request-local variable override.
    Variable { name: String, value: String },
}
