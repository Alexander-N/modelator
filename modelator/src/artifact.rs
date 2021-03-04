use std::fmt;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct ArtifactManifest {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub typ: &'static str    
}

pub trait Artifact: fmt::Display {
    
}

impl fmt::Debug for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}



