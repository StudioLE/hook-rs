//! Security rules for evaluating shell commands.

mod allow_safe;
mod cd_git;
mod chained_push;
mod echo_separator;
mod find;
mod gh;
mod git;
mod git_approval;
mod git_checkout;
mod insta;
mod long_python;
mod rm;

pub use allow_safe::*;
pub use cd_git::*;
pub use chained_push::*;
pub use echo_separator::*;
pub use find::*;
pub use gh::*;
pub use git::*;
pub use git_approval::*;
pub use git_checkout::*;
pub use insta::*;
pub use long_python::*;
pub use rm::*;
