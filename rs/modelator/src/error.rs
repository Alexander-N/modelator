use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

use crate::model::checker::ApalacheError;

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

    /// An error that occurs when a TLA file does not have a module name.
    #[error("Unable to parse module name of: {0}")]
    MissingTlaFileModuleName(String),

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
    NoTestFound(String),

    /// Tla operator name parse error
    #[error("Unable to parse all operator names in tla module with content: {0}")]
    TlaOperatorNameParseError(String),

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
    ApalacheFailure(ApalacheError),

    /// An error that occurs when the counterexample produced by Apalache is unexpected.
    #[error("Invalid Apalache counterexample: {0}")]
    InvalidApalacheCounterexample(String),

    /// An error that occurs when using the `ureq` crate.
    #[error("Ureq error: {0}")]
    Ureq(String),

    /// An error that occurs when using the `nom` crate.
    #[error("Nom error: {0}")]
    Nom(String),

    /// An error that occurs when parsing a JSON value.
    #[error("JSON parse error: {0}")]
    JsonParseError(String),

    /// An error for unrecognized checker name.
    #[error("Unrecognized checker: {0}")]
    UnrecognizedChecker(String),

    /// An error for unsupported output format.
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<ureq::Error> for Error {
    fn from(err: ureq::Error) -> Self {
        Self::Ureq(err.to_string())
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        Self::Nom(err.to_string())
    }
}

/// Set of possible errors that can occur when running a test using `modelator`.
#[derive(Error, Debug)]
pub enum TestError {
    /// A `modelator` [enum@Error].
    #[error("Error while running modelator: {0}")]
    Modelator(Error),

    /// A error that occurs when a test fails.
    #[error("Unhandled test: {test}")]
    UnhandledTest {
        /// Test content
        test: String,
        /// System under test
        system: String,
    },

    /// A error that occurs when a test fails.
    #[error("Test failed: {message}\n   {location}")]
    FailedTest {
        /// Failure message.
        message: String,
        /// Failure location.
        location: String,
        /// Test content
        test: String,
        /// System under test
        system: String,
    },
}
