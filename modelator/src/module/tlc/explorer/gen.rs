use crate::artifact::{TlaConfigFile, TlaFile, TlaVariables};
use crate::Error;
use std::convert::TryFrom;

pub(crate) enum ExplorerInvariant {
    Explore,
    FindInitialState,
}

impl std::fmt::Display for ExplorerInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Explore => "Explore",
            Self::FindInitialState => "FindInitialState",
        };
        write!(f, "{}", name)
    }
}

pub(crate) fn generate_explorer_module(
    tla_file: &TlaFile,
    tla_variables: &TlaVariables,
    start_state: &String,
    known_next_states: Option<&Vec<String>>,
    timestamp: u128,
) -> Result<TlaFile, Error> {
    let content = format!(
        r#"
---------- MODULE {} ----------

EXTENDS {}

VARIABLE nextStates

\* declaration used to construct a representation of an explored TLA+ state
{}

\* init
InitExplore ==
    \* the TLA+ state from where we should start the exploration
    {}
    \* the set of next states (initially empty)
    /\ nextStates = {{}}

\* next
NextExplore ==
    /\ Next
    \* save the next state
    /\ nextStates' = nextStates \union {{{}}}

\* invariant stating that `nextStates` is always empty; if we set the TLA+ state
\* from which we should start exploration to `Init`, because `nextStates` is set
\* to {{}} in `InitExplore` (i.e. the invariant is false), the model checker
\* will return a counterexample showing us the initial TLA+ state
{} ==
    /\ nextStates /= {{}}

\* set of known next TLA+ states (for a given TLA+ state) previously returned by
\* the model checker
KnownNextStates == {}

\* invariant stating that all `nextStates` must be already known (i.e. a subset
\* of `KnownNextStates`); if we don't have yet all next states, the model
\* checker will give us a new one
\* if the model checker finds a violation of this invariant where:
\* - `Len(nextStates) == 0`, then the start state has no next states
\* - `Len(nextStates) == 1`, then the state in the set is indeed a next
\* state of the start state
\* - `Len(nextStates) == 2`, then the model checker has already applied
\* `Next` two times, meaning that we have already retrieved all the next
\* states
{} ==
    /\ nextStates \subseteq KnownNextStates

====================================
"#,
        explore_module_name(timestamp),
        tla_file.tla_module_name(),
        explored_state_tla_definition(&tla_variables),
        start_state,
        explored_state_tla_definition_call(&tla_variables),
        ExplorerInvariant::FindInitialState,
        known_next_states_set(known_next_states),
        ExplorerInvariant::Explore,
    );
    let path = tla_file
        .dir()
        .join(format!("{}.tla", explore_module_name(timestamp)));
    std::fs::write(&path, content).map_err(Error::io)?;
    TlaFile::try_from(path)
}

pub(crate) fn generate_explorer_config(
    tla_file: &TlaFile,
    tla_config_file: &TlaConfigFile,
    explorer_invariant: ExplorerInvariant,
    timestamp: u128,
) -> Result<TlaConfigFile, Error> {
    // TODO: write a config parser: assume that only constant(s) are allowed and
    //       throw error otherwise
    let tla_config = std::fs::read_to_string(tla_config_file.path()).map_err(Error::io)?;
    let content = format!(
        r#"
{}
INIT InitExplore
NEXT NextExplore
INVARIANT {}
"#,
        tla_config, explorer_invariant
    );
    let path = tla_file
        .dir()
        .join(format!("{}.cfg", explore_module_name(timestamp)));
    std::fs::write(&path, content).map_err(Error::io)?;
    TlaConfigFile::try_from(path)
}

fn explore_module_name(timestamp: u128) -> String {
    format!("Explore_{}", timestamp)
}

fn explored_state_tla_definition_call(tla_variables: &TlaVariables) -> String {
    let args = tla_variables
        .vars()
        .iter()
        .map(|var| {
            // tick var
            format!("{}'", var)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("ExploredState({})", args)
}

fn explored_state_tla_definition(tla_variables: &TlaVariables) -> String {
    let args = tla_variables
        .vars()
        .iter()
        .map(|var| format!("{}_value", var))
        .collect::<Vec<_>>()
        .join(", ");
    let history_vars = tla_variables
        .vars()
        .iter()
        .map(|var| format!("{} |-> {}_value", var, var))
        .collect::<Vec<_>>()
        .join(",\n        ");
    format!(
        r#"ExploredState({}) ==
    [
        {}
    ]"#,
        args, history_vars
    )
}

fn known_next_states_set(known_next_states: Option<&Vec<String>>) -> String {
    let known_next_states = known_next_states
        .map(|known_next_states| known_next_states.join(",\n        "))
        .unwrap_or_default();
    format!(
        r#"
    {{
        {}
    }}"#,
        known_next_states
    )
}
