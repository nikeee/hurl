use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

use hurl_core::ast::Filename;

pub trait DirectoryContext<R: Read> {
    fn exists(&self, filename: &Filename) -> bool;
    fn open(&self, filename: &Filename) -> Result<R>;
    fn get_absolute_filename(&self, filename: &Filename) -> String;
}

#[derive(Debug, Clone)]
pub struct FsDirectoryContext<R: Read> {
    resource_type: std::marker::PhantomData<R>,
    base_dir: String,
}

impl FsDirectoryContext<File> {
    pub fn new(base_dir: String) -> FsDirectoryContext<File> {
        FsDirectoryContext {
            resource_type: std::marker::PhantomData,
            base_dir,
        }
    }
}

impl<R: Read> DirectoryContext<File> for FsDirectoryContext<R> {
    fn exists(&self, filename: &Filename) -> bool {
        let absolute_filename = self.get_absolute_filename(filename);

        Path::new(&absolute_filename).exists()
    }

    fn open(&self, filename: &Filename) -> Result<File> {
        let absolute_filename = self.get_absolute_filename(filename);

        File::open(absolute_filename)
    }

    fn get_absolute_filename(&self, filename: &Filename) -> String {
        let f = filename.value.as_str();
        let path = Path::new(f);

        if path.is_absolute() {
            filename.value.clone()
        } else {
            Path::new(self.base_dir.as_str())
                .join(f)
                .to_str()
                .unwrap()
                .to_string()
        }
    }
}
