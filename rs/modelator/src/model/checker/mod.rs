// TLC module.
mod tlc;

// Apalache module.
mod apalache;

// Re-exports.
pub use apalache::{cmd_output::ApalacheError, Apalache};
pub use tlc::Tlc;

use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

const DEFAULT_TRACES_PER_TEST: usize = 1;

/// Set of options to select the model checker to be used and configure them.
#[derive(Clone, Debug)]
pub struct ModelCheckerRuntime {
    /// Which model checker to use.
    pub model_checker: ModelChecker,

    /// Number of model checker worker threads. Possible values: 'auto' to
    /// select the number of worker threads based on the number of available
    /// cores; and any number (e.g. '4') precising the number of workers threads.
    pub workers: ModelCheckerWorkers,

    /// Model checker log file for debugging purposes.
    pub log: PathBuf,

    /// The maximum number of traces to try to generate for a single test.
    pub traces_per_test: usize,
}

impl ModelCheckerRuntime {
    /// Set the model checker.
    pub const fn model_checker(mut self, model_checker: ModelChecker) -> Self {
        self.model_checker = model_checker;
        self
    }

    /// Set number of model checker workers.
    pub const fn workers(mut self, workers: ModelCheckerWorkers) -> Self {
        self.workers = workers;
        self
    }

    /// Set model checker log file.
    pub fn log(mut self, log: impl AsRef<Path>) -> Self {
        self.log = log.as_ref().to_path_buf();
        self
    }

    /// Set the maximum number of traces to try to generate for a single test.
    pub fn traces_per_test(mut self, n: usize) -> Self {
        self.traces_per_test = n;
        self
    }
}

impl Default for ModelCheckerRuntime {
    fn default() -> Self {
        Self {
            model_checker: ModelChecker::Apalache,
            workers: ModelCheckerWorkers::Auto,
            log: Path::new("mc.log").to_path_buf(),
            traces_per_test: DEFAULT_TRACES_PER_TEST,
        }
    }
}

/// Configuration option to select the model checker to be used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelChecker {
    /// Option representing the [TLC](https://github.com/tlaplus/tlaplus) model
    /// checker.
    Tlc,
    /// Option representing the [Apalache](http://github.com/informalsystems/apalache)
    /// mode checker.
    Apalache,
}

impl FromStr for ModelChecker {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "apalache" => Ok(Self::Apalache),
            "tlc" => Ok(Self::Tlc),
            other => Err(Self::Err::UnrecognizedChecker(other.into())),
        }
    }
}

/// Configuration option to select the number of model checker workers.
#[derive(Clone, Copy, Debug)]
pub enum ModelCheckerWorkers {
    /// Automatically select the number of model checker worker threads based
    /// on the number of available cores.
    Auto,
    /// Number of model checker worker threads.
    Count(usize),
}

impl std::str::FromStr for ModelCheckerWorkers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            _ => {
                if let Ok(count) = s.parse() {
                    Ok(Self::Count(count))
                } else {
                    Err(unsupported(s))
                }
            }
        }
    }
}

fn unsupported(s: &str) -> String {
    format!("unsupported value {:?}", s)
}
