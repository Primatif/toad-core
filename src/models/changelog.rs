use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use super::{ActivityTier, VcsStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectChange {
    pub name: String,
    pub change_type: ChangeType,
    pub old_vcs: Option<VcsStatus>,
    pub new_vcs: Option<VcsStatus>,
    pub old_activity: Option<ActivityTier>,
    pub new_activity: Option<ActivityTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemChangelog {
    pub timestamp: SystemTime,
    pub changes: Vec<ProjectChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangelogHistory {
    pub entries: Vec<EcosystemChangelog>,
}
