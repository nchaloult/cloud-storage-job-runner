use std::{fs, path::Path};

use async_trait::async_trait;
use cloud_storage_job_runner::{bucket::Bucket, Result};

/// Mocked implementation of a [Bucket]. Useful for writing integration tests
/// where interactions with a file storage service in the cloud aren't what's
/// being tested.
pub struct DummyBucket {}

#[async_trait]
impl Bucket for DummyBucket {
    /// Pretends to download files from the provided `path_to_remote_inputs`
    /// directory in the cloud. Creates the provided `path_to_local_inputs`
    /// directory on disk, and writes a single text file in that directory.
    async fn download_inputs(
        &self,
        _path_to_remote_inputs: &Path,
        path_to_local_inputs: &Path,
    ) -> Result<()> {
        fs::create_dir(path_to_local_inputs)?;
        fs::write(
            path_to_local_inputs.join("foo.txt"),
            "Whoever is the owner of the white sedan, you left your lights on.",
        )?;
        Ok(())
    }

    /// Pretends to upload files in the provided `path_to_local_outputs`
    /// directory to the cloud. In reality, it does nothing.
    async fn upload_outputs(
        &self,
        _path_to_local_outputs: &Path,
        _path_to_remote_outputs: &Path,
    ) -> Result<()> {
        Ok(())
    }
}
