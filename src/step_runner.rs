use std::{io, process};

pub trait StepRunner {
    /// Executes the provided `step` command as a child process, returning a
    /// handle to it.
    fn run_step(&self, step: &str) -> io::Result<process::Child>;
}
