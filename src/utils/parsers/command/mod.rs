//! Schema-aware hierarchical command parser.
#![expect(dead_code, reason = "implemented for extensibility")]

mod command_parser;
mod command_schema;
mod command_schema_builder;
mod operand_schema;
mod operand_schema_builder;
mod option_schema;
mod option_schema_builder;
mod parsed_command;
mod parsed_option;
mod subcommand;
mod value_constraint;

pub use command_parser::*;
pub use command_schema::*;
pub use command_schema_builder::*;
pub use operand_schema::*;
pub use operand_schema_builder::*;
pub use option_schema::*;
pub use option_schema_builder::*;
pub use parsed_command::*;
pub use parsed_option::*;
pub use subcommand::*;
pub use value_constraint::*;
