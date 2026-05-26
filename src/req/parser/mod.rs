//! Parser for the `.req` file format.
//!
//! Public contract:
//! raw `.req` text in, parser AST out.
//!
//! Lowering this AST into the executable request model is handled by
//! `crate::req::lowering`.

pub mod ast;
mod document;
mod grammar;
mod lexer;
pub mod span;

pub use document::parse;
