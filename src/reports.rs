use crate::config::ContextType;
use crate::models::{ActivityTier, ProjectDetail, VcsStatus};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// --- Git Data Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub author: String,
    pub message: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub current_branch: String,
    pub head_sha: String,
    pub vcs_status: VcsStatus,
    pub local_branches: Vec<BranchInfo>,
    pub remote_branches: Vec<BranchInfo>,
    pub unpushed_commits: Vec<CommitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRepoStatusItem {
    pub name: String,
    pub status: VcsStatus,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRepoStatusReport {
    pub items: Vec<MultiRepoStatusItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiRepoGitReport {
    pub title: String,
    pub results: Vec<GitOpResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOpResult {
    pub project_name: String,
    pub command: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub project_name: String,
    pub is_clean: bool,
    pub is_aligned: bool,
    pub unpushed_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrStatus {
    Open,
    Merged,
    Closed,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPresence {
    pub project_name: String,
    pub exists_locally: bool,
    pub exists_remotely: bool,
    pub pr_status: PrStatus,
    pub pr_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchGroup {
    pub name: String,
    pub projects: Vec<BranchPresence>,
}

// --- CLI Command Result Types (No-Print Mandate) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatus {
    pub name: String,
    pub stack: String,
    pub activity: ActivityTier,
    pub vcs_status: VcsStatus,
    pub is_aligned: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub active_context: Option<String>,
    pub context_type: ContextType,
    pub projects: Vec<ProjectStatus>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalytics {
    pub name: String,
    pub total_size: u64,
    pub artifact_size: u64,
    pub bloat_percentage: f64,
    pub activity: ActivityTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub total_usage: u64,
    pub total_artifacts: u64,
    pub offenders: Vec<ProjectAnalytics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub matches: Vec<ProjectDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub project_name: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationReport {
    pub command: String,
    pub results: Vec<OperationResult>,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
}
