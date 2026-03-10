//! Diagnostic issue types for token validation.
//!
//! Every validation check produces zero or more [`Issue`] values, each carrying
//! a severity, a machine-readable code, a human-readable message, and an
//! optional fix suggestion.

use std::fmt;

/// Severity level for a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The token set is invalid and cannot be compiled.
    Error,
    /// The token set is technically valid but likely incorrect.
    Warning,
    /// Informational note — no action required.
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}

/// A single diagnostic issue produced during validation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Issue {
    /// How severe this issue is.
    pub severity: Severity,
    /// Machine-readable code (e.g. `"contrast-fail"`, `"missing-fallback"`).
    pub code: &'static str,
    /// Dot-path location within the token file (e.g. `"colors.primary"`).
    pub location: String,
    /// Human-readable description of the problem.
    pub message: String,
    /// Optional suggestion for how to fix the issue.
    pub suggestion: Option<String>,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}): {}",
            self.severity, self.code, self.location, self.message
        )?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " — suggestion: {suggestion}")?;
        }
        Ok(())
    }
}

impl Issue {
    /// Create an error-level issue.
    #[must_use]
    pub fn error(
        code: &'static str,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            location: location.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a warning-level issue.
    #[must_use]
    pub fn warning(
        code: &'static str,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            location: location.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create an info-level issue.
    #[must_use]
    pub fn info(
        code: &'static str,
        location: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Info,
            code,
            location: location.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Attach a fix suggestion to this issue.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Returns `true` if this issue is an error.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Returns `true` if this issue is a warning.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}
