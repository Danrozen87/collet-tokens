//! All token types — raw input, resolved output, and intermediate structures.
//!
//! The token pipeline flows: YAML/JSON string → [`TokenInput`] → [`ResolvedTokens`].
//! Input types represent what the user writes. Resolved types represent what the
//! compiler outputs, with all derived values fully computed.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Input types (what the user writes in YAML/JSON)
// ---------------------------------------------------------------------------

/// Raw parsed token input from a YAML or JSON file.
///
/// Every section is optional — the parser fills in sensible defaults for
/// anything the user omits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenInput {
    /// Font family stacks.
    pub fonts: FontConfig,
    /// Named color pairs (light + dark mode).
    pub colors: BTreeMap<String, ColorPair>,
    /// Typography scale configuration.
    pub typography: TypographyConfig,
    /// Spacing scale configuration.
    pub spacing: SpacingConfig,
    /// Border radius presets.
    pub radius: BTreeMap<String, String>,
    /// Motion (duration + easing) configuration.
    pub motion: MotionConfig,
    /// Validation strictness settings.
    pub validation: ValidationConfig,
}

/// A color defined in oklch for both light and dark modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPair {
    /// oklch color for light mode.
    pub light: OklchColor,
    /// oklch color for dark mode.
    pub dark: OklchColor,
}

/// A single color in the oklch perceptual color space.
///
/// - `l`: lightness, 0.0 (black) to 1.0 (white)
/// - `c`: chroma, 0.0 (grey) to ~0.4 (vivid)
/// - `h`: hue angle in degrees, 0.0 to 360.0
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OklchColor {
    /// Lightness (0.0–1.0).
    pub l: f64,
    /// Chroma (0.0–~0.4).
    pub c: f64,
    /// Hue angle in degrees (0.0–360.0).
    pub h: f64,
}

/// Font family stacks for each typographic role.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    /// Display / heading font stack (e.g. `"Inter, system-ui, sans-serif"`).
    pub display: String,
    /// Body / paragraph font stack.
    pub body: String,
    /// Monospace / code font stack.
    pub mono: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            display: "system-ui, sans-serif".to_owned(),
            body: "system-ui, sans-serif".to_owned(),
            mono: "ui-monospace, monospace".to_owned(),
        }
    }
}

/// Typography scale configuration.
///
/// The scale generates 13 type roles from `base_size` and `scale_ratio`
/// using a modular-scale approach.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TypographyConfig {
    /// The modular-scale ratio (e.g. 1.25 for Major Third).
    pub scale_ratio: f64,
    /// Base font size in pixels.
    pub base_size: f64,
    /// Whether headings use `clamp()` fluid sizing.
    pub fluid_headings: bool,
    /// Optional per-role overrides (role name → CSS property map).
    pub roles: BTreeMap<String, BTreeMap<String, String>>,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            scale_ratio: 1.25,
            base_size: 16.0,
            fluid_headings: false,
            roles: BTreeMap::new(),
        }
    }
}

/// Spacing scale configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpacingConfig {
    /// Base spacing unit in pixels (e.g. 4).
    pub base: f64,
    /// Scale multipliers (e.g. `[1, 2, 3, 4, 6, 8, 10, 12, 16]`).
    pub scale: Vec<f64>,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            base: 4.0,
            scale: vec![1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0, 12.0, 16.0],
        }
    }
}

/// Motion (animation timing) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MotionConfig {
    /// Named durations (e.g. `"fast" → "100ms"`).
    pub durations: BTreeMap<String, String>,
    /// Named easings (e.g. `"spring" → "cubic-bezier(0.34, 1.56, 0.64, 1)"`).
    pub easings: BTreeMap<String, String>,
}

impl Default for MotionConfig {
    fn default() -> Self {
        let mut durations = BTreeMap::new();
        durations.insert("instant".to_owned(), "50ms".to_owned());
        durations.insert("fast".to_owned(), "100ms".to_owned());
        durations.insert("normal".to_owned(), "200ms".to_owned());
        durations.insert("slow".to_owned(), "400ms".to_owned());
        durations.insert("slower".to_owned(), "600ms".to_owned());

        let mut easings = BTreeMap::new();
        easings.insert("linear".to_owned(), "linear".to_owned());
        easings.insert(
            "ease-out".to_owned(),
            "cubic-bezier(0.16, 1, 0.3, 1)".to_owned(),
        );
        easings.insert(
            "ease-in".to_owned(),
            "cubic-bezier(0.7, 0, 0.84, 0)".to_owned(),
        );
        easings.insert(
            "ease-in-out".to_owned(),
            "cubic-bezier(0.45, 0, 0.55, 1)".to_owned(),
        );
        easings.insert(
            "spring".to_owned(),
            "cubic-bezier(0.34, 1.56, 0.64, 1)".to_owned(),
        );

        Self { durations, easings }
    }
}

/// Validation strictness configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationConfig {
    /// WCAG contrast level: `"AA"` (4.5:1 for normal text) or `"AAA"` (7:1).
    pub contrast_level: ContrastLevel,
    /// Minimum body text size in pixels.
    pub min_body_size: f64,
    /// Maximum number of font families across all stacks.
    pub max_font_families: usize,
    /// Spacing grid base — all spacing values must be multiples of this.
    pub spacing_grid: f64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            contrast_level: ContrastLevel::Aa,
            min_body_size: 14.0,
            max_font_families: 12,
            spacing_grid: 4.0,
        }
    }
}

/// WCAG contrast compliance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContrastLevel {
    /// WCAG 2.x AA — 4.5:1 for normal text, 3:1 for large text.
    #[serde(rename = "AA")]
    Aa,
    /// WCAG 2.x AAA — 7:1 for normal text, 4.5:1 for large text.
    #[serde(rename = "AAA")]
    Aaa,
}

impl ContrastLevel {
    /// Minimum contrast ratio required for normal-sized text.
    #[must_use]
    pub fn normal_text_ratio(self) -> f64 {
        match self {
            Self::Aa => 4.5,
            Self::Aaa => 7.0,
        }
    }

    /// Minimum contrast ratio required for large text (≥18pt or ≥14pt bold).
    #[must_use]
    pub fn large_text_ratio(self) -> f64 {
        match self {
            Self::Aa => 3.0,
            Self::Aaa => 4.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Resolved types (what the compiler outputs)
// ---------------------------------------------------------------------------

/// Fully resolved token set, ready for output generation.
///
/// All derived values (CSS strings, rem conversions, fluid clamp expressions)
/// have been computed. Output crates consume this to emit CSS, Tailwind config,
/// JSON, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTokens {
    /// Resolved font stacks.
    pub fonts: ResolvedFonts,
    /// Resolved color tokens with CSS strings.
    pub colors: Vec<ResolvedColor>,
    /// Resolved typography roles with computed CSS properties.
    pub typography: Vec<ResolvedTypeRole>,
    /// Resolved spacing scale in rem + px.
    pub spacing: Vec<ResolvedSpacing>,
    /// Resolved border radius presets.
    pub radius: Vec<ResolvedRadius>,
    /// Resolved motion tokens.
    pub motion: ResolvedMotion,
}

/// A resolved color with precomputed CSS output strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedColor {
    /// Token name (e.g. `"primary"`, `"surface"`).
    pub name: String,
    /// CSS custom property name (e.g. `"--color-primary"`).
    pub css_var: String,
    /// Light-mode oklch color.
    pub light: OklchColor,
    /// Dark-mode oklch color.
    pub dark: OklchColor,
    /// Light-mode CSS value string (e.g. `"oklch(0.55 0.25 264)"`).
    pub light_css: String,
    /// Dark-mode CSS value string.
    pub dark_css: String,
}

/// A resolved typography role with all CSS properties computed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTypeRole {
    /// Role name (e.g. `"display"`, `"h1"`, `"body-md"`).
    pub name: String,
    /// Computed CSS properties (font-family, font-size, font-weight,
    /// line-height, letter-spacing).
    pub css_properties: BTreeMap<String, String>,
    /// Fluid `clamp()` value for font-size, if fluid headings are enabled.
    pub fluid_size: Option<String>,
}

/// A resolved spacing value in both rem and px.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedSpacing {
    /// Token name (e.g. `"1"`, `"2"`, `"4"`).
    pub name: String,
    /// Value in rem.
    pub value_rem: f64,
    /// Value in pixels.
    pub value_px: f64,
}

/// A resolved border radius preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRadius {
    /// Token name (e.g. `"sm"`, `"md"`, `"full"`).
    pub name: String,
    /// CSS value (e.g. `"0.25rem"`, `"9999px"`).
    pub value: String,
}

/// Resolved font stacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFonts {
    /// Display / heading font stack.
    pub display: String,
    /// Body / paragraph font stack.
    pub body: String,
    /// Monospace / code font stack.
    pub mono: String,
}

/// Resolved motion tokens (durations + easings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedMotion {
    /// Named durations (e.g. `"fast" → "100ms"`).
    pub durations: BTreeMap<String, String>,
    /// Named easings (e.g. `"spring" → "cubic-bezier(…)"`).
    pub easings: BTreeMap<String, String>,
}
