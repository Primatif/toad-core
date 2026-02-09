pub mod config;
pub mod models;
pub mod registry;
pub mod reports;
pub mod strategy;
pub mod workflow;
pub mod workspace;

// Re-export everything for backward compatibility and convenience
pub use config::{ContextType, GlobalConfig, ProjectContext};
pub use models::{
    ActivityTier, ProjectDetail, StackStrategy, SubmoduleDetail, TargetSource, VcsStatus,
};
pub use registry::{ProjectRegistry, TagRegistry};
pub use reports::{
    AnalyticsReport, BatchOperationReport, BranchGroup, BranchInfo, BranchPresence, CommitInfo,
    GitOpResult, MultiRepoGitReport, MultiRepoStatusItem, MultiRepoStatusReport, OperationResult,
    PrStatus, PreflightResult, ProjectAnalytics, ProjectStatus, RepoStatus, SearchResult,
    StatusReport,
};
pub use workflow::{CustomWorkflow, WorkflowRegistry};
pub use workspace::{HIGH_VALUE_FILES, Workspace};

#[cfg(test)]
mod tests;
