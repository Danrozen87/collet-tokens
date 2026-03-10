//! Token input parser — YAML and JSON auto-detection.
//!
//! Accepts a string containing either YAML or JSON and parses it into a
//! [`TokenInput`]. Format is auto-detected: if the input starts with `{`
//! (ignoring whitespace), it is treated as JSON; otherwise YAML.
//! Missing sections receive sensible defaults via `serde(default)`.

use crate::issue::Issue;
use crate::types::TokenInput;

/// Detected input format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    /// JSON object.
    Json,
    /// YAML document.
    Yaml,
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Yaml => write!(f, "YAML"),
        }
    }
}

/// Auto-detect the input format of a token string.
///
/// Returns [`InputFormat::Json`] if the trimmed input starts with `{`,
/// otherwise [`InputFormat::Yaml`].
#[must_use]
pub fn detect_format(input: &str) -> InputFormat {
    if input.trim_start().starts_with('{') {
        InputFormat::Json
    } else {
        InputFormat::Yaml
    }
}

/// Parse a YAML or JSON string into a [`TokenInput`].
///
/// Format is auto-detected via [`detect_format`]. Missing sections receive
/// default values (see [`TokenInput::default`]).
///
/// # Errors
///
/// Returns a vec containing a single parse-error [`Issue`] if the input
/// is malformed or contains fields that cannot be deserialized into the
/// expected types.
pub fn parse(input: &str) -> Result<TokenInput, Vec<Issue>> {
    let format = detect_format(input);
    match format {
        InputFormat::Json => serde_json::from_str::<TokenInput>(input).map_err(|e| {
            vec![Issue::error(
                "parse-error",
                "root",
                format!("Failed to parse JSON token file: {e}"),
            )]
        }),
        InputFormat::Yaml => serde_yaml::from_str::<TokenInput>(input).map_err(|e| {
            vec![Issue::error(
                "parse-error",
                "root",
                format!("Failed to parse YAML token file: {e}"),
            )]
        }),
    }
}

/// Parse a JSON string into a [`TokenInput`].
///
/// # Errors
///
/// Returns a vec containing a single parse-error [`Issue`] if the JSON
/// is malformed.
pub fn parse_json(input: &str) -> Result<TokenInput, Vec<Issue>> {
    serde_json::from_str::<TokenInput>(input).map_err(|e| {
        vec![Issue::error(
            "parse-error",
            "root",
            format!("Failed to parse JSON token file: {e}"),
        )]
    })
}

/// Parse a YAML string into a [`TokenInput`].
///
/// # Errors
///
/// Returns a vec containing a single parse-error [`Issue`] if the YAML
/// is malformed.
pub fn parse_yaml(input: &str) -> Result<TokenInput, Vec<Issue>> {
    serde_yaml::from_str::<TokenInput>(input).map_err(|e| {
        vec![Issue::error(
            "parse-error",
            "root",
            format!("Failed to parse YAML token file: {e}"),
        )]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_json() {
        assert_eq!(detect_format(r#"{ "fonts": {} }"#), InputFormat::Json);
        assert_eq!(detect_format("  {  }"), InputFormat::Json);
    }

    #[test]
    fn detect_yaml() {
        assert_eq!(detect_format("fonts:\n  display: Inter"), InputFormat::Yaml);
        assert_eq!(detect_format("---\nfonts:"), InputFormat::Yaml);
    }

    #[test]
    fn parse_minimal_yaml() {
        let yaml = r#"
fonts:
  display: "Inter, sans-serif"
  body: "Inter, sans-serif"
  mono: "monospace"
"#;
        let result = parse(yaml);
        assert!(result.is_ok());
        let input = result.expect("parse should succeed");
        assert_eq!(input.fonts.display, "Inter, sans-serif");
    }

    #[test]
    fn parse_empty_json_uses_defaults() {
        let result = parse("{}");
        assert!(result.is_ok());
        let input = result.expect("parse should succeed");
        assert_eq!(input.fonts.display, "system-ui, sans-serif");
        assert!(input.colors.is_empty());
        assert!((input.typography.base_size - 16.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_empty_yaml_uses_defaults() {
        let result = parse("---\n{}");
        assert!(result.is_ok());
        let input = result.expect("parse should succeed");
        assert_eq!(input.fonts.body, "system-ui, sans-serif");
    }

    #[test]
    fn parse_yaml_with_fonts() {
        let yaml = r#"
fonts:
  display: "Inter, sans-serif"
  body: "Source Sans, sans-serif"
  mono: "Fira Code, monospace"
"#;
        let input = parse(yaml).expect("parse YAML with fonts");
        assert_eq!(input.fonts.display, "Inter, sans-serif");
        assert_eq!(input.fonts.body, "Source Sans, sans-serif");
        assert_eq!(input.fonts.mono, "Fira Code, monospace");
    }

    #[test]
    fn parse_json_with_colors() {
        let json = r#"{
            "colors": {
                "primary": {
                    "light": { "l": 0.55, "c": 0.25, "h": 264 },
                    "dark": { "l": 0.75, "c": 0.18, "h": 264 }
                }
            }
        }"#;
        let input = parse(json).expect("parse JSON with colors");
        assert_eq!(input.colors.len(), 1);
        let primary = &input.colors["primary"];
        assert!((primary.light.l - 0.55).abs() < f64::EPSILON);
        assert!((primary.dark.h - 264.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result = parse("{ invalid }");
        assert!(result.is_err());
        let issues = result.expect_err("should fail");
        assert!(issues[0].is_error());
        assert_eq!(issues[0].code, "parse-error");
    }

    #[test]
    fn parse_invalid_yaml_returns_error() {
        let result = parse("{{invalid");
        assert!(result.is_err());
        let issues = result.expect_err("should fail");
        assert!(issues[0].is_error());
        assert_eq!(issues[0].code, "parse-error");
    }

    #[test]
    fn parse_json_explicit() {
        let result = parse_json(r#"{"fonts": {"display": "Arial, sans-serif"}}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_yaml_explicit() {
        let result = parse_yaml("fonts:\n  display: Arial, sans-serif");
        assert!(result.is_ok());
    }
}
