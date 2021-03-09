use serde::{Deserialize, Serialize};
use std::fmt;

pub mod trace;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ArtifactManifest {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub typ: &'static str,
}

enum Type {
    TLA,
}

pub trait Artifact: fmt::Display {
    //fn name() -> &'static str;
}

impl fmt::Debug for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
