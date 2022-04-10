mod bucket;
mod step_runner;

use std::{collections::HashMap, error::Error, fmt, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum CloudServiceProvider {
    GCP,
}

#[derive(Debug)]
pub struct InvalidPathError {
    /// Name of the `path_to_*` key that has a `PathBuf` that can't be
    /// stringified.
    ///
    /// # Examples
    /// - `"path_to_remote_inputs"`
    /// - `"path_to_local_inputs"`
    /// - `"path_to_local_outputs"`
    /// - `"path_to_remote_outputs"`
    path_key_name: String,
}

impl Error for InvalidPathError {}

impl fmt::Display for InvalidPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Revisit message. Is this super helpful/explanatory?
        write!(
            f,
            "Value for \"{}\" in config file can't be stringified",
            self.path_key_name
        )
    }
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

impl Job {
    /// Executes a job, from start to finish.
    async fn run<B, S>(&self, bucket: &B, step_runner: &S) -> Result<(), Box<dyn Error>>
    where
        B: bucket::Bucket,
        S: step_runner::StepRunner,
    {
        bucket
            .download_inputs(&self.path_to_remote_inputs, &self.path_to_local_inputs)
            .await?;
        for step in self.get_steps()? {
            step_runner.run_step(&step)?;
        }
        bucket
            .upload_outputs(&self.path_to_local_outputs, &self.path_to_remote_outputs)
            .await
    }

    /// Returns a list of this [Job]'s steps with all of the `[path_to_*_*]`
    /// tags substituted with their corresponding values.
    fn get_steps(&self) -> Result<Vec<String>, InvalidPathError> {
        let mut steps = Vec::new();
        for step in &self.steps {
            // TODO: Revisit this. Is there a more modular way?
            let s = step
                .replace(
                    "[path_to_remote_inputs]",
                    self.path_to_remote_inputs
                        .to_str()
                        .ok_or(InvalidPathError {
                            // TODO: Revisit this cloning. Can you get fancy
                            // with lifetimes?
                            path_key_name: "path_to_remote_inputs".to_string(),
                        })?,
                )
                .replace(
                    "[path_to_local_inputs]",
                    self.path_to_local_inputs.to_str().ok_or(InvalidPathError {
                        // TODO: Revisit this cloning. Can you get fancy
                        // with lifetimes?
                        path_key_name: "path_to_local_inputs".to_string(),
                    })?,
                )
                .replace(
                    "[path_to_local_outputs]",
                    self.path_to_local_outputs
                        .to_str()
                        .ok_or(InvalidPathError {
                            // TODO: Revisit this cloning. Can you get fancy
                            // with lifetimes?
                            path_key_name: "path_to_local_outputs".to_string(),
                        })?,
                )
                .replace(
                    "[path_to_remote_outputs]",
                    self.path_to_remote_outputs
                        .to_str()
                        .ok_or(InvalidPathError {
                            // TODO: Revisit this cloning. Can you get fancy
                            // with lifetimes?
                            path_key_name: "path_to_remote_outputs".to_string(),
                        })?,
                );
            steps.push(s);
        }
        Ok(steps)
    }
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

/// Collection of all the stuff that's needed to run jobs.
pub struct Context<'a> {
    config: &'a Config,
    /// Counter that keeps track of which job we're currently running.
    job_counter: u8,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            job_counter: 0,
        }
    }

    /// Runs all jobs.
    pub async fn run_all(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO: Find some way to block on each of these calls? What if two
        // jobs' path_to_local_inputs are the same?
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
    pub async fn run_one(&mut self, job_name: &str) -> Result<(), Box<dyn Error>> {
        let job = self.config.jobs.get(job_name).ok_or(JobNotFoundError {
            // TODO: Revisit this cloning. Can you get fancy with
            // lifetimes?
            job_name: job_name.to_string(),
        })?;
        let bucket = match job.cloud_service_provider {
            CloudServiceProvider::GCP => {
                // TODO: Revisit this cloning. Can you get fancy with lifetimes?
                bucket::gcp::CloudStorageBucket::new(job.bucket_name.clone())?
            }
        };
        let step_runner = step_runner::ShellStepRunner {};
        self.print_running_job_status_message(job_name);
        job.run(&bucket, &step_runner).await
    }

    fn print_running_job_status_message(&mut self, job_name: &str) {
        self.job_counter += 1;
        let num_jobs = self.config.jobs.len();
        println!(
            "[{}/{}] Running {}...",
            self.job_counter, num_jobs, job_name
        );
    }
}
