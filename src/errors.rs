use crate::{CloudServiceProvider, PathKeyInConfig};
use std::{error::Error, fmt::Display, io};

/// JobRunnerError enumerates all possible errors returned by this library.
#[derive(Debug)]
pub enum JobRunnerError {
    /// Represents when a job is referenced by name, but that job doesn't exist
    /// in the provided config file.
    JobNotFoundError { job_name: String },

    /// Represents when the credentials to authenticate with a storage bucket in
    /// the cloud can't be found.
    BucketCredentialsNotFoundError(CloudServiceProvider),

    /// Represents when a Path can't be stringified (typically because it
    /// contains non-UTF-8 characters, which is perfectly valid in some
    /// operating systems).
    InvalidPathError(PathKeyInConfig),

    /// Represents when attempting to download a file from a bucket in the cloud
    /// fails.
    DownloadFromBucketError { source: Box<dyn Error> },

    /// Represents all other cases of [io::Error].
    IOError(io::Error),
}

impl Error for JobRunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::JobNotFoundError { job_name: _ } => None,
            Self::BucketCredentialsNotFoundError(_) => None,
            Self::InvalidPathError(_) => None,
            Self::DownloadFromBucketError { source } => Some(source.as_ref()),
            Self::IOError(_) => None,
        }
    }
}

impl Display for JobRunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobNotFoundError { job_name } => {
                write!(f, "\"{}\" not found in the config file", job_name)
            }
            Self::BucketCredentialsNotFoundError(cloud_service_provider) => {
                match cloud_service_provider {
                    CloudServiceProvider::GCP => {
                        write!(
                            f,
                            "Could not find service account credentials. \
                            Looked for a path to a JSON file on disk in the \
                            \"SERVICE_ACCOUNT\" and \
                            \"GOOGLE_APPLICATION_CREDENTIALS\" environment \
                            variables, and looked for credentials as JSON in \
                            the \"SERVICE_ACCOUNT_JSON\" and \
                            \"GOOGLE_APPLICATION_CREDENTIALS_JSON\" \
                            environment variables."
                        )
                    }
                }
            }
            Self::InvalidPathError(path_key) => {
                write!(
                    f,
                    "Value for \"{}\" in config file can't be stringified",
                    path_key
                )
            }
            Self::DownloadFromBucketError { source } => {
                write!(f, "Failed to download object from bucket: {}", source)
            }
            Self::IOError(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for JobRunnerError {
    fn from(err: io::Error) -> Self {
        JobRunnerError::IOError(err)
    }
}
