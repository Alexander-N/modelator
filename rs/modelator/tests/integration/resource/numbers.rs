use crate::common::StepRunnerArgs;
use crate::error::IntegrationTestError;

use modelator::test_util::NumberSystem;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NumbersStep {
    a: u64,
    b: u64,
    action: Action,
    action_outcome: String,
}

// We also define the abstract actions: do nothing / increase a / increase b.
#[derive(Debug, Clone, Deserialize)]
enum Action {
    None,
    IncreaseA,
    IncreaseB,
}

impl modelator::step_runner::StepRunner<NumbersStep> for NumberSystem {
    fn initial_step(&mut self, step: NumbersStep) -> Result<(), String> {
        self.a = step.a;
        self.b = step.b;
        self.recalculate();
        Ok(())
    }

    // how to handle all subsequent steps
    fn next_step(&mut self, step: NumbersStep) -> Result<(), String> {
        // Execute the action, and check the outcome
        let res = match step.action {
            Action::None => Ok(()),
            Action::IncreaseA => self.increase_a(1),
            Action::IncreaseB => self.increase_b(2),
        };
        let outcome = match res {
            Ok(()) => "OK".to_string(),
            Err(s) => s,
        };
        assert_eq!(outcome, step.action_outcome);

        // Check that the system state matches the state of the model
        assert_eq!(self.a, step.a);
        assert_eq!(self.b, step.b);

        Ok(())
    }
}

pub fn test(args: StepRunnerArgs) -> Result<(), IntegrationTestError> {
    match args.test_function_name.as_str() {
        "default" => {
            let mut system = NumberSystem::default();

            args.modelator_runtime.run_tla_steps(
                args.tla_tests_filepath,
                args.tla_config_filepath,
                &mut system,
            )?;

            let expect: NumberSystem = serde_json::value::from_value(args.expect)?;

            assert_eq!(system, expect);
            Ok(())
        }
        _ => Err(IntegrationTestError::FaultyTest(
            "Wrong test_function name given as argument to numbers.rs test".into(),
        )),
    }
}
