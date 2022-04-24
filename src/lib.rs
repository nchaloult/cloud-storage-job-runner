mod errors;
pub mod pretty_print;

use errors::JobRunnerError;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, path::PathBuf};

type Result<T, E = JobRunnerError> = std::result::Result<T, E>;

/// Representations of the different keys in a config file whose values are
/// [PathBuf]s.
#[derive(Debug)]
pub enum PathKeyInConfig {
    RemoteInputs,
    LocalInputs,
    LocalOutputs,
    RemoteOutputs,
}

impl Display for PathKeyInConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathKeyInConfig::RemoteInputs => write!(f, "remote_inputs"),
            PathKeyInConfig::LocalInputs => write!(f, "local_inputs"),
            PathKeyInConfig::LocalOutputs => write!(f, "local_outputs"),
            PathKeyInConfig::RemoteOutputs => write!(f, "remote_outputs"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum CloudServiceProvider {
    GCP,
}

#[derive(Debug, Deserialize)]
pub struct Job {
    pub cloud_service_provider: CloudServiceProvider,
    pub bucket_name: String,
    pub path_to_remote_inputs: PathBuf,
    pub path_to_local_inputs: PathBuf,
    pub path_to_local_outputs: PathBuf,
    pub path_to_remote_outputs: PathBuf,
    pub steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub jobs: HashMap<String, Job>,
}

pub struct JobRunner {
    config: Config,
    /// Counter that keeps track of which job we're currently running.
    job_counter: u8,
}

impl JobRunner {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            job_counter: 0,
        }
    }

    pub async fn run_all(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    /// Runs the job with the provided `job_name`.
    ///
    /// Fetches the [Job] with the name `job_name`, grabs the appropriate
    /// [bucket::Bucket] and [step_runner::StepRunner] implementations, and
    /// calls the job's `run()` method.
    pub async fn run_one(&self, job_name: &str) -> Result<()> {
        todo!()
    }
}
