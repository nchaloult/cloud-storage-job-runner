use cloud_storage_job_runner::{pretty_print, Config, JobRunner};
use std::{fs::File, path::PathBuf, process};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cloud-storage-job-runner",
    about = "Download files from a storage bucket in the cloud, run a job on each of them, and upload the results back to the cloud.",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
struct Opt {
    /// Path to config file
    #[structopt(short, long, parse(from_os_str))]
    config: PathBuf,
    /// Name of job to run. If not present, runs all jobs specified in the provided config file
    #[structopt()]
    job_name: Option<String>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    // TODO: Display friendlier error messages instead of panicking in these
    // situations.
    let config_file = File::open(opt.config).expect("config file couldn't be opened");
    let config: Config =
        serde_yaml::from_reader(config_file).expect("config file's contents are invalid");
    let mut job_runner = JobRunner::new(&config);

    match opt.job_name {
        Some(j) => {
            if let Err(e) = job_runner.run_one(&j).await {
                if pretty_print::error(&e).is_err() {
                    eprintln!("Something went wrong displaying an error message");
                }
                process::exit(1);
            }
        }
        None => {
            if let Err(e) = job_runner.run_all().await {
                if pretty_print::error(&e).is_err() {
                    eprintln!("Something went wrong displaying an error message");
                }
                process::exit(1);
            }
        }
    };
}
