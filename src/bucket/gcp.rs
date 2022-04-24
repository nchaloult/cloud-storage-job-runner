use crate::{
    errors::JobRunnerError::{BucketCredentialsNotFoundError, InvalidPathError},
    CloudServiceProvider, PathKeyInConfig, Result,
};
use async_trait::async_trait;
use cloud_storage::{Client, ListRequest};
use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};
use tokio_stream::StreamExt;

/// Verifies that credentials for a Google Cloud service account are present
/// and accessible.
///
/// The [cloud_storage] crate panics if it can't find credentials. It checks (in
/// the following order) if a path to a JSON file exists on disk in the
/// `SERVICE_ACCOUNT` or `GOOGLE_APPLICATION_CREDENTIALS` environment variables,
/// or if the credentials themselves are in the `SERVICE_ACCOUNT_JSON` or
/// `GOOGLE_APPLICATION_CREDENTIALS_JSON` environment variables. This function
/// is meant to return `false` in the same situations where the [cloud_storage]
/// crate would panic so we can handle things more gracefully.
fn are_auth_creds_present() -> bool {
    if let Ok(path) = env::var("SERVICE_ACCOUNT") {
        if Path::new(&path).exists() {
            return true;
        }
    }
    if let Ok(path) = env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        if Path::new(&path).exists() {
            return true;
        }
    }
    // TODO: Revisit the way we're doing this. Right now, we're just checking
    // if the env var's contents are valid JSON.
    if let Ok(contents) = env::var("SERVICE_ACCOUNT_JSON") {
        if is_valid_json(&contents) {
            return true;
        }
    }
    if let Ok(contents) = env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON") {
        if is_valid_json(&contents) {
            return true;
        }
    }
    false
}

fn is_valid_json(contents: &str) -> bool {
    // TODO: Revisit. This is pretty scrappy LOL.
    contents.starts_with('{') && contents.ends_with('}') && contents.is_ascii()
}

pub struct CloudStorageBucket {
    bucket_name: String,
    client: Client,
}

impl CloudStorageBucket {
    /// Returns a new `CloudStorageBucket` that's authenticated and ready
    /// to download and upload files.
    pub fn new(bucket_name: String) -> Result<Self> {
        if !are_auth_creds_present() {
            return Err(BucketCredentialsNotFoundError(CloudServiceProvider::GCP));
        }
        Ok(Self {
            bucket_name,
            client: Client::default(),
        })
    }

    /// Downloads an object's contents from GCS, and writes it to disk in the
    /// provided `path_to_local_inputs`.
    ///
    /// Creates `path_to_local_inputs` if it doesn't already exist.
    async fn download_object(
        &self,
        remote_file_path: &str,
        path_to_remote_inputs: &Path,
        path_to_local_inputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let contents = self
            .client
            .object()
            .download(&self.bucket_name, remote_file_path)
            .await?;

        // TODO: Is this the right error we should be returning? Feels kinda
        // tightly coupled to our config. I feel like `download_object()`
        // shouldn't know that these Paths came from a config file.
        let path_to_local_inputs_as_string = path_to_local_inputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::LocalInputs))?;
        let mut local_file_path_as_string =
            format!("{}/{}", path_to_local_inputs_as_string, remote_file_path);
        if path_to_remote_inputs.components().next().is_some() {
            // TODO: Is this the right error we should be returning? Feels kinda
            // tightly coupled to our config. I feel like `download_object()`
            // shouldn't know that these Paths came from a config file.
            let path_to_remote_inputs_as_string = path_to_remote_inputs
                .to_str()
                .ok_or(InvalidPathError(PathKeyInConfig::RemoteInputs))?;
            // TODO: Is there a better way to do this than to go from PathBufs
            // to Strings and back to a PathBuf?
            local_file_path_as_string = remote_file_path.replace(
                path_to_remote_inputs_as_string,
                path_to_local_inputs_as_string,
            );
        }
        let local_file_path = Path::new(&local_file_path_as_string);

        // If the file lives inside a directory (or directories), make those.
        if let Some(local_file_dir) = local_file_path.parent() {
            fs::create_dir_all(local_file_dir)?;
        }
        fs::write(local_file_path, contents)?;
        Ok(())
    }

    async fn upload_object(
        &self,
        local_file_path: &Path,
        path_to_local_outputs: &Path,
        path_to_remote_outputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: Return an error with more context instead of panicking.
        // Something like: your operating system supports file paths that are
        // not valid unicode, and we can't deal lol.
        let local_file_path_as_string = local_file_path
            .to_str()
            .expect("csjr doesn't support paths that contain invalid unicode");
        // TODO: Are these the right errors we should be returning? Feels kinda
        // tightly coupled to our config. I feel like `download_object()`
        // shouldn't know that these Paths came from a config file.
        let path_to_local_outputs_as_string = path_to_local_outputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::LocalOutputs))?;
        let path_to_remote_outputs_as_string = path_to_remote_outputs
            .to_str()
            .ok_or(InvalidPathError(PathKeyInConfig::RemoteOutputs))?;
        // TODO: Is there a better way to do this than to go from PathBufs to
        // Strings?
        let remote_file_path_as_string = local_file_path_as_string.replace(
            path_to_local_outputs_as_string,
            path_to_remote_outputs_as_string,
        );

        let contents = fs::read(&local_file_path)?;
        let mime_type = mime_guess::from_path(&local_file_path)
            .first_or_octet_stream()
            .to_string();
        self.client
            .object()
            .create(
                &self.bucket_name,
                contents,
                &remote_file_path_as_string,
                &mime_type,
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl super::Bucket for CloudStorageBucket {
    async fn download_inputs(
        &self,
        path_to_remote_inputs: &Path,
        path_to_local_inputs: &Path,
    ) -> Result<(), Box<dyn Error>> {
        // If path_to_remote_inputs points to a folder inside the bucket, only
        // list the objects inside that folder; otherwise, list all objects in
        // the bucket.
        //
        // https://github.com/rust-lang/rust/pull/31877#issuecomment-191901957
        let lr = if path_to_remote_inputs.components().next().is_some() {
            let path_to_remote_inputs_as_string = path_to_remote_inputs
                .to_str()
                .ok_or(InvalidPathError(PathKeyInConfig::RemoteInputs))?
                .to_string();
            ListRequest {
                prefix: Some(path_to_remote_inputs_as_string),
                ..Default::default()
            }
        } else {
            ListRequest::default()
        };
        let mut object_list_stream =
            Box::pin(self.client.object().list(&self.bucket_name, lr).await?);

        while let Some(object_list) = object_list_stream.next().await {
            match object_list {
                Ok(list) => {
                    for object in list.items {
                        if !is_object_a_directory(&object.name) {
                            self.download_object(
                                &object.name,
                                path_to_remote_inputs,
                                path_to_local_inputs,
                            )
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
        for file_path in find_all_files(path_to_local_outputs)? {
            self.upload_object(&file_path, path_to_local_outputs, path_to_remote_outputs)
                .await?;
        }
        Ok(())
    }
}

fn is_object_a_directory(name: &str) -> bool {
    name.ends_with('/')
}

#[cfg(test)]
mod is_object_a_directory_tests {
    #[test]
    fn valid_dir() {
        assert!(super::is_object_a_directory("/"));
        assert!(super::is_object_a_directory("foo/"));
        assert!(super::is_object_a_directory("foo/bar/"));
    }

    #[test]
    fn valid_object() {
        assert!(!super::is_object_a_directory("foo.txt"));
        assert!(!super::is_object_a_directory("foo/bar.txt"));
        assert!(!super::is_object_a_directory("foo/bar/baz.txt"));
        assert!(!super::is_object_a_directory("foo"));
        assert!(!super::is_object_a_directory("foo/bar"));
        assert!(!super::is_object_a_directory("foo/bar/baz"));
    }
}

fn find_all_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // TODO: Should we be using extend()?
                files.append(&mut find_all_files(&path)?)
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}
