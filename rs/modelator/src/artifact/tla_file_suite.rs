use std::collections::BTreeSet;
use std::path::PathBuf;

use super::tla_config_file::TlaConfigFile;
use super::tla_file::TlaFile;
use super::{Artifact, ArtifactCreator, ArtifactSaver};
use crate::Error;

// https://github.com/tlaplus/tlaplus/tree/master/tlatools/org.lamport.tlatools/src/tla2sany/StandardModules
const STANDARD_MODULES: [&str; 12] = [
    "Bags",
    "FiniteSets",
    "Integers",
    "Json",
    "Naturals",
    "Randomization",
    "Reals",
    "RealTime",
    "Sequences",
    "TLC",
    "TLCExt",
    "Toolbox",
];

/// TODO: split module and cfg into two parts and contain the main module and extended modules in module struct
/// An in-memory representation of all the resources needed to perform model checking
/// Includes the main .tla and .cfg files as well as depended on (via EXTENDS) .tla files.
#[derive(Debug)]
pub struct TlaFileSuite {
    /// The tla file being used as a target for a model checker command
    pub tla_file: TlaFile,
    /// The config file being used for a model checker command
    pub tla_config_file: TlaConfigFile,
    /// Depended-on TLA files (via transitive closure of EXTENDS)
    pub dependency_tla_files: Vec<TlaFile>,
}

fn find_dependencies(tla_module_path: impl AsRef<std::path::Path>) -> Result<Vec<PathBuf>, Error> {
    let current_directory = tla_module_path
        .as_ref()
        .parent()
        .expect("expected a final component")
        .to_path_buf();

    let content = crate::util::try_read_file_contents(tla_module_path)?;

    Ok(content
        .lines()
        .filter(|line| line.starts_with("EXTENDS"))
        .flat_map(|line| line.trim_start_matches("EXTENDS").split(',').map(str::trim))
        .filter(|module_name| !STANDARD_MODULES.contains(module_name))
        .map(|module_name| current_directory.join(format!("{}.tla", module_name)))
        .collect())
}

fn gather_dependencies(
    tla_module_path: impl AsRef<std::path::Path>,
) -> Result<Vec<TlaFile>, Error> {
    let mut extended_modules = find_dependencies(tla_module_path)?;

    let mut explored_set = BTreeSet::new();

    // BFS
    while let Some(current_module_path) = extended_modules.pop() {
        if !explored_set.contains(&current_module_path) {
            explored_set.insert(current_module_path.clone());
            let new_extended_modules = find_dependencies(current_module_path)?;
            extended_modules.extend(new_extended_modules.into_iter());
        }
    }

    explored_set
        .into_iter()
        .map(|tla_module_path| {
            let content = crate::util::try_read_file_contents(tla_module_path)?;
            TlaFile::from_string(&content)
        })
        .collect()
}

impl TlaFileSuite {
    /// Gather all model checking resources from a main .tla and .cfg file
    pub fn from_tla_and_config_paths<P: AsRef<std::path::Path>>(
        tla_file_path: P,
        config_file_path: P,
    ) -> Result<Self, Error> {
        let tla_file = TlaFile::try_read_from_file(&tla_file_path)?;
        let mut tla_config_file = TlaConfigFile::try_read_from_file(config_file_path)?;
        tla_config_file.set_path(std::path::Path::new(&format!(
            "{}.cfg",
            tla_file.module_name()
        )));
        let dependency_tla_files = gather_dependencies(tla_file_path)?;
        Ok(Self {
            tla_file,
            tla_config_file,
            dependency_tla_files,
        })
    }

    /// Gather all resources from a main .tla without .cfg file
    pub fn from_tla_path<P: AsRef<std::path::Path>>(tla_file_path: P) -> Result<Self, Error> {
        let tla_file = TlaFile::try_read_from_file(&tla_file_path)?;
        let tla_config_file = TlaConfigFile::from_string("")?;
        let dependency_tla_files = gather_dependencies(tla_file_path)?;
        Ok(Self {
            tla_file,
            tla_config_file,
            dependency_tla_files,
        })
    }
}

impl<'a> IntoIterator for &'a TlaFileSuite {
    type Item = Box<&'a dyn ArtifactSaver>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut ret: Vec<Self::Item> = Vec::new();
        for f in &self.dependency_tla_files {
            ret.push(Box::new(f));
        }
        ret.push(Box::new(&self.tla_file));
        if !self.tla_config_file.content().is_empty() {
            ret.push(Box::new(&self.tla_config_file));
        }
        ret.into_iter()
    }
}
