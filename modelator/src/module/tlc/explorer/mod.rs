mod gen;
mod graph;

// Re-exports.
pub(crate) use gen::ExplorerInvariant;
pub(crate) use graph::NextStates;

use crate::artifact::{TlaConfigFile, TlaFile, TlaTrace, TlaVariables};
use crate::module::Tlc;
use crate::{Error, Options};

pub(crate) fn explore(
    tla_file: &TlaFile,
    tla_config_file: &TlaConfigFile,
    tla_variables: &TlaVariables,
    start_state: &String,
    known_next_states: Option<&Vec<String>>,
    explorer_invariant: ExplorerInvariant,
    options: &Options,
) -> Result<TlaTrace, Error> {
    let timestamp = crate::util::timestamp();
    // create initial explorer module
    let explorer_tla_file = gen::generate_explorer_module(
        &tla_file,
        tla_variables,
        start_state,
        known_next_states,
        timestamp,
    )?;
    println!("explorer::explore tla file: {:?}", explorer_tla_file);
    // create explorer config
    let explorer_config_file =
        gen::generate_explorer_config(&tla_file, &tla_config_file, explorer_invariant, timestamp)?;
    println!("explorer::explore config file: {:?}", explorer_config_file);

    Tlc::test(explorer_tla_file, explorer_config_file, options)
}
