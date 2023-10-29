use bucc_secco::SanitizedPath;
use std::io::{Read, Seek};

#[derive(Debug)]
pub enum ArchiveError {}

pub trait Archive {
    fn open<R>(reader: R, password: Option<String>) -> Result<Self, ArchiveError>
    where
        R: Read + Seek,
        Self: Sized;
    fn extract(&self, path: SanitizedPath) -> Result<(), ArchiveError>;
    fn tree_iter<I: Iterator>(&self) -> I;
}
