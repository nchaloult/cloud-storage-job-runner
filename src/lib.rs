pub mod pretty_print;

use std::{collections::HashMap, error::Error, path::PathBuf};

use serde::Deserialize;

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
    pub async fn run_one(&self, job_name: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
