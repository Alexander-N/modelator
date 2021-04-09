/// Parsing of Apalache's counterexample file.
mod counterexample;

use crate::artifact::{TlaConfigFile, TlaFile, TlaTrace};
use crate::cache::TlaTraceCache;
use crate::{jar, Error, Options};
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `modelator`'s Apalache module.
#[derive(Debug, Clone, Copy)]
pub struct Apalache;

impl Apalache {
    /// Generate a TLA+ trace given a [TlaFile] and a [TlaConfigFile] produced
    /// by [crate::module::Tla::generate_tests].
    ///
    /// # Examples
    ///
    /// ```
    /// use modelator::artifact::{TlaFile, TlaConfigFile};
    /// use modelator::module::{Tla, Apalache};
    /// use modelator::Options;
    /// use std::convert::TryFrom;
    ///
    /// let tla_tests_file = "tests/integration/tla/NumbersAMaxBMinTest.tla";
    /// let tla_config_file = "tests/integration/tla/Numbers.cfg";
    /// let tla_tests_file = TlaFile::try_from(tla_tests_file).unwrap();
    /// let tla_config_file = TlaConfigFile::try_from(tla_config_file).unwrap();
    ///
    /// let mut tests = Tla::generate_tests(tla_tests_file, tla_config_file).unwrap();
    /// let (tla_test_file, tla_test_config_file) = tests.pop().unwrap();
    /// let options = Options::default();
    /// let tla_trace = Apalache::test(tla_test_file, tla_test_config_file, &options).unwrap();
    /// println!("{:?}", tla_trace);
    /// ```
    pub fn test(
        tla_file: TlaFile,
        tla_config_file: TlaConfigFile,
        options: &Options,
    ) -> Result<TlaTrace, Error> {
        tracing::debug!(
            "Apalache::test {} {} {:?}",
            tla_file,
            tla_config_file,
            options
        );

        // load cache and check if the result is cached
        let mut cache = TlaTraceCache::new(options)?;
        let cache_key = TlaTraceCache::key(&tla_file, &tla_config_file)?;
        if let Some(value) = cache.get(&cache_key)? {
            return Ok(value);
        }

        // create apalache test command
        let cmd = test_cmd(tla_file.path(), tla_config_file.path(), options);

        // run apalache
        run_apalache(cmd, options)?;

        // convert apalache counterexample to a trace
        let counterexample_path = Path::new("counterexample.tla");
        if counterexample_path.is_file() {
            let counterexample = std::fs::read_to_string(counterexample_path).map_err(Error::io)?;
            tracing::debug!("Apalache counterexample:\n{}", counterexample);
            let trace = counterexample::parse(counterexample)?;

            // cache trace and then return it
            cache.insert(cache_key, &trace)?;
            Ok(trace)
        } else {
            panic!("[modelator] expected to find Apalache's counterexample.tla file")
        }
    }

    /// Runs Apalache's `parse` command, returning the [TlaFile] produced by
    /// Apalache.
    ///
    /// # Examples
    ///
    /// ```
    /// use modelator::artifact::TlaFile;
    /// use modelator::module::Apalache;
    /// use modelator::Options;
    /// use std::convert::TryFrom;
    ///
    /// let tla_file = "tests/integration/tla/NumbersAMaxBMinTest.tla";
    /// let tla_file = TlaFile::try_from(tla_file).unwrap();
    ///
    /// let options = Options::default();
    /// let mut tla_parsed_file = Apalache::parse(tla_file, &options).unwrap();
    /// println!("{:?}", tla_parsed_file);
    /// ```
    pub fn parse(tla_file: TlaFile, options: &Options) -> Result<TlaFile, Error> {
        tracing::debug!("Apalache::parse {} {:?}", tla_file, options);

        // parse the tla file, producing also a tla file
        let parsed_file = parse_with_format(tla_file, ApalacheParseFormat::Tla, options)?;

        // create tla file
        use std::convert::TryFrom;
        let tla_parsed_file = TlaFile::try_from(parsed_file)
            .expect("[modelator] apalache should have produced a parsed TLA+ file");
        Ok(tla_parsed_file)
    }

    /// Runs Apalache's `parse` command, and extracts the name of the TLA+
    /// variables in the model.
    ///
    /// # Examples
    ///
    /// ```
    /// use modelator::artifact::TlaFile;
    /// use modelator::module::Apalache;
    /// use modelator::Options;
    /// use std::convert::TryFrom;
    ///
    /// let tla_file = "tests/integration/tla/NumbersAMaxBMinTest.tla";
    /// let tla_file = TlaFile::try_from(tla_file).unwrap();
    ///
    /// let options = Options::default();
    /// let mut vars = Apalache::tla_variables(tla_file, &options).unwrap();
    /// assert_eq!(vars.len(), 2);
    /// assert!(vars.contains("a"));
    /// assert!(vars.contains("b"));
    /// ```
    pub fn tla_variables(tla_file: TlaFile, options: &Options) -> Result<HashSet<String>, Error> {
        tracing::debug!("Apalache::tla_variables {} {:?}", tla_file, options);

        // parse the tla file, producing a json representation of the parsed file
        let parsed_file = parse_with_format(tla_file, ApalacheParseFormat::Json, options)?;

        // parse json file produced by Apalache
        let json = std::fs::read_to_string(parsed_file).map_err(Error::io)?;
        let json: JsonValue = serde_json::from_str(&json)
            .expect("[modelator] json produced by apalache parse should be valid");

        // get tla declarations
        let tla_declarations = json
            .get("declarations")
            .expect("[modelator] json produced by apalache parse should have 'declarations' key")
            .as_array()
            .expect("[modelator] 'declarations' in the json produced by apalache parse should be an array");

        // iterate all declarations and extract tla variables when we find them
        let mut vars = HashSet::new();
        for tla_declaration in tla_declarations {
            if let Some(var) = tla_declaration.get("variable") {
                let var = var.as_str()
                    .expect("[modelator] each 'variable' in 'declarations' in the json produced by apalache parse should be a string")
                    .to_string();
                assert!(
                    vars.insert(var),
                    "[modelator] TLA+ variables should be unique"
                );
            }
        }
        Ok(vars)
    }
}

// Apalache's parse command can produce a TLA file or a JSON file.
// This enum represents the two possible formats.
pub(crate) enum ApalacheParseFormat {
    Tla,
    Json,
}

impl ApalacheParseFormat {
    fn name(&self) -> &str {
        match self {
            Self::Tla => "tla",
            Self::Json => "json",
        }
    }
}

fn parse_with_format(
    tla_file: TlaFile,
    format: ApalacheParseFormat,
    options: &Options,
) -> Result<PathBuf, Error> {
    // compute the directory in which the tla file is stored
    let mut tla_dir = tla_file.path().clone();
    assert!(tla_dir.pop());

    // compute tla module name: it's okay to unwrap as we have already
    // verified that the file exists
    let tla_module_name = tla_file.tla_module_name().unwrap();

    // compute the output tla file
    let parsed_file = tla_dir.join(format!("{}Parsed.{}", tla_module_name, format.name()));

    // create apalache parse command
    let cmd = parse_cmd(tla_file.path(), &parsed_file, options);

    // run apalache
    run_apalache(cmd, options)?;

    Ok(parsed_file)
}

fn run_apalache(mut cmd: Command, options: &Options) -> Result<String, Error> {
    // start apalache
    // TODO: add timeout
    let output = cmd.output().map_err(Error::io)?;

    // get apalache stdout and stderr
    let stdout = crate::util::cmd_output_to_string(&output.stdout);
    let stderr = crate::util::cmd_output_to_string(&output.stderr);
    tracing::debug!("Apalache stdout:\n{}", stdout);
    tracing::debug!("Apalache stderr:\n{}", stderr);

    match (stdout.is_empty(), stderr.is_empty()) {
        (false, true) => {
            // apalache writes all its output to the stdout

            // save apalache log
            std::fs::write(&options.model_checker_options.log, &stdout).map_err(Error::io)?;

            // check if a failure has occurred
            if stdout.contains("EXITCODE: ERROR") {
                return Err(Error::ApalacheFailure(stdout));
            }
            assert!(
                stdout.contains("EXITCODE: OK"),
                "[modelator] unexpected Apalache stdout"
            );
            Ok(stdout)
        }
        _ => {
            panic!("[modelator] unexpected Apalache's stdout/stderr combination")
        }
    }
}

fn test_cmd<P: AsRef<Path>>(tla_file: P, tla_config_file: P, options: &Options) -> Command {
    let mut cmd = apalache_cmd_start(&tla_file, options);
    cmd.arg("check")
        // set tla config file
        .arg(format!(
            "--config={}",
            tla_config_file.as_ref().to_string_lossy()
        ))
        // set tla file
        .arg(tla_file.as_ref());

    tracing::warn!(
        "the following workers option was ignored since apalache is single-threaded: {:?}",
        options.model_checker_options.workers
    );

    // show command being run
    tracing::debug!("{}", crate::util::cmd_show(&cmd));
    cmd
}

fn parse_cmd<P: AsRef<Path>>(tla_file: P, parsed_file: P, options: &Options) -> Command {
    let mut cmd = apalache_cmd_start(&tla_file, options);
    cmd.arg("parse")
        // set output file
        .arg(format!(
            "--output={}",
            parsed_file.as_ref().to_string_lossy()
        ))
        // set tla file
        .arg(tla_file.as_ref());

    // show command being run
    tracing::debug!("{}", crate::util::cmd_show(&cmd));
    cmd
}

fn apalache_cmd_start<P: AsRef<Path>>(tla_file: P, options: &Options) -> Command {
    let apalache = jar::Jar::Apalache.path(&options.dir);

    let mut cmd = Command::new("java");

    // compute the directory where the tla file is, so that it can be added as
    // a tla library
    if let Some(tla_file_dir) = tla_file.as_ref().parent() {
        cmd
            // set tla library
            .arg(format!("-DTLA-Library={}", tla_file_dir.to_string_lossy()));
    }
    cmd
        // set jar
        .arg("-jar")
        .arg(format!("{}", apalache.as_path().to_string_lossy()));
    cmd
}
