mod bucket;
mod step_runner;

use std::{collections::HashMap, error::Error, fmt, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum CloudServiceProvider {
    GCP,
}

#[derive(Debug, Deserialize)]
pub struct Job {
    pub cloud_service_provider: CloudServiceProvider,
    pub path_to_auth_key: PathBuf,
    pub bucket_name: String,
    pub path_to_remote_inputs: PathBuf,
    pub path_to_local_inputs: PathBuf,
    pub path_to_local_outputs: PathBuf,
    pub path_to_remote_outputs: PathBuf,
    pub steps: Vec<String>,
}

#[derive(Debug)]
pub struct JobNotFoundError {
    /// Name of job that cannot be found in the config file.
    job_name: String,
}

impl Error for JobNotFoundError {}

impl fmt::Display for JobNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"{}\" not found in the config file", self.job_name)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub jobs: HashMap<String, Job>,
}

impl Config {
    /// Runs all jobs.
    pub fn run_all(&self) -> Result<(), Box<dyn Error>> {
        // TODO: Parallelize?
        // Maybe not. What if two jobs' path_to_local_inputs are the same?
        for j in self.jobs.keys() {
            self.run_one(j)?;
        }

        Ok(())
    }

    /// Runs the job with the provided `job_name`.
    pub fn run_one(&self, job_name: &str) -> Result<(), Box<dyn Error>> {
        let job = match self.jobs.get(job_name) {
            Some(j) => j,
            None => {
                return Err(Box::new(JobNotFoundError {
                    // TODO: Revisit this cloning. Can you get fancy with
                    // lifetimes?
                    job_name: job_name.to_string(),
                }));
            }
        };

        let bucket = match job.cloud_service_provider {
            CloudServiceProvider::GCP => {
                bucket::GoogleCloudStorageBucket::new(&job.path_to_auth_key)?
            }
        };
        let step_runner = step_runner::ShellStepRunner {};
        let job_runner = JobRunner {
            bucket: Box::new(bucket),
            step_runner: Box::new(step_runner),
        };

        job_runner
            .bucket
            .download_inputs(&job.path_to_remote_inputs, &job.path_to_local_inputs)?;
        for step in &job.steps {
            job_runner.step_runner.run_step(step)?;
        }
        job_runner
            .bucket
            .upload_outputs(&job.path_to_local_outputs, &job.path_to_remote_outputs)?;

        Ok(())
    }
}

struct JobRunner {
    bucket: Box<dyn bucket::Bucket>,
    step_runner: Box<dyn step_runner::StepRunner>,
}
