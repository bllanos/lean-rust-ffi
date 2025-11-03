use std::fs;
use std::io;
use std::path::Path;

pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).ok().as_ref().map(fs::Metadata::is_file) == Some(true)
}

pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).ok().as_ref().map(fs::Metadata::is_dir) == Some(true)
}

pub fn read_file(path: &Path) -> io::Result<String> {
    let mut file = fs::OpenOptions::new().read(true).open(path)?;

    let mut contents = String::new();

    io::Read::read_to_string(&mut file, &mut contents)?;

    Ok(contents)
}

pub fn if_not_empty<S: PartialEq<str>>(s: S) -> Option<S> {
    if s == *"" { None } else { Some(s) }
}
