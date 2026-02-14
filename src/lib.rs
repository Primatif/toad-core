pub mod config;
pub mod error;
pub mod models;
pub mod registry;
pub mod reports;
pub mod strategy;
pub mod ui;
pub mod utils;
pub mod workflow;
pub mod workspace;

// Re-export everything for backward compatibility and convenience
pub use config::{ContextBudget, ContextType, GlobalConfig, ProjectContext};
pub use error::{ToadError, ToadResult};
pub use models::{
    ActivityTier, ChangelogHistory, ChangeType, EcosystemChangelog, ProjectAtlas, ProjectChange,
    ProjectDetail, ProjectDna, StackStrategy, SubmoduleDetail, TargetSource, VcsStatus,
};
pub use registry::{ProjectRegistry, TagRegistry};
pub use reports::{
    AnalyticsReport, BatchCleanReport, BatchOperationReport, BranchGroup, BranchInfo,
    BranchPresence, CleanResult, CommitInfo, GitOpResult, MultiRepoGitReport, MultiRepoStatusItem,
    MultiRepoStatusReport, OperationResult, PrStatus, PreflightResult, ProjectAnalytics,
    ProjectStatus, RepoStatus, SearchResult, StatusReport,
};
pub use ui::{NoOpReporter, ProgressReporter};
pub use workflow::{CustomWorkflow, WorkflowRegistry};
pub use workspace::{HIGH_VALUE_FILES, Workspace};

#[cfg(test)]
mod tests;
