/// Note: Methods that take a `&mut self` and return a `Result` might cause de-sync between the internal data and the file if the `Result` is an `Err`
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
pub use std::path::Path;

pub struct FileSync<T>
where
    T: Serialize + DeserializeOwned,
{
    data: T,
    file: File,
    pub pretty: bool,
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
    /// `pretty` determines if iet will use serde_json::to_writer_pretty instead of `serde_json::to_writer`
    ///
    /// Will return an error if a file at that path already exists
    ///
    /// Will return an error if the creating the `File` returns an error
    ///
    /// Will return an error if `self.write` returns an error

    pub fn new(fp: &Path, data: T, pretty: bool) -> Result<Self, FileSyncError> {
        if fp.exists() {
            Err(FileSyncError::FileAlreadyExists { fp })
        } else {
            let file = File::options()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .open(fp)?;
            Self::write(&file, &data, pretty)?;
            Ok(Self { data, file, pretty })
        }
    }

    /// Creates a new `FileSync` type loading and syncing data from an already existing file
    ///
    /// `pretty` determines if iet will use serde_json::to_writer_pretty instead of `serde_json::to_writer`
    ///
    /// Will return an error if the creating the `File` returns an error
    ///
    /// Will return an error if `serde_json::from_reader` returns an error
    pub fn load(fp: &Path, pretty: bool) -> Result<Self, FileSyncError> {
        let file = File::options().read(true).write(true).open(fp)?;
        let data = serde_json::from_reader(&file)?;
        Ok(Self { data, file, pretty })
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
    /// Returns an error if `serde_json::to_writer`/`serde_json::to_writer_pretty` returns an error
    pub fn set(&mut self, data: T) -> Result<(), FileSyncError> {
        self.clear_file();
        Self::write(&self.file, &self.data, self.pretty)?;
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
    /// Returns an error if `self.write` returns an error
    pub fn modify<F>(&mut self, f: F) -> Result<(), FileSyncError>
    where
        F: Fn(&mut T),
    {
        (f)(&mut self.data);
        self.clear_file();
        Self::write(&self.file, &self.data, self.pretty)?;
        Ok(())
    }

    /// Will return an error if `serde_json::to_writer`/`serde_json::to_writer_pretty` fails
    fn write(file: &File, value: &T, pretty: bool) -> Result<(), serde_json::Error> {
        if pretty {
            serde_json::to_writer_pretty(file, value)?;
        } else {
            serde_json::to_writer(file, value)?;
        }
        Ok(())
    }
}
