use crate::{
    errors::JobRunnerError::{InvalidStepError, StepNonZeroStatusCodeError},
    Result,
};
use std::process;

pub trait StepRunner {
    /// Executes the provided `step` command as a child process. Echos the
    /// child's `stdout` and `stderr` pipes, and blocks until the step
    /// completes.
    fn run_step(&self, step: &str) -> Result<()>;
}

pub struct ShellStepRunner {}

impl StepRunner for ShellStepRunner {
    fn run_step(&self, step: &str) -> Result<()> {
        // Build a process::Command.
        let mut iter = step.split(' ');

        let program_name = iter
            .next()
            .ok_or_else(|| InvalidStepError { step: step.into() })?;
        let mut command = &mut process::Command::new(program_name);

        for arg in iter {
            command = command.arg(arg);
        }

        // Run the process::Command and wait for it to finish.
        let mut child = command
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .spawn()?;

        let status = child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(StepNonZeroStatusCodeError {
                step: step.into(),
                code: status.code(),
            })
        }
    }
}
