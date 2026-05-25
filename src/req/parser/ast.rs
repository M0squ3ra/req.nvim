#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqDocument {
    pub requests: Vec<ReqBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqBlock {
    pub name: Option<String>,
    pub start_line: usize,
    pub lines: Vec<ReqLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReqLine {
    Empty,
    Directive(Directive),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    Env(String),
}
