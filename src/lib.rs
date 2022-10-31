/// Note: Methods that take a `&mut self` and return a `Result` might cause de-sync between the internal data and the file if the `Result` is an `Err`
use serde::de::DeserializeOwned;
pub use serde::{self, Deserialize, Serialize};
use std::fs::File;
pub use std::path::Path;

pub struct FileSync<T>
where
    T: Serialize + DeserializeOwned,
{
    data: T,
    file: File,
}

#[derive(thiserror::Error, Debug)]
pub enum FileSyncError<'a> {
    #[error("File \"{fp}\" already exists")]
    FileAlreadyExists { fp: &'a Path },
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("serde_json error")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl<T> FileSync<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a new `FileSync` type syncing a file with the path `fp` and `data`
    ///
    /// Will return an error if a file at that path already exists
    ///
    /// Will return an error if the creating the `File` returns an error
    ///
    /// Will return an error if `serde_json::to_writer` fails
    pub fn new(fp: &Path, data: T) -> Result<Self, FileSyncError> {
        if fp.exists() {
            Err(FileSyncError::FileAlreadyExists { fp })
        } else {
            let file = File::options()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .open(fp)?;
            serde_json::to_writer(&file, &data)?;
            Ok(Self { data, file })
        }
    }

    /// Creates a new `FileSync` type loading and syncing data from an already existing file
    ///
    /// Will return an error if the creating the `File` returns an error
    ///
    /// Will return an error if `serde_json::from_reader` returns an error
    pub fn load(fp: &Path) -> Result<Self, FileSyncError> {
        let file = File::options().read(true).write(true).open(fp)?;
        let data = serde_json::from_reader(&file)?;
        Ok(Self { data, file })
    }

    /// Clears the file. Panics on failure
    fn clear_file(&mut self) {
        use std::io::{Seek, SeekFrom};
        self.file
            .set_len(0)
            .expect("Failed to set length of file to 0");
        self.file
            .seek(SeekFrom::Start(0))
            .expect("Failed to seek to beginning of file");
    }

    /// Sets the value of `self`
    ///
    /// Panics if it fails to clear the file
    ///
    /// Returns an error if serde_json::to_writer returns an error
    pub fn set(&mut self, data: T) -> Result<(), FileSyncError> {
        self.clear_file();
        serde_json::to_writer(&self.file, &data)?;
        self.data = data;
        Ok(())
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    /// Modifies data and syncs the modified data to the file given a `Fn(&mut T)`
    ///
    /// Panics if it fails to clear the file
    ///
    /// Returns an error if serde_json::to_writer returns an error
    pub fn modify<F>(&mut self, f: F) -> Result<(), FileSyncError>
    where
        F: Fn(&mut T),
    {
        (f)(&mut self.data);
        self.clear_file();
        serde_json::to_writer(&self.file, &self.data)?;
        Ok(())
    }
}
