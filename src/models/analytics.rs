use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub name: String,
    pub version: Option<String>,
    pub depth: usize,
    pub dependents: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub critical_path: Vec<String>,
    pub orphaned_projects: Vec<String>,
    pub circular_dependencies: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityMetrics {
    pub commit_count: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub active_contributors: Vec<String>,
    pub local_deployment_frequency: f64,  // builds per day
    pub local_lead_time_for_changes: f64, // hours from first commit to build
    pub trend: String,                    // e.g. "↗️ Increasing"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtIndicators {
    pub todo_count: usize,
    pub fixme_count: usize,
    pub hack_count: usize,
    pub large_files: Vec<String>,
    pub test_coverage: Option<f32>,
    pub outdated_dependencies: Vec<String>,
    pub churn_complexity_risk: Vec<String>, // files with high churn and high complexity
    pub debt_score: f32,                    // 0-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReadiness {
    pub score: f32, // 0-100
    pub factors: HashMap<String, f32>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub total: u32,
    pub vcs_cleanliness: u32,
    pub test_coverage: u32,
    pub documentation: u32,
    pub activity: u32,
    pub dependencies: u32,
    pub code_quality: u32,
    pub ai_readiness: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInsight {
    pub title: String,
    pub description: String,
    pub severity: String, // "Low", "Medium", "High", "Critical"
    pub action_item: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp: std::time::SystemTime,
    pub health_score: u32,
    pub disk_usage_gb: f64,
    pub commit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendReport {
    pub points: Vec<TrendPoint>,
    pub health_trend: String,
    pub disk_trend: String,
    pub activity_trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetrics {
    pub common_dependencies: Vec<(String, usize)>,
    pub error_handling_consistency: f32,
    pub naming_convention_compliance: f32,
    pub architectural_violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleHealth {
    pub name: String,
    pub alignment_status: String,
    pub commit_drift: i32,
    pub last_sync_hours: f64,
    pub branch_consistency: bool,
}
