//! Spacing scale resolver.
//!
//! Converts a base unit and list of scale multipliers into concrete
//! pixel and rem spacing values.

use crate::types::{ResolvedSpacing, SpacingConfig};

/// The 16px browser default used for px → rem conversion.
const PX_PER_REM: f64 = 16.0;

/// Resolve spacing configuration into named spacing tokens.
///
/// Each entry in `config.scale` is multiplied by `config.base` to produce
/// a pixel value, which is then converted to rem (÷ 16).
///
/// The token name is the scale multiplier (e.g. `"1"`, `"2"`, `"6"`).
#[must_use]
pub fn resolve_spacing(config: &SpacingConfig) -> Vec<ResolvedSpacing> {
    config
        .scale
        .iter()
        .map(|&multiplier| {
            let px = config.base * multiplier;
            let rem = px / PX_PER_REM;

            ResolvedSpacing {
                name: format_multiplier(multiplier),
                value_rem: rem,
                value_px: px,
            }
        })
        .collect()
}

/// Format a scale multiplier as a clean name (no trailing `.0`).
fn format_multiplier(v: f64) -> String {
    if (v - v.round()).abs() < f64::EPSILON {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "scale multipliers are small positive integers, truncation is safe"
        )]
        let int = v as i64;
        format!("{int}")
    } else {
        format!("{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_spacing_produces_9_values() {
        let config = SpacingConfig::default();
        let result = resolve_spacing(&config);
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn base_4_scale_1_is_4px() {
        let config = SpacingConfig {
            base: 4.0,
            scale: vec![1.0],
        };
        let result = resolve_spacing(&config);
        assert_eq!(result.len(), 1);
        assert!((result[0].value_px - 4.0).abs() < f64::EPSILON);
        assert!((result[0].value_rem - 0.25).abs() < f64::EPSILON);
        assert_eq!(result[0].name, "1");
    }

    #[test]
    fn base_8_scale_2_is_16px_1rem() {
        let config = SpacingConfig {
            base: 8.0,
            scale: vec![2.0],
        };
        let result = resolve_spacing(&config);
        assert!((result[0].value_px - 16.0).abs() < f64::EPSILON);
        assert!((result[0].value_rem - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn multiplier_format_integers() {
        assert_eq!(format_multiplier(1.0), "1");
        assert_eq!(format_multiplier(16.0), "16");
    }

    #[test]
    fn multiplier_format_fractional() {
        assert_eq!(format_multiplier(1.5), "1.5");
    }
}
