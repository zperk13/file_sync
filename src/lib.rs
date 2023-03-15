//! Note: Methods that take a `&mut self` and return a [`Result`] might cause de-sync between the internal data and the file if the [`Result`] is an [`Err`]
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
#[doc(no_inline)]
pub use std::path::Path;

// Note: Methods that take a `&mut self` and return a [`Result`] might cause de-sync between the internal data and the file if the [`Result`] is an [`Err`]
#[derive(Debug)]
pub struct FileSync<T>
where
    T: Serialize + DeserializeOwned,
{
    data: T,
    file: File,
    /// Specifies if when writing to the file if [`serde_json::to_writer_pretty`] will be used instead of [`serde_json::to_writer`]
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
    /// `pretty` determines if it will use [`serde_json::to_writer_pretty`] instead of [`serde_json::to_writer`]
    ///
    /// # Errors
    ///
    /// Will return an error if a file at that path already exists
    ///
    /// Will return an error if the creating the [`File`] returns an error
    ///
    /// Will return an error if [`serde_json::to_writer`]/[`serde_json::to_writer_pretty`] returns an error
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
    /// `pretty` determines if iet will use [`serde_json::to_writer_pretty`] instead of [`serde_json::to_writer`]
    ///
    /// # Errors
    ///
    /// Will return an error if the creating the [`File`] returns an error
    ///
    /// Will return an error if [`serde_json::from_reader`] returns an error
    pub fn load(fp: &Path, pretty: bool) -> Result<Self, FileSyncError> {
        let file = File::options().read(true).write(true).open(fp)?;
        let data = serde_json::from_reader(&file)?;
        Ok(Self { data, file, pretty })
    }

    /// Creates a new `FileSync` type loading and syncing data from an already existing file, or creating a new one if the file doesn't exist
    ///
    /// `pretty` determines if iet will use serde_json::to_writer_pretty instead of [`serde_json::to_writer`]
    ///
    /// # Errors
    ///
    /// Will return an error if the creating the [`File`] returns an error
    ///
    /// Will return an error if [`serde_json::to_writer`]/[`serde_json::to_writer_pretty`] returns an error
    ///
    /// Will return an error if [`serde_json::from_reader`] returns an error
    pub fn load_or_new(fp: &Path, data: T, pretty: bool) -> Result<Self, FileSyncError> {
        if fp.exists() {
            FileSync::load(fp, pretty)
        } else {
            FileSync::new(fp, data, pretty)
        }
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

    /// Sets the value of the stored data
    ///
    /// # Panics
    ///
    /// Panics if it fails to clear the file
    ///
    /// # Errors
    ///
    /// Returns an error if [`serde_json::to_writer`]/[`serde_json::to_writer_pretty`] returns an error
    pub fn set(&mut self, data: T) -> Result<(), FileSyncError> {
        self.clear_file();
        Self::write(&self.file, &self.data, self.pretty)?;
        self.data = data;
        Ok(())
    }

    /// Returns an immutable reference to the stored data
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Modifies data and syncs the modified data to the file given a `Fn(&mut T)`
    ///
    /// # Panics
    ///
    /// Panics if it fails to clear the file
    ///
    /// # Errors
    ///
    /// Returns an error if [`serde_json::to_writer`]/[`serde_json::to_writer_pretty`] returns an error
    pub fn modify<F>(&mut self, f: F) -> Result<(), FileSyncError>
    where
        F: FnOnce(&mut T),
    {
        (f)(&mut self.data);
        self.clear_file();
        Self::write(&self.file, &self.data, self.pretty)?;
        Ok(())
    }

    /// # Errors
    ///
    /// Will return an error if [`serde_json::to_writer`]/[`serde_json::to_writer_pretty`] fails
    fn write(file: &File, value: &T, pretty: bool) -> Result<(), serde_json::Error> {
        if pretty {
            serde_json::to_writer_pretty(file, value)?;
        } else {
            serde_json::to_writer(file, value)?;
        }
        Ok(())
    }
}

impl<T> std::ops::Deref for FileSync<T>
where
    T: Serialize + DeserializeOwned,
{
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> std::convert::AsRef<T> for FileSync<T>
where
    T: Serialize + DeserializeOwned,
{
    fn as_ref(&self) -> &T {
        self.get()
    }
}
