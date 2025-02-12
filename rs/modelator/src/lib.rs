//! `modelator` is a framework for model-based testing.
#![warn(
    unreachable_pub,
    missing_docs,
    missing_copy_implementations,
    trivial_numeric_casts,
    unused_extern_crates,
    rust_2018_idioms
)]
// It makes sense to allow those when the development is active
#![allow(unused_imports, dead_code)]

/// Modelator's error type.
mod error;

/// List of artifacts.
pub mod artifact;

/// Model checkers and languages.
pub mod model;

/// Caching of model-checker outputs.
mod cache;

/// Jar utilities.
mod jar;

/// Command-line interface.
pub mod cli;

/// Datastructure converter.
/// Allows to define conversion rules to make (cook)
/// concrete data-structures from the abstract ones for testing purposes.
pub mod datachef;

/// Utilitary functions.
mod util;

/// Provides the way to run sets of test functions on several kinds of test inputs.
pub mod tester;

/// A framework for event-based testing of message-passing systems
/// with possibly partitioned system state.
pub mod event;

/// A runner for steps obtained from Json traces
pub mod step_runner;

/// Testing utilities
pub mod test_util;

use artifact::model_checker_stdout::ModelCheckerStdout;
use artifact::TlaFileSuite;
/// Re-exports.
pub use datachef::Recipe;
pub use error::{Error, TestError};
pub use event::{ActionHandler, Event, EventRunner, EventStream, StateHandler};
use model::checker::{Apalache, ModelChecker, ModelCheckerRuntime, Tlc};
use model::language::Tla;
use serde::de::DeserializeOwned;
pub use step_runner::StepRunner;

use crate::artifact::{Artifact, ArtifactCreator};

use std::collections::BTreeMap;
use std::env;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tempfile::tempdir;

use once_cell::sync::Lazy;
use std::sync::Mutex;

static FILE_SYSTEM_MUTEX: Lazy<Mutex<()>> = Lazy::new(Mutex::default);

/// Wraps the data from running test(s), allowing more convenient access to the results.
pub struct TestReport {
    test_name_to_trace_execution_result: BTreeMap<String, Vec<Result<(), TestError>>>,
}

impl TestReport {
    /// Returns true iff no test failed
    pub fn no_test_failed(&self) -> bool {
        !self
            .test_name_to_trace_execution_result
            .values()
            .flatten()
            .any(Result::is_err)
    }

    /// Get the vector of results from running counterexample(s) for a single test
    pub fn result_of_test(&self, name: &str) -> Option<&Vec<Result<(), TestError>>> {
        self.test_name_to_trace_execution_result.get(name)
    }

    /// Returns the vector containing the results for each test
    pub fn all(
        &self,
    ) -> std::collections::btree_map::Values<
        '_,
        std::string::String,
        Vec<Result<(), error::TestError>>,
    > {
        self.test_name_to_trace_execution_result.values()
    }

    /// Returns the concatenation of the vectors containing the results for each test
    pub fn flat(&self) -> Vec<&Result<(), TestError>> {
        self.all().flatten().collect()
    }
}

/// Set of options to configure `modelator` runtime.
#[derive(Clone, Debug)]
pub struct ModelatorRuntime {
    /// Model checker runtime.
    pub model_checker_runtime: ModelCheckerRuntime,

    /// Modelator directory.
    pub dir: PathBuf,
}

impl Default for ModelatorRuntime {
    fn default() -> Self {
        Self {
            model_checker_runtime: ModelCheckerRuntime::default(),
            dir: directories::ProjectDirs::from("systems", "Informal", "modelator")
                .expect("there is no valid home directory")
                .data_dir()
                .into(), // env::home_dir().unwrap().join(".modelator"), //Path::new(".modelator").to_path_buf(),
        }
    }
}

impl ModelatorRuntime {
    /// Set TLC runtime.
    pub fn model_checker_runtime(mut self, model_checker_runtime: ModelCheckerRuntime) -> Self {
        self.model_checker_runtime = model_checker_runtime;
        self
    }

    /// Set modelator directory.
    pub fn dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.dir = dir.as_ref().to_path_buf();
        self
    }

    pub(crate) fn setup(&self) -> Result<(), Error> {
        // init tracing subscriber (in case it's not already)
        if let Err(e) = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init()
        {
            tracing::trace!(
                "modelator attempted to init the tracing_subscriber: {:?}",
                e
            );
        }

        self.ensure_dependencies_exist_on_filesystem()?;

        Ok(())
    }

    fn ensure_dependencies_exist_on_filesystem(&self) -> Result<(), Error> {
        let _guard = FILE_SYSTEM_MUTEX.lock();

        // create modelator dir if it doesn't already exist
        if !self.dir.as_path().is_dir() {
            std::fs::create_dir_all(&self.dir)?;
        }

        // download missing jars
        jar::download_jars_if_necessary(&self.dir)?;
        tracing::trace!("modelator setup completed");

        Ok(())
    }

    /// Given a [`crate::artifact::TlaFile`] with TLA+ test assertions,
    /// as well as a [`crate::artifact::TlaConfigFile`] with TLA+ configuration,
    /// generate all traces resulting from the test assertions.
    ///
    /// The traces are generated by executing a model checker,
    /// which can be selected via [`ModelatorRuntime`].
    ///
    /// # Examples
    ///
    /// ```
    /// let tla_tests_file_path = "tests/integration/resource/NumbersAMaxBMinTest.tla";
    /// let tla_config_file_path = "tests/integration/resource/Numbers.cfg";
    /// let runtime = modelator::ModelatorRuntime::default();
    /// let trace_results = runtime.traces(tla_tests_file_path, tla_config_file_path).unwrap();
    /// println!("{:?}", trace_results);
    /// ```
    pub fn traces<P: AsRef<Path>>(
        &self,
        tla_tests_file_path: P,
        tla_config_file_path: P,
    ) -> Result<BTreeMap<String, Result<Vec<artifact::JsonTrace>, Error>>, Error> {
        // Each test maps to a result containing the vec of all it's traces.

        let mut res: BTreeMap<String, Result<Vec<artifact::JsonTrace>, Error>> = BTreeMap::new();

        // setup modelator
        self.setup()?;

        let file_suite =
            TlaFileSuite::from_tla_and_config_paths(tla_tests_file_path, tla_config_file_path)?;

        let tests = Tla::generate_tests(&file_suite)?;

        #[allow(clippy::needless_collect)]
        // rust iterators are lazy
        // so we need to collect the traces in memory before deleting the work directory
        let trace_results = (&tests)
            .into_par_iter()
            .map(|test| match self.model_checker_runtime.model_checker {
                ModelChecker::Tlc => Tlc::test(&test.file_suite, self),
                ModelChecker::Apalache => Apalache::test(&test.file_suite, self),
            })
            .collect::<Vec<_>>();

        for (i, trace_result) in trace_results.into_iter().enumerate() {
            let test_name = tests[i].name.clone();
            let (traces, _) = trace_result?;
            let jsons: Result<Vec<artifact::JsonTrace>, Error> = traces
                .into_iter()
                .map(Tla::tla_trace_to_json_trace)
                .collect();
            res.insert(test_name, jsons);
        }

        Ok(res)
    }

    /// This is the most simple interface to run your system under test (SUT)
    /// against traces obtained from TLA+ tests.
    /// The function generates TLA+ traces using [`ModelatorRuntime::traces`] and execute them against
    /// the SUT that implements [`StepRunner`].
    ///
    /// For more information, please consult the documentation of [`ModelatorRuntime::traces`] and
    /// [`StepRunner`].
    ///
    /// # Example
    ///
    // #[allow(clippy::needless_doctest_main)]
    /// ```
    /// use modelator::StepRunner;
    /// use serde::Deserialize;
    ///
    /// // Suppose your system under test (SUT) consists of two integer variables,
    /// // where each number can be increased independently;
    /// // SUT also maintains the sum and product of the numbers.
    /// use modelator::test_util::NumberSystem;
    ///
    /// // We define a structure that is capable to serialize the states of a TLA+ trace.
    /// #[derive(Debug, Clone, Deserialize)]
    /// #[serde(rename_all = "camelCase")]
    /// struct NumbersStep {
    ///     a: u64,
    ///     b: u64,
    ///     action: Action,
    ///     action_outcome: String
    /// }
    ///
    /// // We also define the abstract actions: do nothing / increase a / increase b.
    /// #[derive(Debug, Clone, Deserialize)]
    /// enum Action {
    ///     None,
    ///     IncreaseA,
    ///     IncreaseB
    /// }
    ///
    /// // We implement `StepRunner` for our SUT
    /// // This implementation needs to define only a couple of functions:
    /// impl StepRunner<NumbersStep> for NumberSystem {
    ///     // how to handle the initial step (initialize your system)
    ///     fn initial_step(&mut self, step: NumbersStep) -> Result<(), String> {
    ///         self.a = step.a;
    ///         self.b = step.b;
    ///         self.recalculate();
    ///         Ok(())
    ///     }
    ///
    ///     // how to handle all subsequent steps
    ///     fn next_step(&mut self, step: NumbersStep) -> Result<(), String> {
    ///         // Execute the action, and check the outcome
    ///         let res = match step.action {
    ///             Action::None => Ok(()),
    ///             Action::IncreaseA => self.increase_a(1),
    ///             Action::IncreaseB => self.increase_b(2),
    ///         };
    ///         let outcome = match res {
    ///             Ok(()) => "OK".to_string(),
    ///             Err(s) => s,
    ///         };
    ///         assert_eq!(outcome, step.action_outcome);
    ///
    ///         // Check that the system state matches the state of the model
    ///         assert_eq!(self.a, step.a);
    ///         assert_eq!(self.b, step.b);
    ///
    ///         Ok(())
    ///     }
    /// }
    ///
    /// // To run your system against a TLA+ test, just point to the corresponding TLA+ files.
    /// fn test() {
    ///     let tla_tests_file_path = "tests/integration/resource/NumbersAMaxBMinTest.tla";
    ///     let tla_config_file_path = "tests/integration/resource/Numbers.cfg";
    ///     let runtime = modelator::ModelatorRuntime::default();
    ///     let mut system = NumberSystem::default();
    ///     assert!(runtime.run_tla_steps(tla_tests_file_path, tla_config_file_path, &mut system).is_ok());
    /// }
    /// ```
    pub fn run_tla_steps<P, System, Step>(
        &self,
        tla_tests_file_path: P,
        tla_config_file_path: P,
        system: &mut System,
    ) -> Result<TestReport, Error>
    where
        P: AsRef<Path>,
        System: StepRunner<Step> + Debug + Clone,
        Step: DeserializeOwned + Debug + Clone,
    {
        Ok(TestReport {
            test_name_to_trace_execution_result: {
                let mut ret = BTreeMap::new();

                let traces_for_tests = self.traces(tla_tests_file_path, tla_config_file_path)?;

                for (test_name, traces) in traces_for_tests {
                    let traces = traces?;
                    let results: Vec<Result<(), TestError>> =
                        traces.into_iter().map(|it| system.run(it)).collect();
                    ret.insert(test_name, results);
                }
                ret
            },
        })
    }

    /// Run the system under test (SUT) using the abstract events obtained
    /// from TLA+ traces. Traces are generated using [`ModelatorRuntime::traces`],
    /// To interpret abstract events an [`EventRunner`] needs to be created,
    /// as well as [`StateHandler`] and [`ActionHandler`] to be implemented
    /// for abstract states and actions you want to handle.
    ///
    /// # Example
    ///
    /// ```
    /// use modelator::{EventRunner, ActionHandler, StateHandler};
    /// use serde::Deserialize;
    ///
    /// // Suppose your system under test (SUT) consists of two integer variables,
    /// // where each number can be increased independently;
    /// // SUT also maintains the sum and product of the numbers.
    /// use modelator::test_util::NumberSystem;
    ///
    /// // In order to drive your SUT, we could define two abstract states,
    /// // that contain the state of the variables `a` and `b`.
    /// #[derive(Debug, Clone, Deserialize, PartialEq)]
    /// struct A {
    ///     a: u64,
    /// }
    /// #[derive(Debug, Clone, Deserialize, PartialEq)]
    /// struct B {
    ///     b: u64,
    /// }
    ///
    /// // We also define the abstract actions: do nothing / increase a / increase b.
    /// #[derive(Debug, Clone, Deserialize)]
    /// enum Action {
    ///     None,
    ///     IncreaseA,
    ///     IncreaseB
    /// }
    ///
    /// // We define StateHandlers that are able to initialize your SUT from
    /// // these abstract states, as well as to read them at any point in time.
    /// impl StateHandler<A> for NumberSystem {
    ///     fn init(&mut self, state: A) {
    ///         self.a = state.a
    ///     }
    ///     fn read(&self) -> A {
    ///         A { a: self.a }
    ///     }
    /// }
    /// impl StateHandler<B> for NumberSystem {
    ///     fn init(&mut self, state: B) {
    ///         self.b = state.b
    ///     }
    ///     fn read(&self) -> B {
    ///         B { b: self.b }
    ///     }
    /// }
    ///
    /// // We define also an action handler that processes abstract actions
    /// impl ActionHandler<Action> for NumberSystem {
    ///     type Outcome = String;
    ///
    ///     fn handle(&mut self, action: Action) -> Self::Outcome {
    ///         let result_to_outcome = |res| match res {
    ///             Ok(()) => "OK".to_string(),
    ///             Err(s) => s
    ///         };
    ///         match action {
    ///             Action::None => "OK".to_string(),
    ///             Action::IncreaseA => result_to_outcome(self.increase_a(1)),
    ///             Action::IncreaseB => result_to_outcome(self.increase_b(2))
    ///         }
    ///     }
    /// }
    ///
    /// // To run your system against a TLA+ test, just point to the corresponding TLA+ files.
    /// fn main() {
    ///     let tla_tests_file_path = "tests/integration/resource/NumbersAMaxBMaxTest.tla";
    ///     let tla_config_file_path = "tests/integration/resource/Numbers.cfg";
    ///     let runtime = modelator::ModelatorRuntime::default();
    ///     
    ///     // We create a system under test
    ///     let mut system = NumberSystem::default();
    ///
    ///     // We construct a runner, and tell which which states and actions it should process.
    ///     let mut runner = EventRunner::new()
    ///         .with_state::<A>()
    ///         .with_state::<B>()
    ///         .with_action::<Action>();
    ///
    ///     // run your system against the events produced from TLA+ tests.
    ///     let result = runtime.run_tla_events(tla_tests_file_path, tla_config_file_path, &mut system, &mut runner);
    ///     // At each step of a test, the state of your system is being checked
    ///     // against the state that the TLA+ model expects
    ///     assert!(result.is_ok());
    ///     // You can also check the final state of your system, if you want.
    ///     assert_eq!(system.a, 6);
    ///     assert_eq!(system.b, 6);
    ///     assert_eq!(system.sum, 12);
    ///     assert_eq!(system.prod, 36);
    /// }
    /// ```
    // #[allow(clippy::needless_doctest_main)]
    #[allow(clippy::needless_doctest_main)]
    pub fn run_tla_events<P, System>(
        &self,
        tla_tests_file_path: P,
        tla_config_file_path: P,
        system: &mut System,
        runner: &mut event::EventRunner<System>,
    ) -> Result<TestReport, Error>
    where
        P: AsRef<Path>,
        System: Debug + Default,
    {
        Ok(TestReport {
            test_name_to_trace_execution_result: {
                let mut ret = BTreeMap::new();

                let traces_for_tests = self.traces(tla_tests_file_path, tla_config_file_path)?;

                for (test_name, traces) in traces_for_tests {
                    let traces = traces?;
                    let results: Vec<Result<(), TestError>> = traces
                        .iter()
                        .map(|trace| {
                            let events: EventStream = trace.clone().into();
                            runner
                                .run(system, &mut events.into_iter())
                                .map_err(|op| match op {
                                    TestError::UnhandledTest { system, .. } => {
                                        TestError::UnhandledTest {
                                            test: trace.to_string(),
                                            system,
                                        }
                                    }
                                    TestError::FailedTest {
                                        message,
                                        location,
                                        system,
                                        ..
                                    } => TestError::FailedTest {
                                        test: trace.to_string(),
                                        message,
                                        location,
                                        system,
                                    },
                                    TestError::Modelator(_) => op,
                                })
                        })
                        .collect();
                    ret.insert(test_name, results);
                }
                ret
            },
        })
    }
}
