use std::{error::Error, io, path::Path};

pub trait Bucket {
    /// Downloads the `path_to_remote_inputs` directory, and all its contents,
    /// from a cloud storage bucket, and saves them on disk at
    /// `path_to_local_inputs`.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the `path_to_local_inputs` directory doesn't
    /// exist on disk, and this function fails to create it.
    fn download_inputs(
        &self,
        path_to_remote_inputs: &Path,
        path_to_local_inputs: &Path,
    ) -> Result<(), Box<dyn Error>>;

    /// Uploads the `path_to_local_outputs` directory, and all its contents,
    /// on disk to a cloud storage bucket at `path_to_remote_outputs`.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the `path_to_local_outputs` directory doesn't
    /// exist on disk.
    fn upload_outputs(
        &self,
        path_to_local_outputs: &Path,
        path_to_remote_outputs: &Path,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct GoogleCloudStorageBucket {}

impl GoogleCloudStorageBucket {
    /// Returns a new `GoogleCloudStorageBucket` that's authenticated and ready
    /// to download and upload files.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if `creds` does not exist or can't be read
    /// because of its file permissions.
    pub fn new(creds: &Path) -> io::Result<Self> {
        // TODO: Authenticate. Return an error if the creds don't exist or are
        // inaccessible due to permissions problems.
        Ok(Self {})
    }
}

impl Bucket for GoogleCloudStorageBucket {
    fn download_inputs(
        &self,
        path_to_remote_inputs: &Path,
        path_to_local_inputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn upload_outputs(
        &self,
        path_to_local_outputs: &Path,
        path_to_remote_outputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
