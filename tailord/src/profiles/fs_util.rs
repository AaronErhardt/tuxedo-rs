use std::path::{Path, PathBuf};

use zbus::fdo;

pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn read<P: AsRef<Path>>(path: P) -> fdo::Result<Vec<u8>> {
    std::fs::read(path).map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub fn read_link<P: AsRef<Path>>(path: P) -> fdo::Result<PathBuf> {
    std::fs::read_link(path).map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> fdo::Result<()> {
    std::fs::create_dir_all(path).map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> fdo::Result<()> {
    std::fs::remove_file(path).map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> fdo::Result<()> {
    std::os::unix::fs::symlink(original, link).map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> fdo::Result<()> {
    std::fs::rename(from, to).map_err(|err| fdo::Error::IOError(err.to_string()))
}
