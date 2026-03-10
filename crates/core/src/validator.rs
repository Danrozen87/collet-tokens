//! Token validation engine.
//!
//! Validates a parsed [`TokenInput`] and produces a list of [`Issue`]
//! diagnostics without stopping on the first error, so the user sees
//! every problem at once.
//!
//! Checks include WCAG contrast ratios, type-scale consistency,
//! spacing grid alignment, font fallbacks, minimum body size, and
//! sRGB gamut warnings.

use crate::color::{contrast_ratio, is_in_srgb_gamut};
use crate::issue::{Issue, Severity};
use crate::types::{ContrastLevel, TokenInput};

/// Validate a parsed token input and return all diagnostic issues.
///
/// Returns an empty vec if the token set is fully valid.
///
/// # Checks performed
///
/// 1. **Contrast** — every text color against every surface color (WCAG)
/// 2. **Type scale** — ratio and base size sanity
/// 3. **Spacing grid** — every value aligned to the grid base
/// 4. **Font fallbacks** — every font stack ends with a generic family
/// 5. **Minimum body size** — base size meets the configured minimum
/// 6. **Gamut** — warn on out-of-sRGB colors
#[must_use]
pub fn validate(input: &TokenInput) -> Vec<Issue> {
    let mut issues = Vec::new();

    validate_contrast(input, &mut issues);
    validate_type_scale(input, &mut issues);
    validate_spacing_grid(input, &mut issues);
    validate_font_fallbacks(input, &mut issues);
    validate_min_body_size(input, &mut issues);
    validate_gamut(input, &mut issues);

    issues
}

/// Returns `true` if the issue list contains any errors.
#[must_use]
pub fn has_errors(issues: &[Issue]) -> bool {
    issues.iter().any(|i| i.severity == Severity::Error)
}

// ---------------------------------------------------------------------------
// Contrast
// ---------------------------------------------------------------------------

/// Check WCAG contrast between text-like colors and surface-like colors.
///
/// Uses naming heuristics: color names containing "text", "on-", or
/// "foreground" are foreground; names containing "surface", "background",
/// or "bg" are background. Both light and dark modes are checked.
fn validate_contrast(input: &TokenInput, issues: &mut Vec<Issue>) {
    let required_ratio = input.validation.contrast_level.normal_text_ratio();
    let level_name = match input.validation.contrast_level {
        ContrastLevel::Aa => "WCAG AA",
        ContrastLevel::Aaa => "WCAG AAA",
    };

    let text_colors: Vec<_> = input
        .colors
        .iter()
        .filter(|(name, _)| is_text_color_name(name))
        .collect();

    let surface_colors: Vec<_> = input
        .colors
        .iter()
        .filter(|(name, _)| is_surface_color_name(name))
        .collect();

    for (fg_name, fg_pair) in &text_colors {
        for (bg_name, bg_pair) in &surface_colors {
            // Check light mode
            let light_ratio = contrast_ratio(&fg_pair.light, &bg_pair.light);
            if light_ratio < required_ratio {
                issues.push(
                    Issue::error(
                        "contrast-fail",
                        format!("colors.{fg_name} / colors.{bg_name}"),
                        format!(
                            "Light mode contrast {light_ratio:.2}:1 is below {required_ratio}:1 ({level_name})",
                        ),
                    )
                    .with_suggestion(format!(
                        "Increase lightness difference between '{fg_name}' and '{bg_name}' in light mode"
                    )),
                );
            }

            // Check dark mode
            let dark_ratio = contrast_ratio(&fg_pair.dark, &bg_pair.dark);
            if dark_ratio < required_ratio {
                issues.push(
                    Issue::error(
                        "contrast-fail",
                        format!("colors.{fg_name} / colors.{bg_name}"),
                        format!(
                            "Dark mode contrast {dark_ratio:.2}:1 is below {required_ratio}:1 ({level_name})",
                        ),
                    )
                    .with_suggestion(format!(
                        "Increase lightness difference between '{fg_name}' and '{bg_name}' in dark mode"
                    )),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Type scale
// ---------------------------------------------------------------------------

/// Validate typography settings — scale ratio and base size sanity.
fn validate_type_scale(input: &TokenInput, issues: &mut Vec<Issue>) {
    let ratio = input.typography.scale_ratio;

    if ratio <= 1.0 {
        issues.push(
            Issue::error(
                "invalid-scale-ratio",
                "typography.scale_ratio",
                format!("Scale ratio {ratio} must be greater than 1.0"),
            )
            .with_suggestion(
                "Common ratios: 1.125 (Major Second), 1.2 (Minor Third), \
                 1.25 (Major Third), 1.333 (Perfect Fourth)",
            ),
        );
    } else if ratio > 2.0 {
        issues.push(
            Issue::warning(
                "extreme-scale-ratio",
                "typography.scale_ratio",
                format!("Scale ratio {ratio} is unusually large — heading sizes may be extreme"),
            )
            .with_suggestion("Most design systems use a ratio between 1.125 and 1.5"),
        );
    }

    let base = input.typography.base_size;
    if base <= 0.0 {
        issues.push(Issue::error(
            "invalid-base-size",
            "typography.base_size",
            format!("Base size {base}px must be positive"),
        ));
    }
}

// ---------------------------------------------------------------------------
// Spacing grid
// ---------------------------------------------------------------------------

/// Validate that every spacing value is a multiple of the grid base.
fn validate_spacing_grid(input: &TokenInput, issues: &mut Vec<Issue>) {
    let grid = input.validation.spacing_grid;
    if grid <= 0.0 {
        return;
    }

    let base = input.spacing.base;
    for &multiplier in &input.spacing.scale {
        let px = base * multiplier;
        let remainder = px % grid;
        // Allow small floating-point imprecision
        if remainder > 0.001 && (grid - remainder) > 0.001 {
            issues.push(
                Issue::warning(
                    "grid-misalignment",
                    format!("spacing.scale[{multiplier}]"),
                    format!(
                        "Spacing value {px}px is not a multiple of the {grid}px grid"
                    ),
                )
                .with_suggestion(format!(
                    "Adjust base ({base}) or multiplier ({multiplier}) so the product is a multiple of {grid}"
                )),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Font fallbacks
// ---------------------------------------------------------------------------

/// Check that font stacks include a generic family as fallback.
fn validate_font_fallbacks(input: &TokenInput, issues: &mut Vec<Issue>) {
    let generics = [
        "serif",
        "sans-serif",
        "monospace",
        "cursive",
        "fantasy",
        "system-ui",
        "ui-serif",
        "ui-sans-serif",
        "ui-monospace",
        "ui-rounded",
        "math",
        "emoji",
        "fangsong",
    ];

    let stacks = [
        ("fonts.display", &input.fonts.display),
        ("fonts.body", &input.fonts.body),
        ("fonts.mono", &input.fonts.mono),
    ];

    let mut total_families = 0usize;

    for (location, stack) in stacks {
        let families: Vec<&str> = stack.split(',').map(str::trim).collect();
        total_families += families.len();

        let has_generic = families
            .iter()
            .any(|f| generics.contains(&f.trim_matches('"').trim_matches('\'')));

        if !has_generic {
            issues.push(
                Issue::warning(
                    "missing-fallback",
                    location,
                    format!("Font stack \"{stack}\" has no generic family fallback"),
                )
                .with_suggestion("Add a generic family at the end (e.g. sans-serif, monospace)"),
            );
        }
    }

    if total_families > input.validation.max_font_families {
        issues.push(
            Issue::warning(
                "too-many-fonts",
                "fonts",
                format!(
                    "Total font families ({total_families}) exceeds maximum ({})",
                    input.validation.max_font_families,
                ),
            )
            .with_suggestion("Reduce font families to improve page load performance"),
        );
    }
}

// ---------------------------------------------------------------------------
// Minimum body size
// ---------------------------------------------------------------------------

/// Check that the base font size meets the configured minimum.
fn validate_min_body_size(input: &TokenInput, issues: &mut Vec<Issue>) {
    let base = input.typography.base_size;
    let min = input.validation.min_body_size;

    if base < min {
        issues.push(
            Issue::error(
                "body-too-small",
                "typography.base_size",
                format!("Base size {base}px is below minimum {min}px"),
            )
            .with_suggestion(format!("Set base_size to at least {min}")),
        );
    }
}

// ---------------------------------------------------------------------------
// sRGB gamut
// ---------------------------------------------------------------------------

/// Warn about colors that are outside the sRGB gamut.
fn validate_gamut(input: &TokenInput, issues: &mut Vec<Issue>) {
    for (name, pair) in &input.colors {
        if !is_in_srgb_gamut(&pair.light) {
            issues.push(
                Issue::warning(
                    "out-of-gamut",
                    format!("colors.{name}.light"),
                    format!(
                        "Light color oklch({} {} {}) is outside the sRGB gamut",
                        pair.light.l, pair.light.c, pair.light.h,
                    ),
                )
                .with_suggestion(
                    "Reduce chroma to bring the color into sRGB, or accept \
                     that it will be clamped on non-P3 displays",
                ),
            );
        }
        if !is_in_srgb_gamut(&pair.dark) {
            issues.push(
                Issue::warning(
                    "out-of-gamut",
                    format!("colors.{name}.dark"),
                    format!(
                        "Dark color oklch({} {} {}) is outside the sRGB gamut",
                        pair.dark.l, pair.dark.c, pair.dark.h,
                    ),
                )
                .with_suggestion(
                    "Reduce chroma to bring the color into sRGB, or accept \
                     that it will be clamped on non-P3 displays",
                ),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Name heuristics
// ---------------------------------------------------------------------------

/// Heuristic: is this color name likely a foreground/text color?
fn is_text_color_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("text")
        || lower.contains("foreground")
        || lower.starts_with("on-")
        || lower.starts_with("on_")
}

/// Heuristic: is this color name likely a background/surface color?
fn is_surface_color_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("surface") || lower.contains("background") || lower.contains("bg")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::types::{ColorPair, ContrastLevel, OklchColor, ValidationConfig};

    fn input_with_colors(colors: Vec<(&str, ColorPair)>) -> TokenInput {
        let mut map = BTreeMap::new();
        for (name, pair) in colors {
            map.insert(name.to_owned(), pair);
        }
        TokenInput {
            colors: map,
            ..TokenInput::default()
        }
    }

    #[test]
    fn passing_contrast() {
        let input = input_with_colors(vec![
            (
                "text",
                ColorPair {
                    light: OklchColor::new(0.0, 0.0, 0.0), // black
                    dark: OklchColor::new(1.0, 0.0, 0.0),  // white
                },
            ),
            (
                "surface",
                ColorPair {
                    light: OklchColor::new(1.0, 0.0, 0.0), // white
                    dark: OklchColor::new(0.0, 0.0, 0.0),  // black
                },
            ),
        ]);
        let issues = validate(&input);
        let contrast_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.code == "contrast-fail")
            .collect();
        assert!(
            contrast_errors.is_empty(),
            "expected no contrast errors, got: {contrast_errors:?}"
        );
    }

    #[test]
    fn failing_contrast() {
        let input = input_with_colors(vec![
            (
                "text",
                ColorPair {
                    light: OklchColor::new(0.6, 0.0, 0.0),
                    dark: OklchColor::new(0.6, 0.0, 0.0),
                },
            ),
            (
                "surface",
                ColorPair {
                    light: OklchColor::new(0.7, 0.0, 0.0),
                    dark: OklchColor::new(0.7, 0.0, 0.0),
                },
            ),
        ]);
        let issues = validate(&input);
        let contrast_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.code == "contrast-fail")
            .collect();
        assert!(
            !contrast_errors.is_empty(),
            "expected contrast errors for similar greys"
        );
    }

    #[test]
    fn invalid_scale_ratio() {
        let mut input = TokenInput::default();
        input.typography.scale_ratio = 0.8;
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "invalid-scale-ratio"));
    }

    #[test]
    fn extreme_scale_ratio_warns() {
        let mut input = TokenInput::default();
        input.typography.scale_ratio = 2.5;
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "extreme-scale-ratio"));
    }

    #[test]
    fn body_too_small() {
        let mut input = TokenInput::default();
        input.typography.base_size = 10.0;
        input.validation.min_body_size = 14.0;
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "body-too-small"));
    }

    #[test]
    fn font_missing_fallback() {
        let mut input = TokenInput::default();
        input.fonts.display = "CustomFont".to_owned();
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "missing-fallback"));
    }

    #[test]
    fn font_with_generic_passes() {
        let input = TokenInput::default(); // defaults have system-ui
        let issues = validate(&input);
        let fallback_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code == "missing-fallback")
            .collect();
        assert!(fallback_issues.is_empty());
    }

    #[test]
    fn out_of_gamut_warning() {
        let input = input_with_colors(vec![(
            "vivid",
            ColorPair {
                light: OklchColor::new(0.5, 0.4, 264.0),
                dark: OklchColor::new(0.5, 0.4, 264.0),
            },
        )]);
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "out-of-gamut"));
    }

    #[test]
    fn has_errors_function() {
        let issues = vec![
            Issue::warning("test", "loc", "msg"),
            Issue::info("test", "loc", "msg"),
        ];
        assert!(!has_errors(&issues));

        let issues_with_error = vec![Issue::error("test", "loc", "msg")];
        assert!(has_errors(&issues_with_error));
    }

    #[test]
    fn aaa_requires_higher_contrast() {
        let mut input = input_with_colors(vec![
            (
                "text",
                ColorPair {
                    light: OklchColor::new(0.0, 0.0, 0.0),
                    dark: OklchColor::new(1.0, 0.0, 0.0),
                },
            ),
            (
                "surface",
                ColorPair {
                    light: OklchColor::new(1.0, 0.0, 0.0),
                    dark: OklchColor::new(0.0, 0.0, 0.0),
                },
            ),
        ]);
        input.validation = ValidationConfig {
            contrast_level: ContrastLevel::Aaa,
            ..ValidationConfig::default()
        };
        let issues = validate(&input);
        // Black on white is ~21:1 which passes AAA
        let contrast_errors: Vec<_> = issues
            .iter()
            .filter(|i| i.code == "contrast-fail")
            .collect();
        assert!(contrast_errors.is_empty());
    }

    #[test]
    fn too_many_fonts_warns() {
        let mut input = TokenInput::default();
        input.fonts.display = "A, B, C, D, E, sans-serif".to_owned();
        input.fonts.body = "F, G, H, I, J, sans-serif".to_owned();
        input.fonts.mono = "K, L, monospace".to_owned();
        input.validation.max_font_families = 5;
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "too-many-fonts"));
    }

    #[test]
    fn spacing_grid_misalignment() {
        let mut input = TokenInput::default();
        input.spacing.base = 5.0; // 5px base with scale [1] = 5px, not multiple of 4px grid
        input.spacing.scale = vec![1.0];
        input.validation.spacing_grid = 4.0;
        let issues = validate(&input);
        assert!(issues.iter().any(|i| i.code == "grid-misalignment"));
    }
}
