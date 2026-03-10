---
name: collet-tokens
description: >
  Design token compiler with validation. Use this skill when working on
  token parsing, validation rules, platform output generation, CLI commands,
  or any part of the collet-tokens codebase.
---

# Collet Tokens — Build Skill

## What This Is

A Rust-based design token compiler. Input: YAML/JSON token file. Output: validated, platform-specific code (CSS, Tailwind, iOS, Android, Figma).

## Crate Map

| Crate | Purpose | Dependencies |
|-------|---------|-------------|
| `core` | Parse, validate, resolve tokens | serde, serde_yaml, serde_json |
| `output-css` | CSS custom properties + utilities | core |
| `output-tailwind` | Tailwind v4 theme config | core |
| `output-ios` | Swift UIColor/Font extensions | core |
| `output-android` | Android XML resources | core |
| `output-figma` | Figma Tokens JSON | core |
| `cli` | Binary entry point | core, all outputs, clap, colored |
| `wasm` | Browser/Figma plugin target | core, wasm-bindgen |

## Core Crate Architecture

```
crates/core/src/
  lib.rs          ← pub fn compile(), pub fn validate()
  types.rs        ← Token structs (ColorToken, TypographyConfig, SpacingScale, etc.)
  parser.rs       ← YAML/JSON → TokenInput
  validator.rs    ← Contrast, scale, grid checks → Vec<Issue>
  resolver.rs     ← TokenInput → ResolvedTokens (derived values, dark mode, fluid sizes)
  color.rs        ← oklch math, contrast ratios, gamut mapping
  typography.rs   ← Type scale computation, fluid clamp(), role generation
  spacing.rs      ← Grid alignment, scale validation
  issue.rs        ← Issue struct (severity, location, message, suggestion)
```

## Key Types

```rust
/// Raw parsed input from YAML/JSON
pub struct TokenInput {
    pub fonts: FontConfig,
    pub colors: BTreeMap<String, ColorPair>,
    pub typography: TypographyConfig,
    pub spacing: SpacingConfig,
    pub radius: BTreeMap<String, String>,
    pub motion: MotionConfig,
    pub validation: ValidationConfig,
}

/// A color with light and dark mode values
pub struct ColorPair {
    pub light: OklchColor,
    pub dark: OklchColor,
}

/// oklch color value
pub struct OklchColor {
    pub l: f64,  // lightness 0.0–1.0
    pub c: f64,  // chroma 0.0–0.4
    pub h: f64,  // hue 0–360
}

/// Fully resolved tokens ready for output generation
pub struct ResolvedTokens {
    pub fonts: ResolvedFonts,
    pub colors: Vec<ResolvedColor>,
    pub typography: Vec<ResolvedTypeRole>,
    pub spacing: Vec<ResolvedSpacing>,
    pub radius: Vec<ResolvedRadius>,
    pub motion: ResolvedMotion,
}

/// Validation issue
pub struct Issue {
    pub severity: Severity,     // Error, Warning, Info
    pub code: &'static str,     // e.g., "CONTRAST_FAIL"
    pub location: String,       // e.g., "colors.text-muted"
    pub message: String,        // human-readable
    pub suggestion: Option<String>, // actionable fix
}
```

## Validation Rules

| Code | Severity | Check |
|------|----------|-------|
| `CONTRAST_FAIL` | Error | Text/bg color pair fails WCAG AA (4.5:1) or AAA (7:1) |
| `CONTRAST_LARGE` | Warning | Large text pair fails 3:1 ratio |
| `SCALE_BREAK` | Error | Type scale step deviates >5% from ratio |
| `GRID_VIOLATION` | Warning | Spacing value not aligned to base grid |
| `MIN_BODY_SIZE` | Error | Base body size below configured minimum |
| `FONT_FALLBACK` | Warning | Font stack missing system fallback |
| `GAMUT_CLIP` | Info | oklch color will be clipped in sRGB |
| `DUPLICATE_VALUE` | Warning | Two tokens resolve to identical values |

## Output Format Patterns

Every output crate follows the same interface:

```rust
use collet_tokens_core::ResolvedTokens;

pub struct OutputFile {
    pub path: String,
    pub content: String,
}

pub fn generate(tokens: &ResolvedTokens) -> Vec<OutputFile>;
```

### CSS Output Example

```css
:root {
  /* Colors */
  --color-surface: oklch(1.0 0 0);
  --color-text-primary: oklch(0.27 0.003 90);
  --color-primary: oklch(0.55 0.25 264);

  /* Typography */
  --font-display: 'Plus Jakarta Sans', system-ui, sans-serif;
  --font-body: 'Inter', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;

  /* Spacing */
  --space-1: 0.25rem;
  --space-2: 0.5rem;
  /* ... */

  /* Radius */
  --radius-sm: 0.25rem;
  --radius-md: 0.5rem;
  /* ... */
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-surface: oklch(0.13 0.01 280);
    --color-text-primary: oklch(0.87 0 0);
    --color-primary: oklch(0.65 0.20 264);
  }
}
```

### Tailwind Output Example

```typescript
import type { Config } from 'tailwindcss';

export default {
  theme: {
    colors: {
      surface: 'var(--color-surface)',
      'text-primary': 'var(--color-text-primary)',
      primary: 'var(--color-primary)',
    },
    fontFamily: {
      display: 'var(--font-display)',
      body: 'var(--font-body)',
      mono: 'var(--font-mono)',
    },
    // ...
  },
} satisfies Config;
```

## CLI Commands

```bash
collet-tokens init                    # Create tokens.yaml with defaults
collet-tokens validate                # Check without generating
collet-tokens build                   # Generate all configured outputs
collet-tokens build --output css      # Generate CSS only
collet-tokens build --output css,ios  # Specific outputs
collet-tokens watch                   # Rebuild on file change
collet-tokens diff tokens.yaml        # Show what changed vs last build
```

## Testing Strategy

```bash
# Unit tests — core logic
cargo test -p collet-tokens-core

# Integration tests — CLI end-to-end
cargo test -p collet-tokens-cli

# Golden file tests — output stability
# Each fixture in tests/fixtures/ has .expected/ with golden output.
# Tests generate output and diff against golden files.
```

## Code from rust-frontend

The following modules are extracted from `rust-frontend/crates/design-system/`:

| Source | Destination | What |
|--------|------------|------|
| `color.rs` | `core/src/color.rs` | oklch math, ColorRole, contrast |
| `typography.rs` | `core/src/typography.rs` | TextStyle, TextSize, FontWeight, scales |
| `spacing.rs` | `core/src/spacing.rs` | ComponentSize, grid math |
| `tokens.rs` | `output-css/src/lib.rs` | CSS generation (adapted) |
| `motion.rs` | `core/src/motion.rs` | Duration, easing curves |

These are adapted, not copied verbatim. The design-system crate was built for component styling (StyleBundle, utility classes). The token compiler strips the component-specific parts and keeps the pure token math.
