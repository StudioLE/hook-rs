//! Schema-aware argument parser for flat flag classification.
#![cfg_attr(not(test), expect(dead_code, reason = "implemented for exensibility"))]
#![expect(unused_imports, reason = "implemented for exensibility")]

mod arg;
mod arg_parser;
mod arg_parser_settings;
mod arg_schema;

pub use arg::*;
pub use arg_parser::*;
pub use arg_parser_settings::*;
pub use arg_schema::*;
