use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::ProjectDna;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectAtlas {
    pub dna_map: HashMap<String, ProjectDna>,
}
