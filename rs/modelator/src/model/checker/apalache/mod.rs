/// Apalache Error
pub(crate) mod cmd_output;
use cmd_output::{ApalacheError, CmdOutput};

/// Parsing of Apalache's counterexample file.
mod counterexample;

use crate::artifact::{
    try_write_to_dir, Artifact, ArtifactCreator, ArtifactSaver, ModelCheckerStdout, TlaConfigFile,
    TlaFile, TlaFileSuite, TlaTrace,
};
use crate::cache::TlaTraceCache;
use crate::model::checker::apalache;
use crate::{jar, Error, ModelatorRuntime};
use std::env::temp_dir;
use std::path::Path;
use std::process::Command;

/// `modelator`'s Apalache module.
#[derive(Debug, Clone, Copy)]
pub struct Apalache;

impl Apalache {
    ///
    /// Generate a TLA+ trace given a [`TlaFile`] and a [`TlaConfigFile`] produced
    /// by [`crate::model::language::Tla::generate_tests`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use modelator::artifact::TlaFileSuite;
    /// use modelator::model::{language::Tla, checker::Apalache};
    /// use modelator::ModelatorRuntime;
    /// use std::convert::TryFrom;
    ///
    /// let tla_tests_file = "tests/integration/resource/NumbersAMaxBMinTest.tla";
    /// let tla_config_file = "tests/integration/resource/Numbers.cfg";
    /// let tla_suite = TlaFileSuite::from_tla_and_config_paths(tla_tests_file, tla_config_file).unwrap();
    ///
    /// let mut tests = Tla::generate_tests(&tla_suite).unwrap();
    /// let test_tla_suite = tests.pop().unwrap();
    /// let runtime = ModelatorRuntime::default();
    /// let (tla_trace, _) = Apalache::test(&test_tla_suite, &runtime).unwrap();
    /// println!("{:?}", tla_trace);
    /// ```
    pub fn test(
        input_artifacts: &TlaFileSuite,
        runtime: &ModelatorRuntime,
    ) -> Result<(Vec<TlaTrace>, ModelCheckerStdout), Error> {
        // TODO: this method currently just uses the paths of the files so no need for whole artifact objects!

        tracing::debug!(
            "Apalache::test {} {} {:?}",
            input_artifacts.tla_file,
            input_artifacts.tla_config_file,
            runtime
        );

        let tdir = tempfile::tempdir()?;

        try_write_to_dir(&tdir, input_artifacts)?;

        // Gets Apalache command with tdir as working dir
        let cmd = apalache_start_cmd(&tdir, runtime);

        // Check if the main tla module contains a View
        // The view will have a generated name 'ViewForTestNeg'
        // If it has one, then use it.
        let view = input_artifacts
            .tla_file
            .file_contents_backing()
            .contains("ViewForTestNeg")
            .then(|| "ViewForTestNeg".to_owned());

        // create 'apalache check' command
        let cmd = check_cmd(
            cmd,
            input_artifacts.tla_file.file_name(),
            input_artifacts.tla_config_file.filename(),
            runtime.model_checker_runtime.traces_per_test,
            &view,
        );

        tracing::warn!(
            "the following workers option was ignored since apalache is single-threaded: {:?}",
            runtime.model_checker_runtime.workers
        );

        let apalache_output = run_apalache(cmd)?;

        let counterexample_paths = apalache_output.parse_counterexample_filenames()?;

        if counterexample_paths.is_empty() {
            return Err(Error::NoTestTraceFound(
                //TODO: this will have to be changed to reflect new in-memory log
                runtime.model_checker_runtime.log.clone(),
            ));
        }

        let traces = counterexample_paths
            .iter()
            .map(|counterexample_path_base| {
                // Read the  apalache counterexample from disk and parse a trace from it
                let counterexample_path = tdir.path().join(&counterexample_path_base);

                if !counterexample_path.is_file() {
                    panic!("[modelator] expected to find Apalache's counterexample1.tla file");
                }

                let counterexample: TlaFile = TlaFile::try_read_from_file(counterexample_path)?;
                tracing::debug!("Apalache counterexample:\n{}", counterexample);
                counterexample::parse(counterexample.file_contents_backing())
            })
            .filter_map(Result::ok)
            .collect();

        Ok((
            traces,
            ModelCheckerStdout::from_string(&apalache_output.stdout.join("\n"))?,
        ))
    }

    ///
    /// Runs Apalache's `parse` command, returning the [`TlaFile`] produced by
    /// Apalache.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use modelator::artifact::TlaFileSuite;
    /// use modelator::model::checker::Apalache;
    /// use modelator::ModelatorRuntime;
    /// use std::convert::TryFrom;
    ///
    /// let tla_file = "tests/integration/resource/NumbersAMaxBMinTest.tla";
    /// let tla_file_suite = TlaFileSuite::from_tla_path(tla_file).unwrap();
    ///
    /// let runtime = ModelatorRuntime::default();
    /// let (tla_parsed_file, _) = Apalache::parse(&tla_file_suite, &runtime).unwrap();
    /// println!("{:?}", tla_parsed_file);
    /// ```
    pub fn parse(
        tla_file_suite: &TlaFileSuite,
        runtime: &ModelatorRuntime,
    ) -> Result<(TlaFile, ModelCheckerStdout), Error> {
        // tracing::debug!("Apalache::parse {} {:?}", tla_file, runtime);

        let tdir = tempfile::tempdir()?;

        try_write_to_dir(&tdir, tla_file_suite)?;

        // Gets Apalache command with tdir as working dir
        let cmd = apalache_start_cmd(&tdir, runtime);

        let tla_file_module_name = tla_file_suite.tla_file.module_name();

        let output_path = format!("{}Parsed.tla", tla_file_module_name);

        // create apalache parse command
        let cmd = parse_cmd(cmd, &tla_file_suite.tla_file.file_name(), &output_path);

        // run apalache
        let apalache_output = run_apalache(cmd)?;

        match apalache_output.non_counterexample_error() {
            None => {}
            Some(err) => return Err(Error::ApalacheFailure(err)),
        }

        // create tla file
        let full_output_path = tdir.path().join(output_path);
        let tla_parsed_file = TlaFile::try_read_from_file(full_output_path)?;
        Ok((
            tla_parsed_file,
            ModelCheckerStdout::from_string(&apalache_output.stdout.join("\n"))?,
        ))
    }
}

fn run_apalache(mut cmd: Command) -> Result<CmdOutput, Error> {
    // TODO: add functionality to kill Apalache after a given timeout
    let output = cmd.output()?;

    // get apalache stdout and stderr
    let stdout = crate::util::cmd_output_to_string(&output.stdout);
    let stderr = crate::util::cmd_output_to_string(&output.stderr);
    let status = output.status.code();
    tracing::debug!("Apalache stdout:\n{}", stdout);
    tracing::debug!("Apalache stderr:\n{}", stderr);

    Ok(CmdOutput {
        stdout: stdout.lines().map(Into::into).collect(),
        stderr: stderr.lines().map(Into::into).collect(),
        status,
    })
}

fn check_cmd<P: AsRef<Path>>(
    mut cmd: Command,
    tla_file_base_name: P,
    tla_config_file_base_name: P,
    max_error: usize,
    view: &Option<String>,
) -> Command {
    cmd.arg("check")
        .arg(format!(
            "--config={}",
            tla_config_file_base_name.as_ref().to_string_lossy()
        ))
        .arg(format!("--max-error={}", max_error))
        .arg("--algo=offline");

    if let Some(view) = view {
        cmd.arg(format!("--view={}", view));
    };

    cmd.arg(tla_file_base_name.as_ref());

    // show command being run
    tracing::debug!("{}", crate::util::cmd_show(&cmd));
    cmd
}

fn parse_cmd<P: AsRef<Path>>(
    mut cmd: Command,
    tla_file_base_name: &P,
    output_file_base_name: &P,
) -> Command {
    cmd.arg("parse")
        .arg(format!(
            "--output={}",
            output_file_base_name.as_ref().to_string_lossy()
        ))
        .arg(tla_file_base_name.as_ref());

    // show command being run
    tracing::debug!("{}", crate::util::cmd_show(&cmd));
    cmd
}

/// Creates an Apalache start command providing `temp_dir` as a library directory and the Apalache jar
fn apalache_start_cmd(temp_dir: &tempfile::TempDir, runtime: &ModelatorRuntime) -> Command {
    let apalache = jar::Jar::Apalache.path(&runtime.dir);

    let mut cmd = Command::new("java");

    cmd.current_dir(temp_dir)
        .arg(format!(
            "-DTLA-Library={}",
            temp_dir.path().to_string_lossy()
        ))
        .arg("-jar")
        .arg(format!(
            "-Djava.io.tmpdir={}",
            temp_dir.path().to_string_lossy()
        ))
        .arg(format!("{}", apalache.as_path().to_string_lossy()));

    cmd
}
