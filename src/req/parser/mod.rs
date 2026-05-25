//! Parser for the `.req` file format.
//!
//! The parser is split into stages:
//! document parsing builds an AST from raw text, and request parsing lowers the
//! first request block into the executable plugin model.

pub mod ast;
mod directive;
mod document;
mod marker;
mod request;

pub use document::parse_document;
pub use request::parse_request;
