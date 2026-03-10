//! Token input parser — YAML, JSON, and DTCG auto-detection.
//!
//! Accepts a string containing YAML, JSON, or DTCG (W3C Design Tokens Community
//! Group) format and parses it into a [`TokenInput`]. Format is auto-detected:
//! - If the JSON contains `$value` or `$type` keys, it is treated as DTCG
//! - If the input starts with `{` (ignoring whitespace), it is treated as JSON
//! - Otherwise YAML
//!
//! Missing sections receive sensible defaults via `serde(default)`.

use crate::issue::Issue;
use crate::types::{ColorPair, OklchColor, TokenInput};

/// Detected input format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    /// JSON object.
    Json,
    /// YAML document.
    Yaml,
    /// W3C Design Tokens Community Group (DTCG) format JSON.
    Dtcg,
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Yaml => write!(f, "YAML"),
            Self::Dtcg => write!(f, "DTCG"),
        }
    }
}

/// Auto-detect the input format of a token string.
///
/// Returns [`InputFormat::Dtcg`] if the JSON contains `$value` or `$type` keys,
/// [`InputFormat::Json`] if it starts with `{`, otherwise [`InputFormat::Yaml`].
#[must_use]
pub fn detect_format(input: &str) -> InputFormat {
    if input.trim_start().starts_with('{') {
        // Check for DTCG markers in the JSON.
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(input) {
            if contains_dtcg_markers(&value) {
                return InputFormat::Dtcg;
            }
        }
        InputFormat::Json
    } else {
        InputFormat::Yaml
    }
}

/// Recursively check if a JSON value contains DTCG-style `$value` or `$type` keys.
fn contains_dtcg_markers(value: &serde_json::Value) -> bool {
    if let Some(obj) = value.as_object() {
        if obj.contains_key("$value") || obj.contains_key("$type") {
            return true;
        }
        for v in obj.values() {
            if contains_dtcg_markers(v) {
                return true;
            }
        }
    }
    false
}

/// Parse a YAML, JSON, or DTCG string into a [`TokenInput`].
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
        InputFormat::Dtcg => parse_dtcg(input),
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

/// Parse a DTCG (W3C Design Tokens Community Group) JSON string into a [`TokenInput`].
///
/// DTCG format uses `$type` and `$value` keys with optional `$description`.
/// Token groups can inherit `$type` from parent objects. Supported `$type` values:
///
/// - `"color"` — hex color `#rrggbb`, converted to oklch. If only a single value
///   is provided (no light/dark pair), the dark variant is auto-derived by
///   inverting lightness.
/// - `"fontFamily"` — array of font names, joined into a comma-separated stack.
/// - `"dimension"` — CSS dimension value (e.g. `"0.5rem"`, `"4px"`), mapped to
///   spacing or radius entries based on the group name.
///
/// # Errors
///
/// Returns parse-error [`Issue`] values if the JSON is malformed or if
/// a `$value` cannot be interpreted for its declared `$type`.
pub fn parse_dtcg(input: &str) -> Result<TokenInput, Vec<Issue>> {
    let root: serde_json::Value = serde_json::from_str(input).map_err(|e| {
        vec![Issue::error(
            "parse-error",
            "root",
            format!("Failed to parse DTCG JSON: {e}"),
        )]
    })?;

    let root_obj = root.as_object().ok_or_else(|| {
        vec![Issue::error(
            "parse-error",
            "root",
            "DTCG root must be a JSON object".to_owned(),
        )]
    })?;

    let mut token_input = TokenInput::default();
    let mut issues = Vec::new();

    // Walk top-level groups.
    for (group_name, group_value) in root_obj {
        // Skip DTCG metadata keys at root level.
        if group_name.starts_with('$') {
            continue;
        }
        let inherited_type = root_obj
            .get("$type")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        walk_dtcg_group(
            group_name,
            group_value,
            inherited_type.as_deref(),
            &mut token_input,
            &mut issues,
        );
    }

    if issues.iter().any(Issue::is_error) {
        return Err(issues);
    }

    Ok(token_input)
}

/// Recursively walk a DTCG group, collecting tokens into the appropriate
/// [`TokenInput`] fields.
fn walk_dtcg_group(
    path: &str,
    value: &serde_json::Value,
    inherited_type: Option<&str>,
    output: &mut TokenInput,
    issues: &mut Vec<Issue>,
) {
    let Some(obj) = value.as_object() else {
        return;
    };

    // Determine the effective $type: local overrides inherited.
    let local_type = obj.get("$type").and_then(serde_json::Value::as_str);
    let effective_type = local_type.or(inherited_type);

    // If this object has a $value, it is a leaf token.
    if let Some(token_value) = obj.get("$value") {
        process_dtcg_token(path, effective_type, token_value, output, issues);
        return;
    }

    // Otherwise it is a group — recurse into children.
    for (child_name, child_value) in obj {
        if child_name.starts_with('$') {
            continue;
        }
        let child_path = format!("{path}.{child_name}");
        walk_dtcg_group(&child_path, child_value, effective_type, output, issues);
    }
}

/// Process a single DTCG leaf token.
fn process_dtcg_token(
    path: &str,
    effective_type: Option<&str>,
    value: &serde_json::Value,
    output: &mut TokenInput,
    issues: &mut Vec<Issue>,
) {
    match effective_type {
        Some("color") => process_dtcg_color(path, value, output, issues),
        Some("fontFamily") => process_dtcg_font(path, value, output, issues),
        Some("dimension") => process_dtcg_dimension(path, value, output, issues),
        Some(unknown) => {
            issues.push(Issue::info(
                "dtcg-unsupported-type",
                path,
                format!("Unsupported DTCG $type: {unknown} — token skipped"),
            ));
        }
        None => {
            issues.push(Issue::info(
                "dtcg-no-type",
                path,
                "Token has $value but no $type — skipped".to_owned(),
            ));
        }
    }
}

/// Process a DTCG color token (`$type: "color"`).
fn process_dtcg_color(
    path: &str,
    value: &serde_json::Value,
    output: &mut TokenInput,
    issues: &mut Vec<Issue>,
) {
    let token_name = dtcg_path_to_name(path);

    if let Some(hex) = value.as_str() {
        if let Some(oklch) = OklchColor::from_hex(hex) {
            let dark = oklch.invert_lightness();
            output
                .colors
                .insert(token_name, ColorPair { light: oklch, dark });
        } else {
            issues.push(Issue::error(
                "parse-error",
                path,
                format!("Invalid hex color: {hex}"),
            ));
        }
    } else {
        issues.push(Issue::error(
            "parse-error",
            path,
            "Color $value must be a hex string".to_owned(),
        ));
    }
}

/// Process a DTCG font family token (`$type: "fontFamily"`).
fn process_dtcg_font(
    path: &str,
    value: &serde_json::Value,
    output: &mut TokenInput,
    issues: &mut Vec<Issue>,
) {
    let token_name = dtcg_path_to_name(path);

    if let Some(arr) = value.as_array() {
        let stack: Vec<String> = arr
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(String::from)
            .collect();
        let joined = stack.join(", ");
        assign_font_by_name(&token_name, joined, output);
    } else if let Some(s) = value.as_str() {
        assign_font_by_name(&token_name, s.to_owned(), output);
    } else {
        issues.push(Issue::warning(
            "parse-error",
            path,
            "fontFamily $value should be an array or string".to_owned(),
        ));
    }
}

/// Assign a font stack to the appropriate [`FontConfig`] field based on token name.
fn assign_font_by_name(token_name: &str, value: String, output: &mut TokenInput) {
    let lower = token_name.to_lowercase();
    if lower.contains("display") || lower.contains("heading") {
        output.fonts.display = value;
    } else if lower.contains("mono") || lower.contains("code") {
        output.fonts.mono = value;
    } else {
        output.fonts.body = value;
    }
}

/// Process a DTCG dimension token (`$type: "dimension"`).
fn process_dtcg_dimension(
    path: &str,
    value: &serde_json::Value,
    output: &mut TokenInput,
    issues: &mut Vec<Issue>,
) {
    let token_name = dtcg_path_to_name(path);

    let Some(dim_str) = value.as_str() else {
        issues.push(Issue::warning(
            "parse-error",
            path,
            "dimension $value should be a string".to_owned(),
        ));
        return;
    };

    let lower_path = path.to_lowercase();
    if lower_path.contains("radius") || lower_path.contains("corner") {
        output.radius.insert(token_name, dim_str.to_owned());
    } else if lower_path.contains("spacing") || lower_path.contains("space") {
        // Parse dimension to px for spacing scale.
        if let Some(px) = parse_dimension_to_px(dim_str) {
            let base = output.spacing.base;
            if base > 0.0 {
                let multiplier = px / base;
                if !output
                    .spacing
                    .scale
                    .iter()
                    .any(|&s| (s - multiplier).abs() < 0.001)
                {
                    output.spacing.scale.push(multiplier);
                    output
                        .spacing
                        .scale
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                }
            }
        }
    } else {
        // Unknown dimension group — put in radius as a generic dimension.
        output.radius.insert(token_name, dim_str.to_owned());
    }
}

/// Convert a dot-separated DTCG path to a flat token name.
///
/// For example, `"color.surface"` becomes `"surface"`,
/// `"color.text-primary"` becomes `"text-primary"`.
/// Uses the last segment of the path.
#[must_use]
fn dtcg_path_to_name(path: &str) -> String {
    path.rsplit('.').next().unwrap_or(path).to_owned()
}

/// Parse a CSS dimension string to pixels.
///
/// Supports `rem` (× 16) and `px` suffixes. Returns `None` for unparseable values.
fn parse_dimension_to_px(dim: &str) -> Option<f64> {
    let dim = dim.trim();
    if let Some(rem_str) = dim.strip_suffix("rem") {
        rem_str.trim().parse::<f64>().ok().map(|v| v * 16.0)
    } else if let Some(px_str) = dim.strip_suffix("px") {
        px_str.trim().parse::<f64>().ok()
    } else {
        dim.parse::<f64>().ok()
    }
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

    // -----------------------------------------------------------------------
    // DTCG format tests
    // -----------------------------------------------------------------------

    #[test]
    fn detect_dtcg_format() {
        let dtcg = r##"{ "color": { "surface": { "$type": "color", "$value": "#ffffff" } } }"##;
        assert_eq!(detect_format(dtcg), InputFormat::Dtcg);
    }

    #[test]
    fn detect_dtcg_with_nested_type() {
        let dtcg = r#"{ "spacing": { "$type": "dimension", "sm": { "$value": "0.5rem" } } }"#;
        assert_eq!(detect_format(dtcg), InputFormat::Dtcg);
    }

    #[test]
    fn parse_dtcg_colors() {
        let dtcg = r##"{
            "color": {
                "surface": {
                    "$type": "color",
                    "$value": "#ffffff",
                    "$description": "Main background"
                },
                "text-primary": {
                    "$type": "color",
                    "$value": "#1c1917"
                }
            }
        }"##;
        let result = parse_dtcg(dtcg);
        assert!(result.is_ok(), "parse_dtcg failed: {result:?}");
        let input = result.expect("should parse");
        assert_eq!(input.colors.len(), 2);
        assert!(input.colors.contains_key("surface"));
        assert!(input.colors.contains_key("text-primary"));

        // White should have high lightness.
        let surface = &input.colors["surface"];
        assert!(
            surface.light.l > 0.95,
            "white lightness should be >0.95, got {}",
            surface.light.l
        );

        // Dark variant should be auto-derived (surface → dark surface).
        assert!(
            surface.dark.l < 0.2,
            "dark surface lightness should be <0.2, got {}",
            surface.dark.l
        );
    }

    #[test]
    fn parse_dtcg_font_family() {
        let dtcg = r#"{
            "font": {
                "body": {
                    "$type": "fontFamily",
                    "$value": ["Inter", "system-ui", "sans-serif"]
                },
                "display": {
                    "$type": "fontFamily",
                    "$value": ["Plus Jakarta Sans", "system-ui", "sans-serif"]
                },
                "mono": {
                    "$type": "fontFamily",
                    "$value": ["JetBrains Mono", "monospace"]
                }
            }
        }"#;
        let input = parse_dtcg(dtcg).expect("should parse");
        assert_eq!(input.fonts.body, "Inter, system-ui, sans-serif");
        assert_eq!(
            input.fonts.display,
            "Plus Jakarta Sans, system-ui, sans-serif"
        );
        assert_eq!(input.fonts.mono, "JetBrains Mono, monospace");
    }

    #[test]
    fn parse_dtcg_dimensions_radius() {
        let dtcg = r#"{
            "borderRadius": {
                "$type": "dimension",
                "sm": { "$value": "0.25rem" },
                "md": { "$value": "0.5rem" },
                "lg": { "$value": "1rem" }
            }
        }"#;
        let input = parse_dtcg(dtcg).expect("should parse");
        // borderRadius contains "radius" so tokens go into radius map.
        assert_eq!(input.radius.len(), 3);
        assert_eq!(input.radius["sm"], "0.25rem");
        assert_eq!(input.radius["md"], "0.5rem");
        assert_eq!(input.radius["lg"], "1rem");
    }

    #[test]
    fn parse_dtcg_inherited_type() {
        let dtcg = r##"{
            "color": {
                "$type": "color",
                "primary": { "$value": "#3b82f6" },
                "danger": { "$value": "#ef4444" }
            }
        }"##;
        let input = parse_dtcg(dtcg).expect("should parse");
        assert_eq!(input.colors.len(), 2);
        assert!(input.colors.contains_key("primary"));
        assert!(input.colors.contains_key("danger"));
    }

    #[test]
    fn parse_dtcg_auto_dark_mode() {
        let dtcg = r##"{
            "color": {
                "primary": {
                    "$type": "color",
                    "$value": "#3b82f6"
                }
            }
        }"##;
        let input = parse_dtcg(dtcg).expect("should parse");
        let primary = &input.colors["primary"];
        // Light and dark should have different lightness values.
        assert!(
            (primary.light.l - primary.dark.l).abs() > 0.1,
            "auto-derived dark should differ in lightness: light={}, dark={}",
            primary.light.l,
            primary.dark.l
        );
    }

    #[test]
    fn parse_dtcg_via_auto_detect() {
        let dtcg = r##"{
            "color": {
                "surface": {
                    "$type": "color",
                    "$value": "#f5f5f4"
                }
            }
        }"##;
        // parse() should auto-detect DTCG and route correctly.
        let result = parse(dtcg);
        assert!(result.is_ok(), "auto-detect should handle DTCG: {result:?}");
        let input = result.expect("should parse");
        assert!(input.colors.contains_key("surface"));
    }

    #[test]
    fn parse_dtcg_invalid_hex() {
        let dtcg = r##"{
            "color": {
                "bad": {
                    "$type": "color",
                    "$value": "#xyz"
                }
            }
        }"##;
        let result = parse_dtcg(dtcg);
        assert!(result.is_err());
        let issues = result.expect_err("should fail");
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("Invalid hex color"))
        );
    }

    #[test]
    fn parse_dtcg_spacing() {
        let dtcg = r#"{
            "spacing": {
                "$type": "dimension",
                "sm": { "$value": "0.5rem" },
                "md": { "$value": "1rem" }
            }
        }"#;
        let input = parse_dtcg(dtcg).expect("should parse");
        // "spacing" path contains "spacing" so they go to the spacing scale.
        assert!(
            input.spacing.scale.len() > 2,
            "should have added spacing entries"
        );
    }
}
