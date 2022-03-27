use std::path::PathBuf;

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

fn main() {
    let cli = Cli::from_args();
    println!("{cli:?}");
}
