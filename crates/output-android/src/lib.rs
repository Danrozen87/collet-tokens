//! Android XML resource output generator for collet-tokens.
//!
//! Generates Android resource XML files from [`ResolvedTokens`]:
//! - `values/colors.xml` — light-mode color resources
//! - `values-night/colors.xml` — dark-mode color resources
//! - `values/dimens.xml` — spacing and radius dimension resources
//! - `values/type.xml` — font family references and text appearance styles

use std::fmt::Write;

use collet_tokens_core::OutputFile;
use collet_tokens_core::types::{OklchColor, ResolvedTokens};

/// Generate Android resource XML files from resolved tokens.
///
/// Returns up to four [`OutputFile`] entries for color, dimension, and
/// typography resources.
#[must_use]
pub fn generate(tokens: &ResolvedTokens) -> Vec<OutputFile> {
    vec![
        generate_colors_light(tokens),
        generate_colors_dark(tokens),
        generate_dimens(tokens),
        generate_type(tokens),
    ]
}

/// Generate `values/colors.xml` with light-mode color resources.
fn generate_colors_light(tokens: &ResolvedTokens) -> OutputFile {
    let mut xml = String::with_capacity(1024);
    write_xml_header(&mut xml);
    let _ = writeln!(xml, "<resources>");

    for color in &tokens.colors {
        let hex = oklch_to_android_hex(&color.light);
        let name = to_android_name(&color.name);
        let _ = writeln!(xml, "    <color name=\"{name}\">{hex}</color>");
    }

    let _ = writeln!(xml, "</resources>");

    OutputFile {
        path: "values/colors.xml".to_owned(),
        content: xml,
    }
}

/// Generate `values-night/colors.xml` with dark-mode color resources.
fn generate_colors_dark(tokens: &ResolvedTokens) -> OutputFile {
    let mut xml = String::with_capacity(1024);
    write_xml_header(&mut xml);
    let _ = writeln!(xml, "<resources>");

    for color in &tokens.colors {
        let hex = oklch_to_android_hex(&color.dark);
        let name = to_android_name(&color.name);
        let _ = writeln!(xml, "    <color name=\"{name}\">{hex}</color>");
    }

    let _ = writeln!(xml, "</resources>");

    OutputFile {
        path: "values-night/colors.xml".to_owned(),
        content: xml,
    }
}

/// Generate `values/dimens.xml` with spacing and radius dimension resources.
fn generate_dimens(tokens: &ResolvedTokens) -> OutputFile {
    let mut xml = String::with_capacity(1024);
    write_xml_header(&mut xml);
    let _ = writeln!(xml, "<resources>");

    // Spacing values.
    if !tokens.spacing.is_empty() {
        let _ = writeln!(xml, "    <!-- Spacing -->");
        for sp in &tokens.spacing {
            let name = format!("space_{}", to_android_name(&sp.name));
            // Use dp (density-independent pixels) — 1dp ≈ 1px at mdpi.
            let _ = writeln!(
                xml,
                "    <dimen name=\"{name}\">{:.1}dp</dimen>",
                sp.value_px
            );
        }
    }

    // Radius values.
    if !tokens.radius.is_empty() {
        let _ = writeln!(xml);
        let _ = writeln!(xml, "    <!-- Border Radius -->");
        for r in &tokens.radius {
            let name = format!("radius_{}", to_android_name(&r.name));
            let dp_value = css_value_to_dp(&r.value);
            let _ = writeln!(xml, "    <dimen name=\"{name}\">{dp_value}</dimen>");
        }
    }

    let _ = writeln!(xml, "</resources>");

    OutputFile {
        path: "values/dimens.xml".to_owned(),
        content: xml,
    }
}

/// Generate `values/type.xml` with font family references and text appearance styles.
fn generate_type(tokens: &ResolvedTokens) -> OutputFile {
    let mut xml = String::with_capacity(2048);
    write_xml_header(&mut xml);
    let _ = writeln!(xml, "<resources>");

    // Font family strings.
    let _ = writeln!(xml, "    <!-- Font Families -->");
    let _ = writeln!(
        xml,
        "    <string name=\"font_display\" translatable=\"false\">{}</string>",
        escape_xml(&tokens.fonts.display)
    );
    let _ = writeln!(
        xml,
        "    <string name=\"font_body\" translatable=\"false\">{}</string>",
        escape_xml(&tokens.fonts.body)
    );
    let _ = writeln!(
        xml,
        "    <string name=\"font_mono\" translatable=\"false\">{}</string>",
        escape_xml(&tokens.fonts.mono)
    );
    let _ = writeln!(xml);

    // Text appearance styles.
    let _ = writeln!(xml, "    <!-- Text Appearance Styles -->");
    for role in &tokens.typography {
        let style_name = format!("TextAppearance_{}", to_pascal_case(&role.name));
        let _ = writeln!(xml, "    <style name=\"{style_name}\">");

        if let Some(size) = role.css_properties.get("font-size") {
            let sp_value = css_value_to_sp(size);
            let _ = writeln!(
                xml,
                "        <item name=\"android:textSize\">{sp_value}</item>"
            );
        }

        if let Some(weight) = role.css_properties.get("font-weight") {
            let style = css_weight_to_android_style(weight);
            if !style.is_empty() {
                let _ = writeln!(
                    xml,
                    "        <item name=\"android:textStyle\">{style}</item>"
                );
            }
        }

        if let Some(line_height) = role.css_properties.get("line-height") {
            if let Ok(lh) = line_height.parse::<f64>() {
                if let Some(size) = role.css_properties.get("font-size") {
                    let size_px = css_rem_to_px(size);
                    let lh_sp = lh * size_px;
                    let _ = writeln!(
                        xml,
                        "        <item name=\"android:lineHeight\">{lh_sp:.1}sp</item>"
                    );
                }
            }
        }

        if let Some(spacing) = role.css_properties.get("letter-spacing") {
            let em_value = css_letter_spacing_to_em(spacing);
            if em_value.abs() > f64::EPSILON {
                let _ = writeln!(
                    xml,
                    "        <item name=\"android:letterSpacing\">{em_value:.3}</item>"
                );
            }
        }

        let _ = writeln!(xml, "    </style>");
        let _ = writeln!(xml);
    }

    let _ = writeln!(xml, "</resources>");

    OutputFile {
        path: "values/type.xml".to_owned(),
        content: xml,
    }
}

/// Write the standard XML header.
fn write_xml_header(xml: &mut String) {
    let _ = writeln!(xml, "<?xml version=\"1.0\" encoding=\"utf-8\"?>");
    let _ = writeln!(
        xml,
        "<!-- Generated by collet-tokens — do not edit manually -->"
    );
}

/// Convert an [`OklchColor`] to an Android hex color string (`#RRGGBB`).
#[must_use]
pub fn oklch_to_android_hex(color: &OklchColor) -> String {
    color.to_hex().to_uppercase()
}

/// Convert a kebab-case token name to an Android resource name (`snake_case`).
///
/// E.g., `"text-primary"` → `"text_primary"`, `"surface-raised"` → `"surface_raised"`.
fn to_android_name(name: &str) -> String {
    name.replace('-', "_")
}

/// Convert a kebab-case name to `PascalCase` for style names.
///
/// E.g., `"body-md"` → `"BodyMd"`, `"label-lg"` → `"LabelLg"`.
fn to_pascal_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut capitalize_next = true;

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

/// Escape a string for XML content.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Convert a CSS value (rem or px) to Android dp string.
///
/// Uses 1rem = 16dp, 1px = 1dp as a reasonable approximation.
fn css_value_to_dp(value: &str) -> String {
    let value = value.trim();
    if let Some(rem_str) = value.strip_suffix("rem") {
        if let Ok(rem) = rem_str.trim().parse::<f64>() {
            return format!("{:.1}dp", rem * 16.0);
        }
    }
    if let Some(px_str) = value.strip_suffix("px") {
        if let Ok(px) = px_str.trim().parse::<f64>() {
            return format!("{px:.1}dp");
        }
    }
    // Fallback: return as-is with dp suffix.
    format!("{value}dp")
}

/// Convert a CSS font-size (rem) to Android sp string.
///
/// Uses 1rem = 16sp.
fn css_value_to_sp(value: &str) -> String {
    let value = value.trim();
    if let Some(rem_str) = value.strip_suffix("rem") {
        if let Ok(rem) = rem_str.trim().parse::<f64>() {
            return format!("{:.1}sp", rem * 16.0);
        }
    }
    format!("{value}sp")
}

/// Convert a CSS rem value to pixels (for intermediate calculations).
fn css_rem_to_px(value: &str) -> f64 {
    value
        .trim()
        .strip_suffix("rem")
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map_or(16.0, |rem| rem * 16.0)
}

/// Map a CSS `font-weight` to an Android `textStyle` value.
///
/// Android only supports `normal`, `bold`, and `italic` natively.
/// Weights ≥ 600 map to `bold`; others to empty (normal is the default).
fn css_weight_to_android_style(weight: &str) -> &'static str {
    match weight.parse::<u16>() {
        Ok(w) if w >= 600 => "bold",
        _ => "",
    }
}

/// Convert CSS letter-spacing to Android `letterSpacing` (in em units).
///
/// Android `letterSpacing` is already in em, so we just parse the value.
fn css_letter_spacing_to_em(spacing: &str) -> f64 {
    if spacing == "normal" {
        return 0.0;
    }
    if let Some(em_str) = spacing.strip_suffix("em") {
        if let Ok(em) = em_str.parse::<f64>() {
            return em;
        }
    }
    0.0
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
                props.insert("font-weight".to_owned(), "700".to_owned());
                props.insert("line-height".to_owned(), "1.6".to_owned());
                props.insert("letter-spacing".to_owned(), "-0.02em".to_owned());
                props.insert("font-family".to_owned(), "Inter, sans-serif".to_owned());
                ResolvedTypeRole {
                    name: "body-md".to_owned(),
                    css_properties: props,
                    fluid_size: None,
                }
            }],
            spacing: vec![
                ResolvedSpacing {
                    name: "1".to_owned(),
                    value_rem: 0.25,
                    value_px: 4.0,
                },
                ResolvedSpacing {
                    name: "4".to_owned(),
                    value_rem: 1.0,
                    value_px: 16.0,
                },
            ],
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
    fn generates_four_files() {
        let files = generate(&minimal_tokens());
        assert_eq!(files.len(), 4);
        assert_eq!(files[0].path, "values/colors.xml");
        assert_eq!(files[1].path, "values-night/colors.xml");
        assert_eq!(files[2].path, "values/dimens.xml");
        assert_eq!(files[3].path, "values/type.xml");
    }

    #[test]
    fn light_colors_valid_xml() {
        let files = generate(&minimal_tokens());
        let content = &files[0].content;
        assert!(content.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(content.contains("<resources>"));
        assert!(content.contains("</resources>"));
        assert!(content.contains("<color name=\"surface\">"));
    }

    #[test]
    fn dark_colors_separate_file() {
        let files = generate(&minimal_tokens());
        let light = &files[0].content;
        let dark = &files[1].content;
        // Both should have <color> tags but with different values.
        assert!(light.contains("<color name=\"surface\">"));
        assert!(dark.contains("<color name=\"surface\">"));
        // Light surface is white (#FFFFFF), dark is near-black.
        assert!(light.contains("#FFFFFF"));
        assert!(!dark.contains("#FFFFFF"));
    }

    #[test]
    fn color_names_use_underscores() {
        let files = generate(&minimal_tokens());
        let content = &files[0].content;
        assert!(
            content.contains("text_primary"),
            "kebab-case should become snake_case"
        );
    }

    #[test]
    fn dimens_contains_spacing() {
        let files = generate(&minimal_tokens());
        let content = &files[2].content;
        assert!(content.contains("<dimen name=\"space_1\">4.0dp</dimen>"));
        assert!(content.contains("<dimen name=\"space_4\">16.0dp</dimen>"));
    }

    #[test]
    fn dimens_contains_radius() {
        let files = generate(&minimal_tokens());
        let content = &files[2].content;
        assert!(content.contains("<dimen name=\"radius_md\">8.0dp</dimen>"));
    }

    #[test]
    fn type_contains_font_families() {
        let files = generate(&minimal_tokens());
        let content = &files[3].content;
        assert!(content.contains("<string name=\"font_display\""));
        assert!(content.contains("<string name=\"font_body\""));
        assert!(content.contains("<string name=\"font_mono\""));
    }

    #[test]
    fn type_contains_text_appearance() {
        let files = generate(&minimal_tokens());
        let content = &files[3].content;
        assert!(content.contains("<style name=\"TextAppearance_BodyMd\">"));
        assert!(content.contains("android:textSize"));
    }

    #[test]
    fn type_bold_weight() {
        let files = generate(&minimal_tokens());
        let content = &files[3].content;
        // 700 weight should map to bold style.
        assert!(content.contains("android:textStyle\">bold</item>"));
    }

    #[test]
    fn type_letter_spacing() {
        let files = generate(&minimal_tokens());
        let content = &files[3].content;
        assert!(content.contains("android:letterSpacing\">-0.020</item>"));
    }

    #[test]
    fn to_pascal_case_converts() {
        assert_eq!(to_pascal_case("body-md"), "BodyMd");
        assert_eq!(to_pascal_case("label-lg"), "LabelLg");
        assert_eq!(to_pascal_case("display"), "Display");
    }

    #[test]
    fn to_android_name_converts() {
        assert_eq!(to_android_name("text-primary"), "text_primary");
        assert_eq!(to_android_name("surface"), "surface");
    }

    #[test]
    fn css_value_to_dp_converts() {
        assert_eq!(css_value_to_dp("0.5rem"), "8.0dp");
        assert_eq!(css_value_to_dp("16px"), "16.0dp");
        assert_eq!(css_value_to_dp("9999px"), "9999.0dp");
    }

    #[test]
    fn valid_xml_structure() {
        let files = generate(&minimal_tokens());
        for file in &files {
            assert!(
                file.content.contains("<?xml"),
                "missing xml header in {}",
                file.path
            );
            assert!(
                file.content.contains("<resources>"),
                "missing resources open in {}",
                file.path
            );
            assert!(
                file.content.contains("</resources>"),
                "missing resources close in {}",
                file.path
            );
        }
    }
}
