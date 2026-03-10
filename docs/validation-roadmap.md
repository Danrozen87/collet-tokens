# Validation Roadmap — Configurability & Per-Token Control

## Current State (v0.1.0)

Six validation rules, all based on measurable standards:

| Rule | Standard | Configurable |
|------|----------|-------------|
| Contrast | WCAG 2.x luminance ratio (4.5:1 AA, 7:1 AAA) | `contrast_level: "AA"` or `"AAA"` |
| Type scale | Mathematical ratio consistency (±5% tolerance) | `scale_ratio` in typography config |
| Spacing grid | Base unit alignment | `spacing_grid: 4` (or 8, etc.) |
| Font fallback | System generic required in stack | Not configurable (always on) |
| Body size | Minimum px for base text | `min_body_size: 16` (adjustable) |
| Gamut | sRGB clipping detection for oklch colors | Not configurable (warning only) |

## Planned: Selective Validation

### Per-token exemptions

Designers sometimes intentionally break contrast for decorative text, watermarks, or disabled states. The tool should allow acknowledged violations rather than forcing a fix:

```yaml
colors:
  watermark:
    light: { l: 0.85, c: 0.0, h: 0 }
    dark: { l: 0.2, c: 0.0, h: 0 }
    validation: { contrast: skip }     # intentionally low contrast, reviewed
```

Implementation: Add optional `validation` field to `ColorPair` in `types.rs`. Check for `skip` in `validator.rs` before running contrast checks on that pair.

### Color pair scoping

Currently checks ALL text colors × ALL surface colors. Should support explicit pairing to reduce noise:

```yaml
validation:
  contrast_level: "AA"
  pairs:
    - text: text-primary
      on: [surface, surface-raised]
    - text: text-muted
      on: [surface]
      # text-muted on surface-raised intentionally not checked
```

Implementation: If `pairs` is defined, only check those combinations. If omitted, fall back to checking all combinations (current behavior).

### Severity overrides

Downgrade a specific rule from error to warning for a specific project:

```yaml
validation:
  overrides:
    - rule: contrast-fail
      for: "text-muted / surface-raised"
      severity: warning    # acknowledge but don't block CI
```

### Custom rules

Allow project-specific validation beyond the built-in rules:

```yaml
validation:
  custom:
    - name: brand-chroma
      check: "primary.light.c >= 0.15"
      message: "Primary color must have enough vibrancy for brand recognition"
    - name: dark-mode-exists
      check: "all colors have dark variant"
      message: "Every color must define a dark mode value"
```

This is more complex — requires a mini expression language. Consider whether WASM plugin system (user-provided validation functions) is more practical than a DSL.

## Design Principle

**Strict by default, opt-out with documentation.**

Every exemption requires an explicit annotation in the token file. This means:
- The violation is version-controlled (visible in PR diffs)
- The decision is documented (why was contrast skipped?)
- Auditors can grep for `validation: { contrast: skip }` to find all exemptions
- Default behavior catches everything — you only opt out when you've thought about it
