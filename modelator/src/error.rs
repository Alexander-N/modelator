use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

/// Set of possible errors that can occur when running `modelator`.
#[allow(clippy::upper_case_acronyms)]
#[derive(Error, Debug, Serialize)]
pub enum Error {
    /// An error that occurs when there's an IO error.
    #[error("IO error: {0}")]
    IO(String),

    /// An error that occurs when invalid unicode is encountered.
    #[error("Invalid unicode: {0:?}")]
    InvalidUnicode(std::ffi::OsString),

    /// An error that occurs when a file is not found.
    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),

    /// An error that occurs when `Java` is not installed.
    #[error("Missing Java. Please install it.")]
    MissingJava,

    /// An error that occurs when the version `Java` installed is too low.
    #[error("Current Java version is: {0}. Minimum Java version supported is: {1}")]
    MinimumJavaVersion(usize, usize),

    /// An error that occurs when a TLA+ file representing a set of tests contains no test.
    #[error("No test found in {0}")]
    NoTestFound(std::path::PathBuf),

    /// An error that occurs when the model checker isn't able to generate a test trace.
    #[error("No trace found in {0}")]
    NoTestTraceFound(std::path::PathBuf),

    /// An error that occurs when the output of TLC is unexpected.
    #[error("Invalid TLC output: {0}")]
    InvalidTLCOutput(std::path::PathBuf),

    /// An error that occurs when the output of TLC returns an error.
    #[error("TLC failure: {0}")]
    TLCFailure(String),

    /// An error that occurs when the output of Apalache returns an error.
    #[error("Apalache failure: {0}")]
    ApalacheFailure(String),

    /// An error that occurs when the counterexample produced by Apalache is unexpected.
    #[error("Invalid Apalache counterexample: {0}")]
    InvalidApalacheCounterexample(String),

    /// An error that occurs when using the `ureq` crate.
    #[error("Ureq error: {0}")]
    Ureq(String),

    /// An error that occurs when using the `nom` crate.
    #[error("Nom error: {0}")]
    Nom(String),

    /// An error that occurs when serializing/deserializing value into/from JSON.
    #[error("JSON parse error: {0}")]
    SerdeJson(String),
}

impl Error {
    pub(crate) fn io(err: std::io::Error) -> Error {
        Error::IO(err.to_string())
    }

    pub(crate) fn ureq(err: ureq::Error) -> Error {
        Error::Ureq(err.to_string())
    }

    pub(crate) fn nom(err: nom::Err<nom::error::Error<&str>>) -> Error {
        Error::Nom(err.to_string())
    }

    pub(crate) fn serde_json(err: serde_json::Error) -> Error {
        Error::SerdeJson(err.to_string())
    }
}

/// Set of possible errors that can occur when running a test using `modelator`.
#[derive(Error, Debug)]
pub enum TestError<Step: Debug> {
    /// A `modelator` [enum@Error].
    #[error("Error while running modelator: {0}")]
    Modelator(Error),

    /// A error that occurs when a test fails.
    #[error("Test failed on step {step_index}/{step_count}:\nsteps: {steps:#?}")]
    FailedTest {
        /// The step index at which the test failed.
        step_index: usize,
        /// The total number of steps in the test.
        step_count: usize,
        /// All the steps in the test.
        steps: Vec<Step>,
    },
}
