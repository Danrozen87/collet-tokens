# Collet Tokens — Project Instructions

## What This Project Is

A design token compiler with built-in validation. One YAML/JSON input file produces validated, platform-specific output (CSS, Tailwind, iOS Swift, Android XML, Figma tokens). The core differentiator is **compile-time validation** — WCAG contrast checking, type scale consistency, spacing grid alignment — powered by Rust.

This is NOT a component library. This is the **token engine** extracted from the Collet component library (`rust-frontend/crates/design-system/`), repackaged as a standalone tool.

## Architecture

```
collet-tokens/
  crates/
    core/           ← Token types, parser, validator, resolver (pure Rust, no I/O)
    output-css/     ← CSS custom properties + utility classes
    output-tailwind/← Tailwind v4 theme config
    output-ios/     ← Swift UIColor + Font extensions
    output-android/ ← XML resource files
    output-figma/   ← Figma Tokens JSON format
    cli/            ← Binary entry point (clap CLI)
    wasm/           ← WASM build for browser/Figma plugin
  tests/
    fixtures/       ← Sample token files for integration tests
  docs/
    schema.md       ← Token format specification
```

### Core Crate (`crates/core/`)

Pure library. No filesystem, no stdout, no side effects. Takes `&str` (YAML/JSON), returns `Result<ResolvedTokens, Vec<Issue>>`.

```rust
pub fn compile(input: &str) -> Result<ResolvedTokens, Vec<Issue>>;
pub fn validate(input: &str) -> Vec<Issue>;
```

### Output Crates (`crates/output-*/`)

Each output crate takes `&ResolvedTokens` and returns a `Vec<OutputFile>`:

```rust
pub fn generate(tokens: &ResolvedTokens) -> Vec<OutputFile>;

pub struct OutputFile {
    pub path: String,      // e.g., "tokens.css"
    pub content: String,   // file content
}
```

### CLI Crate (`crates/cli/`)

Thin wrapper: parse args → read file → call core → write output files.

## Engineering Standards

### From the Collet Component Library

This project inherits the same standards as `rust-frontend`:

- `unsafe_code = "forbid"` — no unsafe anywhere
- `clippy::all = "deny"`, `clippy::pedantic = "warn"` — zero warnings
- `clippy::unwrap_used = "deny"`, `clippy::panic = "deny"` — no unwrap/panic
- Every public item has a doc comment
- Every enum derives `Debug, Clone, Copy, PartialEq, Eq, Hash`
- Tests verify behavior, not just compilation

### Validation-Specific Standards

- **oklch color math must be exact.** Use `f64` for all color computations. No `f32` shortcuts. The validator's credibility depends on accurate contrast ratios.
- **Error messages must be actionable.** Not "contrast failed" but "text-muted (#999) fails 4.5:1 contrast against bg-surface (#fff) — ratio is 2.85:1. Minimum required: 4.5:1 for AA normal text."
- **The parser must accept partial input gracefully.** Missing sections default to sensible values. Only truly invalid input (malformed YAML, unknown keys) should error.
- **Output must be deterministic.** Same input → same output, byte-for-byte. No timestamps, no random IDs, no non-deterministic iteration order.

## Key Design Decisions

### oklch Over Hex/RGB

Colors are defined in oklch (Oklab Lightness, Chroma, Hue) because:
1. Perceptually uniform — equal lightness steps look equally spaced to humans
2. Wider gamut — P3 displays are standard on Apple devices since 2016
3. Predictable manipulation — rotating hue doesn't shift perceived brightness
4. CSS Color Level 4 — `oklch()` is supported in all modern browsers

The validator computes contrast in oklch space using the APCA (Accessible Perceptual Contrast Algorithm) method, falling back to WCAG 2.x luminance ratio for compatibility.

### YAML Over JSON

YAML supports comments. Design tokens need comments ("this color is the brand primary — do not change without marketing approval"). JSON doesn't allow comments. The parser accepts both formats.

### Semantic Roles Over Raw Scales

The typography system generates 13 semantic roles (display, h1-h3, label-lg/md/sm, body-lg/md/sm, overline, caption, code) from a base size + scale ratio. This is more useful than "font-size-1 through font-size-10" because the role names carry intent. When a designer says "make the body text bigger," the developer changes `base-size: 16` to `base-size: 18` and every role adjusts proportionally.

### Dark Mode as Derivation, Not Duplication

Colors are defined with explicit light and dark values. The resolver computes intermediate states (hover, active, disabled) automatically from the base values using oklch math. This means adding a new color requires TWO values (light + dark), not TEN (light + dark × 5 states).

## Workflow

### Adding a New Output Format

1. Create `crates/output-{platform}/`
2. Implement `pub fn generate(tokens: &ResolvedTokens) -> Vec<OutputFile>`
3. Register in `crates/cli/src/main.rs` output dispatch
4. Add integration test in `tests/fixtures/`
5. Document in `docs/outputs.md`

### Adding a New Validation Rule

1. Add to `crates/core/src/validator.rs`
2. Return `Issue` with severity (Error, Warning, Info), location, message, and fix suggestion
3. Add test case in `crates/core/tests/`
4. Document in `docs/validation.md`

## Rust Settings

```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT OR Apache-2.0"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
panic = "deny"
```

## Dependencies (Minimal)

```toml
# Core (zero runtime dependencies beyond these)
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"

# CLI only
clap = { version = "4", features = ["derive"] }
colored = "3"

# WASM only
wasm-bindgen = "0.2"
```

## Testing

```bash
cargo test --workspace              # All tests
cargo test -p collet-tokens-core    # Core logic only
cargo test -p collet-tokens-cli     # CLI integration tests
```

Integration tests use fixture files in `tests/fixtures/`:
- `valid-minimal.yaml` — smallest valid input
- `valid-full.yaml` — all sections populated
- `invalid-contrast.yaml` — deliberate contrast failures
- `invalid-scale.yaml` — broken type scale
- `invalid-syntax.yaml` — malformed YAML

Each fixture has a corresponding `.expected/` directory with golden output files.
