use std::{error, fmt, process};

#[derive(Debug)]
struct InvalidStepError {
    step: String,
}

impl error::Error for InvalidStepError {}

impl fmt::Display for InvalidStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Improve error message. Can we explain what about the step is
        // invalid?
        write!(f, "invalid step: \"{}\"", self.step)
    }
}

#[derive(Debug)]
struct NonZeroStatusCodeError {
    step: String,
    code: Option<i32>,
}

impl error::Error for NonZeroStatusCodeError {}

impl fmt::Display for NonZeroStatusCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => write!(
                f,
                "\"{}\" exited with a non-zero status code: {}",
                self.step, code
            ),
            None => write!(f, "\"{}\" was terminated by a signal", self.step),
        }
    }
}

pub trait StepRunner {
    /// Executes the provided `step` command as a child process. Echos the
    /// child's `stdout` and `stderr` pipes, and blocks until the step
    /// completes.
    fn run_step(&self, step: &str) -> Result<(), Box<dyn error::Error>>;
}

pub struct ShellStepRunner {}

impl StepRunner for ShellStepRunner {
    fn run_step(&self, step: &str) -> Result<(), Box<dyn error::Error>> {
        // Build a process::Command.
        let mut iter = step.split(' ');

        let program_name = iter.next().ok_or(InvalidStepError {
            // TODO: Revisit this cloning. Can you get fancy with lifetimes?
            step: step.to_string(),
        })?;
        let mut command = &mut process::Command::new(program_name);

        for arg in iter {
            // TODO: Write "[path_to_*_*]" substitution logic.
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
            Err(Box::new(NonZeroStatusCodeError {
                step: step.to_string(),
                code: status.code(),
            }))
        }
    }
}
