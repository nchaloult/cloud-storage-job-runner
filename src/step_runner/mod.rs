pub mod shell;

use crate::Result;

pub trait StepRunner {
    /// Executes the provided `step` command as a child process. Echos the
    /// child's `stdout` and `stderr` pipes, and blocks until the step
    /// completes.
    fn run_step(&self, step: &str) -> Result<()>;
}
