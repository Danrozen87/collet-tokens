//! iOS Swift output generator for collet-tokens.
//!
//! Generates two Swift files from [`ResolvedTokens`]:
//! - `Colors.swift` — `UIColor` extensions with programmatic dark mode via
//!   `UIColor.init(dynamicProvider:)`
//! - `Typography.swift` — font definitions mapping the 13 type roles to
//!   `UIFont` descriptors

use std::fmt::Write;

use collet_tokens_core::OutputFile;
use collet_tokens_core::color::oklch_to_srgb;
use collet_tokens_core::types::{OklchColor, ResolvedTokens};

/// Generate iOS Swift output files from resolved tokens.
///
/// Returns two [`OutputFile`] entries: `Colors.swift` and `Typography.swift`.
#[must_use]
pub fn generate(tokens: &ResolvedTokens) -> Vec<OutputFile> {
    vec![generate_colors(tokens), generate_typography(tokens)]
}

/// Generate `Colors.swift` with dynamic light/dark mode color definitions.
fn generate_colors(tokens: &ResolvedTokens) -> OutputFile {
    let mut swift = String::with_capacity(2048);

    write_file_header(&mut swift, "Colors");
    let _ = writeln!(swift, "import UIKit");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "// MARK: - Design Token Colors");
    let _ = writeln!(swift);
    let _ = writeln!(
        swift,
        "/// Design token colors with automatic light/dark mode support."
    );
    let _ = writeln!(swift, "///");
    let _ = writeln!(
        swift,
        "/// Generated from the collet-tokens design token compiler."
    );
    let _ = writeln!(swift, "extension UIColor {{");
    let _ = writeln!(swift);

    for color in &tokens.colors {
        let swift_name = to_camel_case(&color.name);
        let (lr, lg, lb) = oklch_to_srgb(&color.light);
        let (dr, dg, db) = oklch_to_srgb(&color.dark);

        let _ = writeln!(swift, "    /// Design token color: `{}`.", color.name);
        let _ = writeln!(
            swift,
            "    static let {swift_name} = UIColor {{ traitCollection in"
        );
        let _ = writeln!(
            swift,
            "        if traitCollection.userInterfaceStyle == .dark {{"
        );
        let _ = writeln!(
            swift,
            "            return UIColor(red: {dr:.4}, green: {dg:.4}, blue: {db:.4}, alpha: 1.0)"
        );
        let _ = writeln!(swift, "        }} else {{");
        let _ = writeln!(
            swift,
            "            return UIColor(red: {lr:.4}, green: {lg:.4}, blue: {lb:.4}, alpha: 1.0)"
        );
        let _ = writeln!(swift, "        }}");
        let _ = writeln!(swift, "    }}");
        let _ = writeln!(swift);
    }

    let _ = writeln!(swift, "}}");
    let _ = writeln!(swift);

    // Also generate SwiftUI Color extension.
    let _ = writeln!(swift, "import SwiftUI");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "// MARK: - SwiftUI Color Extensions");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "/// SwiftUI bridge for design token colors.");
    let _ = writeln!(swift, "extension Color {{");
    let _ = writeln!(swift);

    for color in &tokens.colors {
        let swift_name = to_camel_case(&color.name);
        let _ = writeln!(swift, "    /// Design token color: `{}`.", color.name);
        let _ = writeln!(
            swift,
            "    static let {swift_name} = Color(uiColor: .{swift_name})"
        );
        let _ = writeln!(swift);
    }

    let _ = writeln!(swift, "}}");

    OutputFile {
        path: "Colors.swift".to_owned(),
        content: swift,
    }
}

/// Generate `Typography.swift` with font definitions for the 13 type roles.
fn generate_typography(tokens: &ResolvedTokens) -> OutputFile {
    let mut swift = String::with_capacity(2048);

    write_file_header(&mut swift, "Typography");
    let _ = writeln!(swift, "import UIKit");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "// MARK: - Design Token Typography");
    let _ = writeln!(swift);
    let _ = writeln!(
        swift,
        "/// Typography role descriptor containing all properties for a text style."
    );
    let _ = writeln!(swift, "struct TypographyRole {{");
    let _ = writeln!(swift, "    /// Font name or system reference.");
    let _ = writeln!(swift, "    let fontFamily: String");
    let _ = writeln!(swift, "    /// Font size in points.");
    let _ = writeln!(swift, "    let fontSize: CGFloat");
    let _ = writeln!(swift, "    /// Font weight as a `UIFont.Weight` value.");
    let _ = writeln!(swift, "    let fontWeight: UIFont.Weight");
    let _ = writeln!(swift, "    /// Line height multiplier.");
    let _ = writeln!(swift, "    let lineHeight: CGFloat");
    let _ = writeln!(swift, "    /// Letter spacing in points.");
    let _ = writeln!(swift, "    let letterSpacing: CGFloat");
    let _ = writeln!(swift, "}}");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "// MARK: - Font Stacks");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "/// Design token font family stacks.");
    let _ = writeln!(swift, "enum DesignFonts {{");
    let _ = writeln!(swift, "    /// Display / heading font stack.");
    let _ = writeln!(
        swift,
        r#"    static let display = "{}""#,
        escape_swift(&tokens.fonts.display)
    );
    let _ = writeln!(swift, "    /// Body / paragraph font stack.");
    let _ = writeln!(
        swift,
        r#"    static let body = "{}""#,
        escape_swift(&tokens.fonts.body)
    );
    let _ = writeln!(swift, "    /// Monospace / code font stack.");
    let _ = writeln!(
        swift,
        r#"    static let mono = "{}""#,
        escape_swift(&tokens.fonts.mono)
    );
    let _ = writeln!(swift, "}}");
    let _ = writeln!(swift);
    let _ = writeln!(swift, "// MARK: - Type Roles");
    let _ = writeln!(swift);
    let _ = writeln!(
        swift,
        "/// Pre-computed typography roles from the design token scale."
    );
    let _ = writeln!(swift, "enum DesignTypography {{");
    let _ = writeln!(swift);

    for role in &tokens.typography {
        let swift_name = to_camel_case(&role.name);
        let font_size = extract_font_size_pt(&role.css_properties);
        let font_weight = css_weight_to_swift(
            role.css_properties
                .get("font-weight")
                .map_or("400", String::as_str),
        );
        let line_height = role
            .css_properties
            .get("line-height")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.5);
        let letter_spacing = parse_letter_spacing(
            role.css_properties
                .get("letter-spacing")
                .map_or("normal", String::as_str),
            font_size,
        );
        let font_family = role
            .css_properties
            .get("font-family")
            .map_or("system", String::as_str);

        let _ = writeln!(swift, "    /// Typography role: `{}`.", role.name);
        let _ = writeln!(swift, "    static let {swift_name} = TypographyRole(");
        let _ = writeln!(
            swift,
            r#"        fontFamily: "{}","#,
            escape_swift(font_family)
        );
        let _ = writeln!(swift, "        fontSize: {font_size:.1},");
        let _ = writeln!(swift, "        fontWeight: {font_weight},");
        let _ = writeln!(swift, "        lineHeight: {line_height:.2},");
        let _ = writeln!(swift, "        letterSpacing: {letter_spacing:.3}");
        let _ = writeln!(swift, "    )");
        let _ = writeln!(swift);
    }

    let _ = writeln!(swift, "}}");

    OutputFile {
        path: "Typography.swift".to_owned(),
        content: swift,
    }
}

/// Write a file header comment.
fn write_file_header(swift: &mut String, name: &str) {
    let _ = writeln!(swift, "// {name}.swift");
    let _ = writeln!(
        swift,
        "// Generated by collet-tokens — do not edit manually"
    );
    let _ = writeln!(swift);
}

/// Convert a kebab-case token name to `camelCase` for Swift.
///
/// E.g., `"text-primary"` → `"textPrimary"`, `"surface-raised"` → `"surfaceRaised"`.
fn to_camel_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut capitalize_next = false;

    for ch in name.chars() {
        if ch == '-' || ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            for upper in ch.to_uppercase() {
                result.push(upper);
            }
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Escape a string for Swift string literals.
fn escape_swift(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Extract font size in points from CSS properties.
///
/// Parses the `font-size` value (expected in rem) and converts to points
/// using the standard 1rem = 16pt assumption.
fn extract_font_size_pt(props: &std::collections::BTreeMap<String, String>) -> f64 {
    props
        .get("font-size")
        .and_then(|s| {
            s.trim_end_matches("rem")
                .parse::<f64>()
                .ok()
                .map(|rem| rem * 16.0)
        })
        .unwrap_or(16.0)
}

/// Map a CSS `font-weight` value to a Swift `UIFont.Weight` constant.
fn css_weight_to_swift(weight: &str) -> &'static str {
    match weight {
        "100" => ".ultraLight",
        "200" => ".thin",
        "300" => ".light",
        "500" => ".medium",
        "600" => ".semibold",
        "700" => ".bold",
        "800" => ".heavy",
        "900" => ".black",
        _ => ".regular",
    }
}

/// Parse CSS letter-spacing to points.
///
/// Handles `em` units (relative to font size) and `normal` (0.0).
fn parse_letter_spacing(value: &str, font_size_pt: f64) -> f64 {
    if value == "normal" {
        return 0.0;
    }
    if let Some(em_str) = value.strip_suffix("em") {
        if let Ok(em) = em_str.parse::<f64>() {
            return em * font_size_pt;
        }
    }
    0.0
}

/// Convert an [`OklchColor`] to a Swift `UIColor` literal string.
///
/// Returns a string like `UIColor(red: 0.2, green: 0.4, blue: 0.8, alpha: 1.0)`.
#[must_use]
pub fn oklch_to_swift_color(color: &OklchColor) -> String {
    let (r, g, b) = oklch_to_srgb(color);
    format!("UIColor(red: {r:.4}, green: {g:.4}, blue: {b:.4}, alpha: 1.0)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use collet_tokens_core::OklchColor;
    use collet_tokens_core::types::{
        ResolvedColor, ResolvedFonts, ResolvedMotion, ResolvedRadius, ResolvedSpacing,
        ResolvedTypeRole,
    };
    use std::collections::BTreeMap;

    fn minimal_tokens() -> ResolvedTokens {
        ResolvedTokens {
            fonts: ResolvedFonts {
                display: "Inter, sans-serif".to_owned(),
                body: "Inter, sans-serif".to_owned(),
                mono: "JetBrains Mono, monospace".to_owned(),
            },
            colors: vec![
                ResolvedColor {
                    name: "surface".to_owned(),
                    css_var: "--color-surface".to_owned(),
                    light: OklchColor::new(1.0, 0.0, 0.0),
                    dark: OklchColor::new(0.13, 0.0, 0.0),
                    light_css: "oklch(1 0 0)".to_owned(),
                    dark_css: "oklch(0.13 0 0)".to_owned(),
                },
                ResolvedColor {
                    name: "text-primary".to_owned(),
                    css_var: "--color-text-primary".to_owned(),
                    light: OklchColor::new(0.27, 0.003, 90.0),
                    dark: OklchColor::new(0.87, 0.0, 0.0),
                    light_css: "oklch(0.27 0.003 90)".to_owned(),
                    dark_css: "oklch(0.87 0 0)".to_owned(),
                },
            ],
            typography: vec![{
                let mut props = BTreeMap::new();
                props.insert("font-size".to_owned(), "1.0rem".to_owned());
                props.insert("font-weight".to_owned(), "400".to_owned());
                props.insert("line-height".to_owned(), "1.6".to_owned());
                props.insert("letter-spacing".to_owned(), "normal".to_owned());
                props.insert("font-family".to_owned(), "Inter, sans-serif".to_owned());
                ResolvedTypeRole {
                    name: "body-md".to_owned(),
                    css_properties: props,
                    fluid_size: None,
                }
            }],
            spacing: vec![ResolvedSpacing {
                name: "1".to_owned(),
                value_rem: 0.25,
                value_px: 4.0,
            }],
            radius: vec![ResolvedRadius {
                name: "md".to_owned(),
                value: "0.5rem".to_owned(),
            }],
            motion: ResolvedMotion {
                durations: BTreeMap::new(),
                easings: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn generates_two_files() {
        let files = generate(&minimal_tokens());
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "Colors.swift");
        assert_eq!(files[1].path, "Typography.swift");
    }

    #[test]
    fn colors_contains_uicolor_extension() {
        let files = generate(&minimal_tokens());
        let content = &files[0].content;
        assert!(content.contains("extension UIColor {"));
        assert!(content.contains("import UIKit"));
    }

    #[test]
    fn colors_contains_dynamic_provider() {
        let files = generate(&minimal_tokens());
        let content = &files[0].content;
        assert!(content.contains("traitCollection.userInterfaceStyle == .dark"));
        assert!(content.contains("static let surface"));
    }

    #[test]
    fn colors_contains_swiftui_extension() {
        let files = generate(&minimal_tokens());
        let content = &files[0].content;
        assert!(content.contains("import SwiftUI"));
        assert!(content.contains("extension Color {"));
    }

    #[test]
    fn colors_camel_cases_names() {
        let files = generate(&minimal_tokens());
        let content = &files[0].content;
        assert!(
            content.contains("static let textPrimary"),
            "should camelCase text-primary: {content}"
        );
    }

    #[test]
    fn typography_contains_role() {
        let files = generate(&minimal_tokens());
        let content = &files[1].content;
        assert!(content.contains("struct TypographyRole {"));
        assert!(content.contains("static let bodyMd"));
    }

    #[test]
    fn typography_contains_font_stacks() {
        let files = generate(&minimal_tokens());
        let content = &files[1].content;
        assert!(content.contains("enum DesignFonts {"));
        assert!(content.contains("Inter, sans-serif"));
    }

    #[test]
    fn typography_contains_weight() {
        let files = generate(&minimal_tokens());
        let content = &files[1].content;
        assert!(
            content.contains(".regular"),
            "body-md should have .regular weight"
        );
    }

    #[test]
    fn to_camel_case_converts() {
        assert_eq!(to_camel_case("text-primary"), "textPrimary");
        assert_eq!(to_camel_case("surface-raised"), "surfaceRaised");
        assert_eq!(to_camel_case("surface"), "surface");
        assert_eq!(to_camel_case("body-md"), "bodyMd");
    }

    #[test]
    fn css_weight_to_swift_maps_correctly() {
        assert_eq!(css_weight_to_swift("700"), ".bold");
        assert_eq!(css_weight_to_swift("400"), ".regular");
        assert_eq!(css_weight_to_swift("600"), ".semibold");
    }

    #[test]
    fn swift_color_string_format() {
        let color = OklchColor::new(1.0, 0.0, 0.0);
        let swift = oklch_to_swift_color(&color);
        assert!(swift.contains("UIColor(red:"));
        assert!(swift.contains("alpha: 1.0"));
    }

    #[test]
    fn valid_swift_syntax_no_unclosed_braces() {
        let files = generate(&minimal_tokens());
        for file in &files {
            let opens = file.content.matches('{').count();
            let closes = file.content.matches('}').count();
            assert_eq!(opens, closes, "mismatched braces in {}", file.path);
        }
    }
}
