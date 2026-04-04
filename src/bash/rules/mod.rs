//! Security rules for evaluating shell commands.

mod allow_safe;
mod cd_git;
mod chained_push;
mod fd;
mod find;
mod gh;
mod git_allow;
mod git_c;
mod git_deny;
mod insta;
mod journalctl;
mod long_python;
mod modern_alternatives;
mod rm;

pub use allow_safe::*;
pub use cd_git::*;
pub use chained_push::*;
pub use fd::*;
pub use find::*;
pub use gh::*;
pub use git_allow::*;
pub use git_c::*;
pub use git_deny::*;
pub use insta::*;
pub use journalctl::*;
pub use long_python::*;
pub use modern_alternatives::*;
pub use rm::*;
