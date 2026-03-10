//! Token resolver — computes derived values from raw input.
//!
//! Takes a validated [`TokenInput`] and produces [`ResolvedTokens`]
//! with all CSS-ready values fully computed. This is the main pipeline
//! transform: parse → validate → **resolve** → output.

use crate::spacing::resolve_spacing;
use crate::types::{
    ResolvedColor, ResolvedFonts, ResolvedMotion, ResolvedRadius, ResolvedTokens, TokenInput,
};
use crate::typography::resolve_typography;

/// Resolve a [`TokenInput`] into fully computed [`ResolvedTokens`].
///
/// All derived values — CSS oklch strings, rem conversions, fluid clamp
/// expressions, typography roles — are computed and ready for output
/// generation.
#[must_use]
pub fn resolve(input: &TokenInput) -> ResolvedTokens {
    ResolvedTokens {
        fonts: resolve_fonts(input),
        colors: resolve_colors(input),
        typography: resolve_typography(&input.typography, &input.fonts),
        spacing: resolve_spacing(&input.spacing),
        radius: resolve_radius(input),
        motion: resolve_motion(input),
    }
}

/// Resolve font stacks — direct passthrough from input.
fn resolve_fonts(input: &TokenInput) -> ResolvedFonts {
    ResolvedFonts {
        display: input.fonts.display.clone(),
        body: input.fonts.body.clone(),
        mono: input.fonts.mono.clone(),
    }
}

/// Resolve colors — generate CSS `oklch()` strings for light and dark.
fn resolve_colors(input: &TokenInput) -> Vec<ResolvedColor> {
    input
        .colors
        .iter()
        .map(|(name, pair)| ResolvedColor {
            name: name.clone(),
            css_var: format!("--color-{name}"),
            light: pair.light,
            dark: pair.dark,
            light_css: pair.light.to_css(),
            dark_css: pair.dark.to_css(),
        })
        .collect()
}

/// Resolve radius — pass through user-defined values.
fn resolve_radius(input: &TokenInput) -> Vec<ResolvedRadius> {
    input
        .radius
        .iter()
        .map(|(name, value)| ResolvedRadius {
            name: name.clone(),
            value: value.clone(),
        })
        .collect()
}

/// Resolve motion — pass through durations and easings.
fn resolve_motion(input: &TokenInput) -> ResolvedMotion {
    ResolvedMotion {
        durations: input.motion.durations.clone(),
        easings: input.motion.easings.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ColorPair, OklchColor};

    #[test]
    fn resolve_default_produces_13_type_roles() {
        let input = TokenInput::default();
        let resolved = resolve(&input);
        assert_eq!(resolved.typography.len(), 13);
    }

    #[test]
    fn resolve_default_spacing() {
        let input = TokenInput::default();
        let resolved = resolve(&input);
        assert!(!resolved.spacing.is_empty());
        // First entry is multiplier 1 → 4px → 0.25rem
        assert_eq!(resolved.spacing[0].name, "1");
        assert!((resolved.spacing[0].value_px - 4.0).abs() < 0.001);
    }

    #[test]
    fn resolve_colors_generates_css_vars() {
        let mut input = TokenInput::default();
        input.colors.insert(
            "primary".to_owned(),
            ColorPair {
                light: OklchColor::new(0.55, 0.25, 264.0),
                dark: OklchColor::new(0.65, 0.20, 264.0),
            },
        );
        let resolved = resolve(&input);
        assert_eq!(resolved.colors.len(), 1);
        assert_eq!(resolved.colors[0].css_var, "--color-primary");
        assert!(resolved.colors[0].light_css.starts_with("oklch("));
        assert!(resolved.colors[0].dark_css.starts_with("oklch("));
    }

    #[test]
    fn resolve_fonts_passthrough() {
        let mut input = TokenInput::default();
        input.fonts.display = "CustomDisplay, serif".to_owned();
        let resolved = resolve(&input);
        assert_eq!(resolved.fonts.display, "CustomDisplay, serif");
    }

    #[test]
    fn resolve_radius_passthrough() {
        let mut input = TokenInput::default();
        input.radius.insert("sm".to_owned(), "0.25rem".to_owned());
        input.radius.insert("full".to_owned(), "9999px".to_owned());
        let resolved = resolve(&input);
        assert_eq!(resolved.radius.len(), 2);
    }

    #[test]
    fn resolve_motion_passthrough() {
        let input = TokenInput::default();
        let resolved = resolve(&input);
        assert!(!resolved.motion.durations.is_empty());
        assert!(!resolved.motion.easings.is_empty());
    }

    #[test]
    fn resolve_body_md_is_base_size() {
        let input = TokenInput::default();
        let resolved = resolve(&input);
        let body_md = resolved
            .typography
            .iter()
            .find(|r| r.name == "body-md")
            .expect("body-md not found");
        // Default base_size is 16px = 1.0rem
        let font_size = &body_md.css_properties["font-size"];
        assert!(
            font_size.contains("1.0") || font_size.contains("1rem"),
            "body-md font-size should be ~1rem, got: {font_size}"
        );
    }
}
