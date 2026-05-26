use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Diagnostic severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Critical issue that prevents proper functionality
    Error,
    /// Warning that should be addressed but doesn't break functionality
    Warning,
    /// Informational message
    Info,
}

/// Diagnostic type for tracking metadata parsing and validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    /// Project name where the diagnostic occurred
    pub project_name: String,
    /// Path to the project
    pub project_path: PathBuf,
    /// File that failed to parse (e.g., "Cargo.toml", "package.json")
    pub file_name: String,
    /// Severity of the diagnostic
    pub severity: DiagnosticSeverity,
    /// Human-readable description of the issue
    pub message: String,
    /// Optional detailed error information
    pub details: Option<String>,
}

impl ParseDiagnostic {
    /// Create a new error diagnostic
    pub fn error(
        project_name: String,
        project_path: PathBuf,
        file_name: String,
        message: String,
    ) -> Self {
        Self {
            project_name,
            project_path,
            file_name,
            severity: DiagnosticSeverity::Error,
            message,
            details: None,
        }
    }

    /// Create a new warning diagnostic
    pub fn warning(
        project_name: String,
        project_path: PathBuf,
        file_name: String,
        message: String,
    ) -> Self {
        Self {
            project_name,
            project_path,
            file_name,
            severity: DiagnosticSeverity::Warning,
            message,
            details: None,
        }
    }

    /// Add detailed error information
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

/// Collection of diagnostics for a project scan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticReport {
    /// All diagnostics collected during scanning
    pub diagnostics: Vec<ParseDiagnostic>,
}

impl DiagnosticReport {
    /// Create a new empty diagnostic report
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic to the report
    pub fn add(&mut self, diagnostic: ParseDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Check if there are any error-level diagnostics
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Check if there are any warning-level diagnostics
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning)
    }

    /// Get count of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    /// Get count of warnings
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }

    /// Merge another diagnostic report into this one
    pub fn merge(&mut self, other: DiagnosticReport) {
        self.diagnostics.extend(other.diagnostics);
    }
}
