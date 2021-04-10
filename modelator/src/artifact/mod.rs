mod json_trace;
mod tla_cfg_file;
mod tla_file;
mod tla_next_states;
mod tla_trace;
mod tla_variables;

pub(crate) type TlaState = String;
pub(crate) use tla_next_states::TlaAndJsonState;

// Re-exports.
pub use json_trace::JsonTrace;
pub use tla_cfg_file::TlaConfigFile;
pub use tla_file::TlaFile;
pub use tla_next_states::TlaNextStates;
pub use tla_trace::TlaTrace;
pub use tla_variables::TlaVariables;
