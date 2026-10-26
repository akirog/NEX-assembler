mod lexer;
mod errors;
mod parser;
mod ast;
mod semantic_analyzer;
pub mod pipeline;

use colored::Colorize;
use self::lexer::Lexer;
use self::parser::Parser;

pub use self::errors::{CompilerError, Span, SpanPosition};
pub use self::ast::*;