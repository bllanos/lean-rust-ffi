use std::cmp::Ordering;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::path::Path;

use super::LakeLibraryDescription;

struct CFile {
    module_name: String,
    path: String,
}

impl Ord for CFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.module_name.cmp(&other.module_name)
    }
}

impl PartialOrd for CFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Directory> for CFile {
    fn partial_cmp(&self, other: &Directory) -> Option<Ordering> {
        Some(self.module_name.cmp(&other.module_name))
    }
}

impl PartialEq for CFile {
    fn eq(&self, other: &Self) -> bool {
        self.module_name == other.module_name
    }
}

impl PartialEq<Directory> for CFile {
    fn eq(&self, other: &Directory) -> bool {
        self.module_name == other.module_name
    }
}

impl Eq for CFile {}

struct Directory {
    module_name: String,
    children: Vec<LakeBuildOutput>,
}

impl Ord for Directory {
    fn cmp(&self, other: &Self) -> Ordering {
        self.module_name.cmp(&other.module_name)
    }
}

impl PartialOrd for Directory {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<CFile> for Directory {
    fn partial_cmp(&self, other: &CFile) -> Option<Ordering> {
        Some(self.module_name.cmp(&other.module_name))
    }
}

impl PartialEq for Directory {
    fn eq(&self, other: &Self) -> bool {
        self.module_name == other.module_name
    }
}

impl PartialEq<CFile> for Directory {
    fn eq(&self, other: &CFile) -> bool {
        self.module_name == other.module_name
    }
}

impl Eq for Directory {}

enum LakeBuildOutput {
    CFile(CFile),
    Directory(Directory),
}

impl Ord for LakeBuildOutput {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::CFile(file) => match other {
                Self::CFile(other_file) => file.cmp(other_file),
                Self::Directory(other_directory) => file.partial_cmp(other_directory).unwrap(),
            },
            Self::Directory(directory) => match other {
                Self::CFile(other_file) => directory.partial_cmp(other_file).unwrap(),
                Self::Directory(other_directory) => directory.cmp(other_directory),
            },
        }
    }
}

impl PartialOrd for LakeBuildOutput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LakeBuildOutput {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::CFile(file) => match other {
                Self::CFile(other_file) => file == other_file,
                Self::Directory(other_directory) => file == other_directory,
            },
            Self::Directory(directory) => match other {
                Self::CFile(other_file) => directory == other_file,
                Self::Directory(other_directory) => directory == other_directory,
            },
        }
    }
}

impl Eq for LakeBuildOutput {}

pub enum LakeBuildOutputTraversalEvent<'a> {
    PushDirectory { module_name: &'a str },
    CFile { path: &'a str, module_name: &'a str },
    PopDirectory,
}

pub trait LakeBuildOutputTraverser {
    fn visit<'a, F, E>(&'a self, callback: &mut F) -> Result<(), E>
    where
        E: Error,
        F: FnMut(LakeBuildOutputTraversalEvent<'a>) -> Result<(), E>;
}

impl LakeBuildOutputTraverser for CFile {
    fn visit<'a, F, E>(&'a self, callback: &mut F) -> Result<(), E>
    where
        E: Error,
        F: FnMut(LakeBuildOutputTraversalEvent<'a>) -> Result<(), E>,
    {
        callback(LakeBuildOutputTraversalEvent::CFile {
            path: &self.path,
            module_name: &self.module_name,
        })
    }
}

impl LakeBuildOutputTraverser for Directory {
    fn visit<'a, F, E>(&'a self, callback: &mut F) -> Result<(), E>
    where
        E: Error,
        F: FnMut(LakeBuildOutputTraversalEvent<'a>) -> Result<(), E>,
    {
        let is_not_root = !self.module_name.is_empty();
        if is_not_root {
            callback(LakeBuildOutputTraversalEvent::PushDirectory {
                module_name: &self.module_name,
            })?;
        }
        for child in self.children.iter() {
            child.visit::<F, E>(callback)?;
        }
        if is_not_root {
            callback(LakeBuildOutputTraversalEvent::PopDirectory)
        } else {
            Ok(())
        }
    }
}

impl LakeBuildOutputTraverser for LakeBuildOutput {
    fn visit<'a, F, E>(&'a self, callback: &mut F) -> Result<(), E>
    where
        E: Error,
        F: FnMut(LakeBuildOutputTraversalEvent<'a>) -> Result<(), E>,
    {
        match self {
            Self::CFile(file) => file.visit(callback),
            Self::Directory(directory) => directory.visit(callback),
        }
    }
}

fn create_module_name(s: &str) -> Result<String, Box<dyn Error>> {
    let module_name: String = s
        .chars()
        .filter(|c| c == &'_' || c.is_ascii_alphanumeric())
        .collect();
    if module_name.is_empty() {
        Err(format!("\"{s}\" could not be converted to a valid module name",).into())
    } else {
        Ok(module_name)
    }
}

impl LakeBuildOutput {
    fn find_children<P: AsRef<Path>>(directory: P) -> Result<Vec<LakeBuildOutput>, Box<dyn Error>> {
        let mut children = directory
            .as_ref()
            .read_dir()?
            .filter_map(|result| match result {
                Ok(entry) => match Self::convert_dir_entry(entry) {
                    Ok(option) => option.map(Ok),
                    Err(e) => Some(Err(e)),
                },
                Err(e) => Some(Err(e.into())),
            })
            .collect::<Result<Vec<LakeBuildOutput>, _>>()?;
        children.sort();
        Ok(children)
    }

    fn convert_dir_entry(value: DirEntry) -> Result<Option<Self>, Box<dyn Error>> {
        let file_type = value.file_type()?;
        if file_type.is_file() {
            let path = value.path();
            if path.extension() == Some(OsStr::new("c")) {
                let stem = String::from_utf8(
                    path.file_stem()
                        .ok_or_else(|| format!("no file stem found in \"{}\"", path.display()))?
                        .as_encoded_bytes()
                        .to_vec(),
                )
                .map_err(|_| format!("file path \"{}\" is not valid UTF-8", path.display()))?;
                let module_name = format!("{}_c", create_module_name(&stem)?);

                let path_str = String::from_utf8(path.into_os_string().into_encoded_bytes())
                    .map_err(|err| {
                        format!(
                            "file path \"{}\" is not valid UTF-8",
                            Path::new(unsafe {
                                OsStr::from_encoded_bytes_unchecked(err.as_bytes())
                            })
                            .display()
                        )
                    })?;
                Ok(Some(Self::CFile(CFile {
                    path: path_str,
                    module_name,
                })))
            } else {
                Ok(None)
            }
        } else if file_type.is_dir() {
            let path = value.path();
            let children = Self::find_children(&path)?;
            if children.is_empty() {
                Ok(None)
            } else {
                let name = String::from_utf8(
                    path.file_name()
                        .ok_or_else(|| format!("no file name found in \"{}\"", path.display()))?
                        .as_encoded_bytes()
                        .to_vec(),
                )
                .map_err(|_| format!("directory path \"{}\" is not valid UTF-8", path.display()))?;
                let module_name = create_module_name(&name)?;
                Ok(Some(Self::Directory(Directory {
                    children,
                    module_name,
                })))
            }
        } else {
            Ok(None)
        }
    }

    fn traverse_path<P: AsRef<Path>>(base_path: P) -> Result<Self, Box<dyn Error>> {
        let children = Self::find_children(base_path)?;
        Ok(Self::Directory(Directory {
            children,
            module_name: String::new(),
        }))
    }
}

pub fn find_c_files<P: AsRef<Path>, Q: AsRef<OsStr>, R: AsRef<Path>, S: AsRef<Path>>(
    lake_library_description: &LakeLibraryDescription<P, Q, R, S>,
) -> Result<impl LakeBuildOutputTraverser, Box<dyn Error>> {
    LakeBuildOutput::traverse_path(lake_library_description.get_c_files_directory())
}
