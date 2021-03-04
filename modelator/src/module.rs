use serde::{Deserialize};
use crate::{Artifact, ArtifactManifest};

#[derive(Debug, Deserialize)]
pub struct ModuleManifest {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub methods: Vec<MethodManifest>
}

// For static manifests of modules in Rust
impl From<&'static str> for ModuleManifest {
    fn from(json: &'static str) -> ModuleManifest {
        serde_json::from_str(json).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct MethodManifest {
    pub name: &'static str,
    pub description: &'static str,
    pub inputs: Vec<ArtifactManifest>,    
    pub results: Vec<ArtifactManifest>,    
    pub errors: Vec<ArtifactManifest>,    
}



pub trait Module {
    fn manifest() -> ModuleManifest;
    fn run(method: &str, inputs: Vec<Box<dyn Artifact>>) -> Result<Vec<Box<dyn Artifact>>, Vec<Box<dyn Artifact>>>;

}

