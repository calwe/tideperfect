use std::{any::type_name, fmt::Debug, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use snafu::{Snafu, ResultExt};
use tracing::{instrument, trace, warn};

const FILE_EXTENSION: &str = ".json";

/// Allows a type to be persisted to a file.
///
/// ## Warning
///
/// The default identifier() implementation assumes the type has a unique name. If this isn't the
/// case, implement the function yourself with the desired filename (minus the extention).
pub trait PersistenceContext: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug {
    fn identifier() -> String {
        let full_name = type_name::<Self>();
        let short_name = full_name.split("::").last().unwrap_or(full_name);
        short_name.to_lowercase()
    }
}

#[derive(Debug)]
pub struct Persistence {
    data_dir: PathBuf,
}

impl Persistence {
    #[instrument]
    pub fn new(data_dir: &Path) -> Result<Self, PersistanceError> {
        trace!("Creating persistence manager");
        let data_dir = &data_dir.join(Path::new("tideperfect"));
        std::fs::create_dir_all(data_dir).context(WriteSnafu { path: data_dir.to_path_buf() })?;

        Ok(Self {
            data_dir: data_dir.to_owned(),
        })
    }

    #[instrument]
    fn get_path<T: PersistenceContext>(&self) -> PathBuf {
        self.data_dir.join(format!("{}{}", T::identifier(), FILE_EXTENSION))
    }

    /// Stores the data. Overwrites existing data.
    #[instrument(err)]
    pub fn store<T: PersistenceContext>(&self, data: &T) -> Result<(), PersistanceError> {
        let path = self.get_path::<T>();

        trace!("Writing data to {}", path.display());

        let writer = std::fs::File::create(&path).context(WriteSnafu { path: path.to_path_buf() })?;

        serde_json::to_writer_pretty(writer, data).context(SerializeSnafu { data_type: type_name::<T>() })
    }

    /// Load data from persistent storage
    #[instrument(err)]
    pub fn load<T: PersistenceContext>(&self) -> Result<T, PersistanceError> {
        let path = self.get_path::<T>();

        trace!("Loading data from {}", path.display());

        if !path.exists() {
            warn!("File '{}' does not exist", path.display());
            return Err(PersistanceError::FileDoesNotExist { path: path.to_path_buf() })
        }

        let reader = std::fs::File::open(&path).context(ReadSnafu { path: path.to_path_buf() })?;

        serde_json::from_reader(reader).context(DeserializeSnafu { data_type: type_name::<T>(), path: path.to_path_buf() })
    }
}

#[derive(Debug, Snafu)]
pub enum PersistanceError {
    #[snafu(display("File '{}' does not exist", path.display()))]
    FileDoesNotExist {
        path: PathBuf,
    },
    #[snafu(display("Could not read from '{}'", path.display()))]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not write to '{}'", path.display()))]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Failed to serialize '{data_type}'"))]
    Serialize {
        data_type: String,
        source: serde_json::Error,
    },
    #[snafu(display("Failed to deserialize '{data_type}' from '{}'", path.display()))]
    Deserialize {
        data_type: String,
        path: PathBuf,
        source: serde_json::Error,
    },
}

