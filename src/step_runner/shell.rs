use crate::{
    errors::JobRunnerError::{InvalidStepError, StepNonZeroStatusCodeError},
    Result,
};
use std::process;

pub struct Runner {}

impl super::StepRunner for Runner {
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
