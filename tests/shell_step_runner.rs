mod common;

use cloud_storage_job_runner::{self, step_runner::shell, CloudServiceProvider, Job};
use std::fs;

#[tokio::test]
async fn shell_step_runner_write_new_files_to_disk() {
    // Arrange

    // TODO: Should this var be a const in common?
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html
    let tmp_dir_path = env!("CARGO_TARGET_TMPDIR");
    let job = Job {
        cloud_service_provider: CloudServiceProvider::GCP,
        bucket_name: "foo".into(),
        path_to_remote_inputs: "foo".into(),
        path_to_local_inputs: format!("{}/foo", tmp_dir_path).into(),
        path_to_local_outputs: format!("{}/bar", tmp_dir_path).into(),
        path_to_remote_outputs: "bar".into(),
        steps: vec!["cp -r [path_to_local_inputs] [path_to_local_outputs]".into()],
    };
    let bucket = common::DummyBucket {};
    let step_runner = shell::Runner {};

    // Act

    job.run(&bucket, &step_runner)
        .await
        .expect("Something went wrong running the job");

    // Assert

    // TODO: Clean up that temporary folder you made.
    fs::remove_dir_all(tmp_dir_path)
        .and_then(|_| fs::create_dir(tmp_dir_path))
        .expect("Something went wrong removing the temp folder after running a test");
}
