use std::{fs::File, path::PathBuf, process};

use cloud_storage_job_runner::{Config, Context};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cloud-storage-job-runner",
    about = "Download files from a storage bucket in the cloud, run a job on each of them, and upload the results back to the cloud.",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
struct Cli {
    /// Path to config file
    #[structopt(short, long, parse(from_os_str))]
    config: PathBuf,
    /// Name of job to run. If not present, runs all jobs specified in the provided config file
    #[structopt()]
    job_name: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::from_args();

    // TODO: Display friendlier error messages instead of panicking in these
    // situations.
    let config_file = File::open(cli.config).expect("config file couldn't be opened");
    let config: Config =
        serde_yaml::from_reader(config_file).expect("config file's contents are invalid");
    let mut ctx = Context::new(&config);

    match cli.job_name {
        Some(j) => {
            if let Err(e) = ctx.run_one(&j).await {
                eprintln!("{e}");
                process::exit(1);
            }
        }
        None => {
            if let Err(e) = ctx.run_all().await {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    };
}
