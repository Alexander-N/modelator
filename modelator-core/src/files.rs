use std::path::PathBuf;
use ignore::Walk;
use std::ffi::OsStr;

pub struct Files {}

impl Files {
    
    /// Recursive walk over all files in the current directory
    pub fn walk() -> impl Iterator<Item = PathBuf> {
        Walk::new("./").filter_map(|entry| {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    return Some(entry.path().to_path_buf());
                }
            };
            None
        }
        )
    }

    // A predicate for files that have the given extension
    pub fn with_extension(ext: &'static str) -> Box<dyn FnMut(&PathBuf) -> bool> {
        Box::new(move |path: &PathBuf| path.extension() == Some(OsStr::new(ext)))
    }

    // Example walk over all Jsonnet files
    // for f in Files::walk().filter(Files::with_extension("jsonnet")) {
    //    println!("{}", f.to_string_lossy())
    // }
}
