use crate::artifact::{TlaConfigFile, TlaVariables};
use crate::Error;

pub(crate) fn generate_explorer_module(
    tla_module_name: &str,
    tla_variables: &TlaVariables,
    start_state: &String,
    known_next_states: Option<&Vec<String>>,
) -> String {
    format!(
        r#"
---------- MODULE Explore ----------

EXTENDS {}

VARIABLE nextStates

\* construct a representation of an explored TLA+ state
{}

\* set of known next TLA+ states previously explored
KnownNextStates == {}

\* invariant stating that all `nextStates` must be already known; if we don't
\* have yet all next states, the model checker will give us a new one
Explore ==
    /\ nextStates \subseteq KnownNextStates

InitExplore ==
    \* the TLA+ state from where we should start the exploration
    {}
    \* the set of next states; if the model checker finds a violation of the
    \* `Explore` invariant above where:
    \* - `Len(nextStates) == 0`, then the start state has no next states
    \* - `Len(nextStates) == 1`, then the state in the set is indeed a next
    \* state of the start state
    \* - `Len(nextStates) == 2`, then the model checker has already aplied
    \* `Next` two times, meaning that we have already retrieved all the next
    \* states
    /\ nextStates = {{}}

NextExplore ==
    /\ Next
    /\ nextStates' = nextStates \\union {{{}}}

====================================
"#,
        tla_module_name,
        explored_state_tla_definition(&tla_variables),
        known_next_states_set(known_next_states),
        start_state,
        explored_state_tla_definition_call(&tla_variables)
    )
}

pub(crate) fn generate_explorer_config(tla_config_file: &TlaConfigFile) -> Result<String, Error> {
    // TODO: write a config parser: assume that only constant(s) are allowed and
    //       throw error otherwise
    let tla_config = std::fs::read_to_string(tla_config_file.path()).map_err(Error::io)?;
    Ok(format!(
        r#"
{}
INIT InitExplore
NEXT NextExplore
INVARIANT Explore
"#,
        tla_config
    ))
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
        .join(",\n\t\t");
    format!(
        r#"
ExploredState({}) ==
    [
        {}
    ]
"#,
        args, history_vars
    )
}

fn known_next_states_set(known_next_states: Option<&Vec<String>>) -> String {
    let known_next_states = known_next_states
        .map(|known_next_states| known_next_states.join(",\n\t\t"))
        .unwrap_or_default();
    format!(
        r#"
    {{
        {}
    }}
"#,
        known_next_states
    )
}
