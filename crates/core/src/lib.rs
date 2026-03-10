//! Core library for the collet-tokens design token compiler.
//!
//! This crate is the heart of the token compiler — parsing, validation,
//! and resolution. It is a pure library with **no I/O** dependencies.
//! All operations work on strings and structs; filesystem access is left
//! to the CLI and output crates.
//!
//! # Pipeline
//!
//! ```text
//! YAML/JSON &str ─→ TokenInput ─→ validate ─→ ResolvedTokens
//!                    (parse)       (issues)     (resolve)
//! ```
//!
//! # Quick start
//!
//! ```
//! use collet_tokens_core::{compile, ResolvedTokens, Issue};
//!
//! let yaml = r#"
//! fonts:
//!   display: "Inter, sans-serif"
//!   body: "Inter, sans-serif"
//!   mono: "JetBrains Mono, monospace"
//! typography:
//!   base_size: 16
//!   scale_ratio: 1.25
//! "#;
//!
//! match compile(yaml) {
//!     Ok((tokens, warnings)) => {
//!         assert_eq!(tokens.typography.len(), 13);
//!         for w in &warnings {
//!             eprintln!("{w}");
//!         }
//!     }
//!     Err(errors) => {
//!         for e in &errors {
//!             eprintln!("{e}");
//!         }
//!     }
//! }
//! ```

pub mod color;
pub mod issue;
pub mod parser;
pub mod resolver;
pub mod spacing;
pub mod types;
pub mod typography;
pub mod validator;

pub use color::{contrast_ratio, is_in_srgb_gamut, oklch_to_srgb, relative_luminance};
pub use issue::{Issue, Severity};
pub use parser::{InputFormat, detect_format, parse, parse_json, parse_yaml};
pub use resolver::resolve;
pub use types::{
    ColorPair, ContrastLevel, FontConfig, MotionConfig, OklchColor, ResolvedColor, ResolvedFonts,
    ResolvedMotion, ResolvedRadius, ResolvedSpacing, ResolvedTokens, ResolvedTypeRole,
    SpacingConfig, TokenInput, TypographyConfig, ValidationConfig,
};
pub use validator::{has_errors, validate};

/// A file to be written to disk by output generators.
///
/// The core crate produces these descriptors; the CLI or output crates
/// handle the actual filesystem writes.
#[derive(Debug, Clone)]
pub struct OutputFile {
    /// Relative path for the output file (e.g. `"tokens.css"`).
    pub path: String,
    /// File content.
    pub content: String,
}

/// Parse, validate, and resolve a token file in one step.
///
/// Returns the resolved tokens and any non-error issues (warnings, infos)
/// on success. Warnings do not prevent compilation.
///
/// # Errors
///
/// Returns a vector of [`Issue`] if parsing fails or any error-level
/// issues are found during validation.
pub fn compile(input: &str) -> Result<(ResolvedTokens, Vec<Issue>), Vec<Issue>> {
    let token_input = parse(input)?;
    let issues = validate(&token_input);

    if has_errors(&issues) {
        return Err(issues);
    }

    let resolved = resolve(&token_input);
    Ok((resolved, issues))
}

/// Validate a token input string without resolving.
///
/// Useful for lint-only workflows where you want to check for issues
/// without computing the full output.
///
/// # Errors
///
/// Returns a vector of [`Issue`] if parsing fails.
pub fn validate_str(input: &str) -> Result<Vec<Issue>, Vec<Issue>> {
    let token_input = parse(input)?;
    Ok(validate(&token_input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_minimal_yaml() {
        let yaml = r#"
fonts:
  display: "Inter, sans-serif"
  body: "Inter, sans-serif"
  mono: "JetBrains Mono, monospace"
"#;
        let result = compile(yaml);
        assert!(result.is_ok());
        let (tokens, _warnings) = result.expect("compile should succeed");
        assert_eq!(tokens.typography.len(), 13);
        assert!(!tokens.spacing.is_empty());
    }

    #[test]
    fn compile_empty_json() {
        let result = compile("{}");
        assert!(result.is_ok());
    }

    #[test]
    fn compile_invalid_input() {
        let result = compile("{{broken yaml");
        assert!(result.is_err());
    }

    #[test]
    fn compile_with_contrast_error() {
        let yaml = r"
colors:
  text:
    light: { l: 0.6, c: 0, h: 0 }
    dark: { l: 0.6, c: 0, h: 0 }
  surface:
    light: { l: 0.7, c: 0, h: 0 }
    dark: { l: 0.7, c: 0, h: 0 }
";
        let result = compile(yaml);
        assert!(result.is_err());
        let issues = result.expect_err("should fail on contrast");
        assert!(issues.iter().any(|i| i.code == "contrast-fail"));
    }

    #[test]
    fn validate_str_returns_warnings() {
        let yaml = r"
fonts:
  display: CustomFont
  body: system-ui, sans-serif
  mono: monospace
";
        let issues = validate_str(yaml).expect("parse should succeed");
        assert!(issues.iter().any(|i| i.code == "missing-fallback"));
    }

    #[test]
    fn output_file_debug() {
        let f = OutputFile {
            path: "tokens.css".to_owned(),
            content: ":root {}".to_owned(),
        };
        let debug = format!("{f:?}");
        assert!(debug.contains("tokens.css"));
    }
}
