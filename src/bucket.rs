use std::{error::Error, fmt, fs, path::Path};

use async_trait::async_trait;
use cloud_storage::{Client, ListRequest};
use tokio_stream::StreamExt;

#[derive(Debug)]
struct CredentialsNotFoundError {}

impl Error for CredentialsNotFoundError {}

impl fmt::Display for CredentialsNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not find service account credentials. Looked for a \
        path to a JSON file on disk in the \"SERVICE_ACCOUNT\" and \
        \"GOOGLE_APPLICATION_CREDENTIALS\" environment variables, and looked \
        for JSON credentials themselves in the \"SERVICE_ACCOUNT_JSON\" and \
        \"GOOGLE_APPLICATION_CREDENTIALS_JSON\" environment variables."
        )
    }
}

#[async_trait]
pub trait Bucket {
    /// Downloads the `path_to_remote_inputs` directory, and all its contents,
    /// from a cloud storage bucket, and saves them on disk at
    /// `path_to_local_inputs`.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the `path_to_local_inputs` directory doesn't
    /// exist on disk, and this function fails to create it.
    async fn download_inputs(
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
    async fn upload_outputs(
        &self,
        path_to_local_outputs: &Path,
        path_to_remote_outputs: &Path,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct GoogleCloudStorageBucket {
    bucket_name: String,
    client: Client,
}

impl GoogleCloudStorageBucket {
    /// Returns a new `GoogleCloudStorageBucket` that's authenticated and ready
    /// to download and upload files.
    pub fn new(bucket_name: String) -> Self {
        Self {
            bucket_name,
            client: Client::default(),
        }
    }

    async fn download_object(
        &self,
        remote_file_path: &str,
        local_destination_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let contents = self
            .client
            .object()
            .download(&self.bucket_name, remote_file_path)
            .await?;

        // TODO: This doesn't work on Windows because paths include back-slashes
        // instead of forward-slashes. Revisit remote_file_path's type. Can it
        // be a Path instead of a string?
        let file_name = remote_file_path.split('/').last().unwrap();
        let local_file_path = local_destination_dir.join(file_name);

        fs::create_dir_all(local_destination_dir)?;
        fs::write(local_file_path, contents)?;

        Ok(())
    }
}

#[async_trait]
impl Bucket for GoogleCloudStorageBucket {
    async fn download_inputs(
        &self,
        path_to_remote_inputs: &Path,
        path_to_local_inputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let path_to_remote_inputs_as_string = path_to_remote_inputs
            .to_str()
            .ok_or(super::InvalidPathError {
                path_key_name: "path_to_remote_inputs".to_string(),
            })?
            .to_string();
        let lr = ListRequest {
            prefix: Some(path_to_remote_inputs_as_string),
            ..Default::default()
        };
        let mut object_list_stream =
            Box::pin(self.client.object().list(&self.bucket_name, lr).await?);

        while let Some(object_list) = object_list_stream.next().await {
            match object_list {
                Ok(list) => {
                    for object in list.items {
                        if !super::is_object_a_directory(&object.name) {
                            self.download_object(&object.name, path_to_local_inputs)
                                .await?;
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }

    async fn upload_outputs(
        &self,
        path_to_local_outputs: &Path,
        path_to_remote_outputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
