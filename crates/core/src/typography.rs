//! Typography scale resolver.
//!
//! Generates 13 type roles from a base size and modular-scale ratio,
//! mapping each role to its computed CSS properties.

use std::collections::BTreeMap;

use crate::types::{FontConfig, ResolvedTypeRole, TypographyConfig};

/// The 16px browser default used for px → rem conversion.
const PX_PER_REM: f64 = 16.0;

/// Minimum viewport width for fluid clamp (in rem).
const FLUID_MIN_VW: f64 = 20.0; // 320px
/// Maximum viewport width for fluid clamp (in rem).
const FLUID_MAX_VW: f64 = 80.0; // 1280px
/// Scale factor applied to the minimum fluid font size (percentage of computed size).
const FLUID_MIN_SCALE: f64 = 0.75;

/// Resolve typography configuration into 13 named type roles.
///
/// # Roles generated
///
/// **Display tier** (uses display font, bold weight):
/// - `display` — scale^6
/// - `h1` — scale^4
/// - `h2` — scale^3
/// - `h3` — scale^1.5
///
/// **Label tier** (uses body font, medium weight):
/// - `label-lg` — scale^1
/// - `label-md` — scale^0 (= base)
/// - `label-sm` — scale^-1
///
/// **Body tier** (uses body font, regular weight, relaxed leading):
/// - `body-lg` — scale^1
/// - `body-md` — base
/// - `body-sm` — scale^-1
///
/// **Utility** (special treatments):
/// - `overline` — scale^-2, uppercase, wide tracking
/// - `caption` — scale^-2
/// - `code` — scale^-1, mono font
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "flat role list is clearer than abstracting into data tables"
)]
pub fn resolve_typography(config: &TypographyConfig, fonts: &FontConfig) -> Vec<ResolvedTypeRole> {
    let base = config.base_size;
    let ratio = config.scale_ratio;

    let mut roles = Vec::with_capacity(13);

    // Helper: compute font size in px from a scale exponent.
    let scaled = |exp: f64| -> f64 { base * ratio.powf(exp) };

    // Display tier
    roles.push(build_role(
        "display",
        scaled(6.0),
        &fonts.display,
        "700",
        "1.1",
        None,
        config,
    ));
    roles.push(build_role(
        "h1",
        scaled(4.0),
        &fonts.display,
        "700",
        "1.15",
        None,
        config,
    ));
    roles.push(build_role(
        "h2",
        scaled(3.0),
        &fonts.display,
        "600",
        "1.2",
        None,
        config,
    ));
    roles.push(build_role(
        "h3",
        scaled(1.5),
        &fonts.display,
        "600",
        "1.3",
        None,
        config,
    ));

    // Label tier
    roles.push(build_role(
        "label-lg",
        scaled(1.0),
        &fonts.body,
        "500",
        "1.4",
        None,
        config,
    ));
    roles.push(build_role(
        "label-md",
        scaled(0.0),
        &fonts.body,
        "500",
        "1.4",
        None,
        config,
    ));
    roles.push(build_role(
        "label-sm",
        scaled(-1.0),
        &fonts.body,
        "500",
        "1.4",
        None,
        config,
    ));

    // Body tier
    roles.push(build_role(
        "body-lg",
        scaled(1.0),
        &fonts.body,
        "400",
        "1.6",
        None,
        config,
    ));
    roles.push(build_role(
        "body-md",
        scaled(0.0),
        &fonts.body,
        "400",
        "1.6",
        None,
        config,
    ));
    roles.push(build_role(
        "body-sm",
        scaled(-1.0),
        &fonts.body,
        "400",
        "1.6",
        None,
        config,
    ));

    // Utility tier
    roles.push(build_role(
        "overline",
        scaled(-2.0),
        &fonts.body,
        "600",
        "1.4",
        Some(("text-transform", "uppercase")),
        config,
    ));
    roles.push(build_role(
        "caption",
        scaled(-2.0),
        &fonts.body,
        "400",
        "1.4",
        None,
        config,
    ));
    roles.push(build_role(
        "code",
        scaled(-1.0),
        &fonts.mono,
        "400",
        "1.6",
        None,
        config,
    ));

    // Apply per-role overrides from config
    for role in &mut roles {
        if let Some(overrides) = config.roles.get(&role.name) {
            for (prop, value) in overrides {
                role.css_properties.insert(prop.clone(), value.clone());
            }
        }
    }

    roles
}

/// Build a single resolved type role.
fn build_role(
    name: &str,
    size_px: f64,
    font_family: &str,
    font_weight: &str,
    line_height: &str,
    extra: Option<(&str, &str)>,
    config: &TypographyConfig,
) -> ResolvedTypeRole {
    let size_rem = size_px / PX_PER_REM;
    let size_str = format_rem(size_rem);

    let mut css_properties = BTreeMap::new();
    css_properties.insert("font-family".to_owned(), font_family.to_owned());
    css_properties.insert("font-size".to_owned(), size_str);
    css_properties.insert("font-weight".to_owned(), font_weight.to_owned());
    css_properties.insert("line-height".to_owned(), line_height.to_owned());

    // Letter spacing: tighter for large sizes, normal for body, wider for small utility
    let letter_spacing = compute_letter_spacing(name, size_px);
    css_properties.insert("letter-spacing".to_owned(), letter_spacing);

    if let Some((prop, value)) = extra {
        css_properties.insert(prop.to_owned(), value.to_owned());
    }

    // Fluid headings: display, h1, h2 get clamp() values
    let fluid_size = if config.fluid_headings && is_fluid_eligible(name) {
        Some(build_clamp(size_rem))
    } else {
        None
    };

    ResolvedTypeRole {
        name: name.to_owned(),
        css_properties,
        fluid_size,
    }
}

/// Check if a role is eligible for fluid sizing.
fn is_fluid_eligible(name: &str) -> bool {
    matches!(name, "display" | "h1" | "h2")
}

/// Build a CSS `clamp()` expression for fluid typography.
///
/// `clamp(min, preferred, max)` where:
/// - min = `FLUID_MIN_SCALE` × max
/// - preferred = linear interpolation between min and max viewports
/// - max = the computed rem size
fn build_clamp(max_rem: f64) -> String {
    let min_rem = max_rem * FLUID_MIN_SCALE;
    let slope = (max_rem - min_rem) / (FLUID_MAX_VW - FLUID_MIN_VW);
    let intercept = min_rem - slope * FLUID_MIN_VW;

    let min_str = format_rem(min_rem);
    let intercept_str = format_rem(intercept);
    let slope_pct = format!("{:.4}", slope * 100.0)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned();
    let max_str = format_rem(max_rem);

    format!("clamp({min_str}, {intercept_str} + {slope_pct}vw, {max_str})")
}

/// Compute letter-spacing based on role and size.
fn compute_letter_spacing(name: &str, size_px: f64) -> String {
    match name {
        "display" | "h1" => "-0.02em".to_owned(),
        "h2" | "h3" => "-0.01em".to_owned(),
        "overline" => "0.08em".to_owned(),
        "caption" => "0.02em".to_owned(),
        _ => {
            if size_px < 14.0 {
                "0.01em".to_owned()
            } else {
                "normal".to_owned()
            }
        }
    }
}

/// Format a rem value with reasonable precision.
fn format_rem(rem: f64) -> String {
    let s = format!("{rem:.4}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    // Ensure at least one decimal place for clean CSS output
    if s.contains('.') {
        format!("{s}rem")
    } else {
        format!("{s}.0rem")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_fonts() -> FontConfig {
        FontConfig {
            display: "Inter, sans-serif".to_owned(),
            body: "Inter, sans-serif".to_owned(),
            mono: "JetBrains Mono, monospace".to_owned(),
        }
    }

    #[test]
    fn generates_13_roles() {
        let config = TypographyConfig::default();
        let roles = resolve_typography(&config, &default_fonts());
        assert_eq!(roles.len(), 13);
    }

    #[test]
    fn body_md_is_base_size() {
        let config = TypographyConfig {
            base_size: 16.0,
            ..TypographyConfig::default()
        };
        let roles = resolve_typography(&config, &default_fonts());
        let body_md = roles
            .iter()
            .find(|r| r.name == "body-md")
            .expect("body-md not found");
        assert_eq!(body_md.css_properties["font-size"], "1.0rem");
    }

    #[test]
    fn display_is_largest() {
        let config = TypographyConfig::default();
        let roles = resolve_typography(&config, &default_fonts());
        let display = roles.iter().find(|r| r.name == "display").expect("display");
        let body_md = roles.iter().find(|r| r.name == "body-md").expect("body-md");

        let display_size: f64 = display.css_properties["font-size"]
            .trim_end_matches("rem")
            .parse()
            .expect("parse display size");
        let body_size: f64 = body_md.css_properties["font-size"]
            .trim_end_matches("rem")
            .parse()
            .expect("parse body size");

        assert!(display_size > body_size);
    }

    #[test]
    fn fluid_headings_generate_clamp() {
        let config = TypographyConfig {
            fluid_headings: true,
            ..TypographyConfig::default()
        };
        let roles = resolve_typography(&config, &default_fonts());
        let display = roles.iter().find(|r| r.name == "display").expect("display");
        assert!(display.fluid_size.is_some());
        assert!(
            display
                .fluid_size
                .as_ref()
                .is_some_and(|s| s.starts_with("clamp("))
        );

        // h3 should NOT get fluid
        let h3 = roles.iter().find(|r| r.name == "h3").expect("h3");
        assert!(h3.fluid_size.is_none());
    }

    #[test]
    fn code_uses_mono_font() {
        let fonts = default_fonts();
        let config = TypographyConfig::default();
        let roles = resolve_typography(&config, &fonts);
        let code = roles.iter().find(|r| r.name == "code").expect("code");
        assert_eq!(code.css_properties["font-family"], fonts.mono);
    }

    #[test]
    fn all_13_role_names_present_and_unique() {
        let config = TypographyConfig::default();
        let roles = resolve_typography(&config, &default_fonts());

        let expected = [
            "display", "h1", "h2", "h3", "label-lg", "label-md", "label-sm", "body-lg", "body-md",
            "body-sm", "overline", "caption", "code",
        ];

        // Exactly 13 roles
        assert_eq!(roles.len(), expected.len(), "expected exactly 13 roles");

        // All expected names are present
        let names: Vec<&str> = roles.iter().map(|r| r.name.as_str()).collect();
        for &expected_name in &expected {
            assert!(
                names.contains(&expected_name),
                "missing role: {expected_name}"
            );
        }

        // No duplicates
        let mut seen = std::collections::HashSet::new();
        for name in &names {
            assert!(seen.insert(name), "duplicate role name: {name}");
        }
    }

    #[test]
    fn overrides_apply() {
        let mut overrides = BTreeMap::new();
        overrides.insert("font-weight".to_owned(), "900".to_owned());
        let mut roles_map = BTreeMap::new();
        roles_map.insert("display".to_owned(), overrides);

        let config = TypographyConfig {
            roles: roles_map,
            ..TypographyConfig::default()
        };
        let roles = resolve_typography(&config, &default_fonts());
        let display = roles.iter().find(|r| r.name == "display").expect("display");
        assert_eq!(display.css_properties["font-weight"], "900");
    }
}
