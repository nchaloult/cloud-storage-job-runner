mod bucket;
mod errors;
pub mod pretty_print;
mod step_runner;

use errors::JobRunnerError::{self, InvalidPathError, JobNotFoundError};
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fmt::Display, io, path::PathBuf};

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

pub struct JobRunner<'a> {
    config: &'a Config,
    /// Counter that keeps track of which job we're currently running.
    job_counter: u8,
}

impl<'a> JobRunner<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            job_counter: 0,
        }
    }

    /// Runs all jobs.
    pub async fn run_all(&mut self) -> Result<(), Box<dyn Error>> {
        for j in self.config.jobs.keys() {
            self.run_one(j).await?;
        }
        Ok(())
    }

    /// Runs the job with the provided `job_name`.
    ///
    /// Fetches the [Job] with the name `job_name`, grabs the appropriate
    /// [bucket::Bucket] and [step_runner::StepRunner] implementations, and
    /// calls the job's `run()` method.
    pub async fn run_one(&mut self, job_name: &str) -> Result<()> {
        let job = self
            .config
            .jobs
            .get(job_name)
            .ok_or_else(|| JobNotFoundError {
                job_name: job_name.into(),
            })?;
        let bucket = match job.cloud_service_provider {
            CloudServiceProvider::GCP => bucket::gcp::CloudStorageBucket::new(&job.bucket_name)?,
        };
        let step_runner = step_runner::shell::Runner {};

        self.print_running_job_status_message(job_name)?;
        job.run(&bucket, &step_runner).await
    }

    fn print_running_job_status_message(&mut self, job_name: &str) -> io::Result<()> {
        self.job_counter += 1;
        let num_jobs = self.config.jobs.len();
        pretty_print::status(
            &format!("[{}/{}]", self.job_counter, num_jobs),
            &format!("Running {job_name}..."),
            false,
        )
    }
}

impl Job {
    /// Executes a job, from start to finish.
    async fn run<B, S>(&self, bucket: &B, step_runner: &S) -> Result<()>
    where
        B: bucket::Bucket,
        S: step_runner::StepRunner,
    {
        // TODO: Revisit these unwrap() calls.
        //
        // The situation is kinda weird since we're gonna check for an invalid
        // path in bucket.download_inputs(). Honestly, need to revisit how we're
        // handling errors to do with paths that can't be serialized as Unicode
        // strings all across the project. We shouldn't be handling that in the
        // bucket's impl logic.
        pretty_print::status(
            "Downloading",
            &format!(
                "\"{}\" to \"{}\"",
                self.path_to_remote_inputs.to_str().unwrap(),
                self.path_to_local_inputs.to_str().unwrap()
            ),
            true,
        )?;
        bucket
            .download_inputs(&self.path_to_remote_inputs, &self.path_to_local_inputs)
            .await?;
        for step in self.get_steps()? {
            pretty_print::status("Running", &format!("`{step}`"), true)?;
            step_runner.run_step(&step)?;
        }
        // TODO: Same here: revisit these unwrap() calls.
        //
        // Same situation as before where bucket.upload_outputs() is handling
        // invalid paths, but it really shouldn't.
        pretty_print::status(
            "Uploading",
            &format!(
                "\"{}\" to \"{}\"",
                self.path_to_local_outputs.to_str().unwrap(),
                self.path_to_remote_outputs.to_str().unwrap()
            ),
            true,
        )?;
        bucket
            .upload_outputs(&self.path_to_local_outputs, &self.path_to_remote_outputs)
            .await
    }

    /// Returns a list of this [Job]'s steps with all of the `[path_to_*_*]`
    /// tags substituted with their corresponding values.
    fn get_steps(&self) -> Result<Vec<String>> {
        let path_to_remote_inputs_as_string = self
            .path_to_remote_inputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::RemoteInputs))?;
        let path_to_local_inputs_as_string = self
            .path_to_local_inputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::LocalInputs))?;
        let path_to_local_outputs_as_string = self
            .path_to_local_outputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::LocalOutputs))?;
        let path_to_remote_outputs_as_string = self
            .path_to_remote_outputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::RemoteOutputs))?;

        let mut steps = Vec::new();
        for step in &self.steps {
            // TODO: Revisit this. Is there a more modular way?
            let s = step
                .replace("[path_to_remote_inputs]", path_to_remote_inputs_as_string)
                .replace("[path_to_local_inputs]", path_to_local_inputs_as_string)
                .replace("[path_to_local_outputs]", path_to_local_outputs_as_string)
                .replace("[path_to_remote_outputs]", path_to_remote_outputs_as_string);
            steps.push(s);
        }
        Ok(steps)
    }
}
