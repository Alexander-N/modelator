mod history;
mod graph;

use crate::artifact::TlaConfigFile;
use std::collections::HashSet;
use crate::Error;

pub(crate) fn generate_explorer_module(tla_module_name: &str, vars: &HashSet<String>) -> String {
    format!(
        r#"
---------- MODULE Explore ----------

EXTENDS {}, Sequences

VARIABLE history

{}

History0 == <<
    HistoryEntry("undefined", "undefined")
>>

Explore ==
    /\ history \in {{History0}}

InitExplore ==
    /\ Init
    /\ history = History0

NextExplore ==
    /\ Next
    /\ history' = Append(history, {})

====================================
"#,
        tla_module_name,
        history_entry_tla_definition(&vars),
        history_entry_tla_definition_call(&vars)
    )
}

pub(crate) fn generate_explorer_config(
    tla_config_file: &TlaConfigFile,
    invariant: &str,
) -> Result<String, Error> {
    // TODO: write a config parser: assume that only constant(s) and init/next are allowed and throw error otherwise
    let tla_config = std::fs::read_to_string(tla_config_file.path()).map_err(Error::io)?;
    Ok(format!(
        r#"
{}
INVARIANT {}
"#,
        tla_config, invariant
    ))
}

fn history_entry_tla_definition_call(vars: &HashSet<String>) -> String {
    let args = vars.iter().cloned().collect::<Vec<_>>().join(", ");
    format!("HistoryEntry({})", args)
}

fn history_entry_tla_definition(vars: &HashSet<String>) -> String {
    let args = vars
        .iter()
        .map(|var| format!("{}_value", var))
        .collect::<Vec<_>>()
        .join(", ");
    let history_vars = vars
        .iter()
        .map(|var| format!("prev_{} |-> {}_value", var, var))
        .collect::<Vec<_>>()
        .join(",\n\t\t");
    format!(
        r#"
HistoryEntry({}) ==
    [
        {}
    ]
"#,
        args, history_vars
    )
}
