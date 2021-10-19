
use std::io::Read;
use std::path::Path;

pub trait DirectoryContext<R: Read> /* : Clone */ {
    fn open(&self, absolute_filename: &Path) -> Option<R>;
}

#[derive(Debug, Clone)]
pub struct FsDirectoryContext<R: Read> {
    resource_type: std::marker::PhantomData<R>,
    base_dir: String,
}

impl FsDirectoryContext<std::fs::File> {
    pub fn new(base_dir: String) -> FsDirectoryContext<std::fs::File> {
        FsDirectoryContext {
            resource_type: std::marker::PhantomData,
            base_dir,
        }
    }
}

impl<R: Read> DirectoryContext<R> for FsDirectoryContext<R> {
    fn open(&self, absolute_filename: &Path) -> Option<R> {
        None
    }
}
