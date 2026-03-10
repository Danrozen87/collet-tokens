//! oklch color space operations — CSS output, sRGB conversion, and WCAG contrast.
//!
//! Implements the full oklch → oklab → linear sRGB → sRGB pipeline and the
//! WCAG 2.x relative-luminance contrast ratio formula.

use crate::types::OklchColor;

impl OklchColor {
    /// Create a new oklch color.
    #[must_use]
    pub const fn new(l: f64, c: f64, h: f64) -> Self {
        Self { l, c, h }
    }

    /// Render as a CSS `oklch()` function value.
    ///
    /// # Example
    /// ```
    /// use collet_tokens_core::types::OklchColor;
    /// let color = OklchColor::new(0.55, 0.25, 264.0);
    /// assert_eq!(color.to_css(), "oklch(0.55 0.25 264)");
    /// ```
    #[must_use]
    pub fn to_css(&self) -> String {
        // Format without trailing zeros for clean CSS output.
        let l = format_f64(self.l);
        let c = format_f64(self.c);
        let h = format_f64(self.h);
        format!("oklch({l} {c} {h})")
    }

    /// Approximate sRGB hex color string (e.g. `"#3a5bc7"`).
    ///
    /// Out-of-gamut colors are clamped to the sRGB cube.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let (r, g, b) = oklch_to_srgb(self);
        let ri = float_to_u8(r);
        let gi = float_to_u8(g);
        let bi = float_to_u8(b);
        format!("#{ri:02x}{gi:02x}{bi:02x}")
    }
}

/// Format an f64 without unnecessary trailing zeros.
fn format_f64(v: f64) -> String {
    // Use enough precision to round-trip, then strip trailing zeros.
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_owned()
}

/// Clamp an f64 in 0.0..=1.0 to a u8 in 0..=255.
fn float_to_u8(v: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "value is clamped to 0.0..=1.0, so truncation and sign are safe"
    )]
    let byte = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    byte
}

// ---------------------------------------------------------------------------
// oklch → sRGB conversion pipeline
// ---------------------------------------------------------------------------

/// Convert an oklch color to gamma-encoded sRGB (each channel 0.0–1.0).
///
/// Out-of-gamut values are clamped to 0.0–1.0.
///
/// Pipeline: oklch → oklab → linear RGB (via LMS) → sRGB gamma.
#[must_use]
#[expect(
    clippy::many_single_char_names,
    reason = "l/m/s and r/g/b are standard color-science names"
)]
pub fn oklch_to_srgb(color: &OklchColor) -> (f64, f64, f64) {
    // Step 1: oklch → oklab
    let h_rad = color.h.to_radians();
    let ok_l = color.l;
    let ok_a = color.c * h_rad.cos();
    let ok_b = color.c * h_rad.sin();

    // Step 2: oklab → LMS (cube-root domain)
    // Inverse of the oklab → LMS matrix from Björn Ottosson's spec.
    let l_ = ok_l + 0.396_337_792_3 * ok_a + 0.215_803_758_2 * ok_b;
    let m_ = ok_l - 0.105_561_346_2 * ok_a - 0.063_854_174_77 * ok_b;
    let s_ = ok_l - 0.089_484_182_09 * ok_a - 1.291_485_548 * ok_b;

    // Step 3: undo cube root — LMS cube-root → linear LMS
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // Step 4: linear LMS → linear sRGB via the standard matrix
    let r_lin = 4.076_741_662 * l - 3.307_711_591 * m + 0.230_969_929 * s;
    let g_lin = -1.268_438_005 * l + 2.609_757_401 * m - 0.341_319_396 * s;
    let b_lin = -0.004_196_086_3 * l - 0.703_418_615 * m + 1.707_614_701 * s;

    // Step 5: linear sRGB → gamma-encoded sRGB
    let r = linear_to_srgb_gamma(r_lin);
    let g = linear_to_srgb_gamma(g_lin);
    let b = linear_to_srgb_gamma(b_lin);

    // Clamp to gamut
    (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

/// Apply the sRGB gamma transfer function to a linear-light value.
fn linear_to_srgb_gamma(x: f64) -> f64 {
    if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

/// Remove sRGB gamma to get a linear-light value (inverse of the gamma TF).
fn srgb_gamma_to_linear(x: f64) -> f64 {
    if x <= 0.040_45 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

// ---------------------------------------------------------------------------
// WCAG 2.x contrast
// ---------------------------------------------------------------------------

/// Compute relative luminance of a gamma-encoded sRGB color.
///
/// Input channels are 0.0–1.0 (gamma-encoded). The function linearises them
/// before applying the luminance weights.
///
/// Formula: `L = 0.2126 * R_lin + 0.7152 * G_lin + 0.0722 * B_lin`
#[must_use]
pub fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    let r_lin = srgb_gamma_to_linear(r);
    let g_lin = srgb_gamma_to_linear(g);
    let b_lin = srgb_gamma_to_linear(b);
    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

/// Compute the WCAG 2.x contrast ratio between two oklch colors.
///
/// Returns a value ≥ 1.0. Higher means more contrast.
///
/// Formula: `(L1 + 0.05) / (L2 + 0.05)` where `L1 ≥ L2`.
#[must_use]
pub fn contrast_ratio(fg: &OklchColor, bg: &OklchColor) -> f64 {
    let (fr, fg_g, fb) = oklch_to_srgb(fg);
    let (br, bg_g, bb) = oklch_to_srgb(bg);

    let l1 = relative_luminance(fr, fg_g, fb);
    let l2 = relative_luminance(br, bg_g, bb);

    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Check whether an oklch color is representable within the sRGB gamut.
///
/// A color is considered in-gamut if all sRGB channels (before clamping)
/// fall within -0.001..=1.001 (small epsilon for floating-point imprecision).
#[must_use]
#[expect(
    clippy::many_single_char_names,
    reason = "l/m/s and r/g/b are standard color-science names"
)]
pub fn is_in_srgb_gamut(color: &OklchColor) -> bool {
    const EPS: f64 = 0.001;

    let h_rad = color.h.to_radians();
    let ok_l = color.l;
    let ok_a = color.c * h_rad.cos();
    let ok_b = color.c * h_rad.sin();

    let l_ = ok_l + 0.396_337_792_3 * ok_a + 0.215_803_758_2 * ok_b;
    let m_ = ok_l - 0.105_561_346_2 * ok_a - 0.063_854_174_77 * ok_b;
    let s_ = ok_l - 0.089_484_182_09 * ok_a - 1.291_485_548 * ok_b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r_lin = 4.076_741_662 * l - 3.307_711_591 * m + 0.230_969_929 * s;
    let g_lin = -1.268_438_005 * l + 2.609_757_401 * m - 0.341_319_396 * s;
    let b_lin = -0.004_196_086_3 * l - 0.703_418_615 * m + 1.707_614_701 * s;

    let r = linear_to_srgb_gamma(r_lin);
    let g = linear_to_srgb_gamma(g_lin);
    let b = linear_to_srgb_gamma(b_lin);

    (-EPS..=1.0 + EPS).contains(&r)
        && (-EPS..=1.0 + EPS).contains(&g)
        && (-EPS..=1.0 + EPS).contains(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_to_css() {
        let white = OklchColor::new(1.0, 0.0, 0.0);
        assert_eq!(white.to_css(), "oklch(1 0 0)");
    }

    #[test]
    fn black_to_hex() {
        let black = OklchColor::new(0.0, 0.0, 0.0);
        assert_eq!(black.to_hex(), "#000000");
    }

    #[test]
    fn white_to_hex() {
        let white = OklchColor::new(1.0, 0.0, 0.0);
        assert_eq!(white.to_hex(), "#ffffff");
    }

    #[test]
    fn contrast_black_white() {
        let black = OklchColor::new(0.0, 0.0, 0.0);
        let white = OklchColor::new(1.0, 0.0, 0.0);
        let ratio = contrast_ratio(&black, &white);
        // WCAG black-on-white is 21:1
        assert!((ratio - 21.0).abs() < 0.5, "expected ~21:1, got {ratio}");
    }

    #[test]
    fn contrast_same_color_is_one() {
        let c = OklchColor::new(0.5, 0.1, 180.0);
        let ratio = contrast_ratio(&c, &c);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn achromatic_grey_in_gamut() {
        let grey = OklchColor::new(0.5, 0.0, 0.0);
        assert!(is_in_srgb_gamut(&grey));
    }

    #[test]
    fn highly_saturated_out_of_gamut() {
        // Extremely saturated color — likely out of sRGB
        let vivid = OklchColor::new(0.5, 0.4, 264.0);
        assert!(!is_in_srgb_gamut(&vivid));
    }

    #[test]
    fn relative_luminance_black() {
        assert!((relative_luminance(0.0, 0.0, 0.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn relative_luminance_white() {
        let lum = relative_luminance(1.0, 1.0, 1.0);
        assert!((lum - 1.0).abs() < 0.001);
    }

    #[test]
    fn format_f64_strips_trailing_zeros() {
        assert_eq!(format_f64(0.5), "0.5");
        assert_eq!(format_f64(1.0), "1");
        assert_eq!(format_f64(0.123_456), "0.123456");
    }
}
