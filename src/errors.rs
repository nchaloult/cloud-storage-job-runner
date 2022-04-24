use std::{error::Error, fmt::Display};

/// JobRunnerError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum JobRunnerError {
    /// Represents when a job is referenced by name, but that job doesn't exist
    /// in the provided config file.
    JobNotFoundError { job_name: String },
}

impl Error for JobRunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            JobRunnerError::JobNotFoundError { job_name: _ } => None,
        }
    }
}

impl Display for JobRunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobNotFoundError { job_name } => {
                write!(f, "\"{}\" not found in the config file", job_name)
            }
        }
    }
}
