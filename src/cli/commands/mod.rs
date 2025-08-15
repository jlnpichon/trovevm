mod compile;
mod eval;
mod parse;
mod run;
pub use compile::{CompileCommandError, compile};
pub use eval::{EvalError, eval};
pub use parse::{ParseCommandError, parse};
pub use run::{RunError, run};
