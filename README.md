# Collet Tokens

A design token compiler that **validates before it generates**. One YAML file in, multi-platform code out.

```bash
collet-tokens build --input tokens.yaml

  ✓ 24 colors validated (WCAG AA contrast: all pass)
  ✓ Type scale: 1.25 ratio, base 16px, 13 roles
  ✓ Spacing: 4px grid, 18 steps
  Generated: dist/tokens.css (4.2KB)
  Generated: dist/tailwind.config.ts (1.1KB)
```

## Why

Style Dictionary transforms tokens. It doesn't validate them.

Collet Tokens checks every text/background color pair for WCAG contrast, verifies your type scale ratio is consistent, ensures spacing aligns to your grid — then generates platform code. Mistakes are caught at build time, not in production.

## Install

```bash
cargo install collet-tokens-cli
```

## Quick Start

```bash
# Create a starter token file
collet-tokens init

# Validate without generating
collet-tokens validate --input tokens.yaml

# Build CSS + Tailwind output
collet-tokens build --input tokens.yaml
```

## Token File

```yaml
collet: "1.0"

fonts:
  display: "'Plus Jakarta Sans', system-ui, sans-serif"
  body: "'Inter', system-ui, sans-serif"
  mono: "'JetBrains Mono', monospace"

colors:
  surface:
    light: { l: 1.0, c: 0.0, h: 0 }
    dark:  { l: 0.13, c: 0.01, h: 280 }
  text-primary:
    light: { l: 0.27, c: 0.003, h: 90 }
    dark:  { l: 0.87, c: 0.0, h: 0 }
  primary:
    light: { l: 0.55, c: 0.25, h: 264 }
    dark:  { l: 0.65, c: 0.20, h: 264 }

typography:
  scale-ratio: 1.25
  base-size: 16

spacing:
  base: 4
  scale: [1, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24, 32, 40, 48, 64]

validation:
  contrast: "AA"
```

## What It Generates

### CSS (`tokens.css`)

```css
:root {
  --font-display: 'Plus Jakarta Sans', system-ui, sans-serif;
  --font-body: 'Inter', system-ui, sans-serif;
  --color-surface: oklch(1 0 0);
  --color-text-primary: oklch(0.27 0.003 90);
  --space-4: 1rem;
  /* ... all tokens as custom properties */
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-surface: oklch(0.13 0.01 280);
    --color-text-primary: oklch(0.87 0 0);
  }
}

/* 13 typography roles */
.t-display { font-size: clamp(3rem, 5vw + 1rem, 4.5rem); /* ... */ }
.t-h1 { font-size: clamp(2rem, 3.5vw + 0.5rem, 3.25rem); /* ... */ }
.t-body-md { font-size: 1rem; line-height: 1.6; /* ... */ }
.t-code { font-family: var(--font-mono); /* ... */ }

/* 6 composable modifiers */
.t--muted { opacity: 0.5; }
.t--strong { font-weight: 700; }
.t--truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
```

### Tailwind (`tailwind.config.ts`)

```typescript
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
  },
} satisfies Config;
```

## Validation

| Check | What it catches |
|-------|----------------|
| **Contrast** | Text/background pairs that fail WCAG AA (4.5:1) or AAA (7:1) |
| **Type scale** | Scale steps that deviate from the configured ratio |
| **Spacing grid** | Values not aligned to the base grid unit |
| **Font fallback** | Font stacks missing system fallback families |
| **Body size** | Base body font below accessible minimum |
| **Gamut** | oklch colors that clip when converted to sRGB |

Errors include actionable fix suggestions:

```
error: [contrast-fail] Light mode contrast 2.67:1 is below 4.5:1 (WCAG AA)
       at colors.text-muted / colors.surface
       fix: Increase lightness difference between 'text-muted' and 'surface'
```

## CLI Reference

```bash
collet-tokens build [OPTIONS]
  --input <PATH>      Token file (YAML or JSON)
  --output <FORMATS>  Comma-separated: css,tailwind (default: all)
  --outdir <DIR>      Output directory (default: ./dist)

collet-tokens validate [OPTIONS]
  --input <PATH>      Token file to validate

collet-tokens init
  Creates a starter tokens.yaml in the current directory
```

## Architecture

```
tokens.yaml → Parser → Validator → Resolver → Output Generators
                          ↓
                    Issues (errors,
                    warnings, fixes)
```

- **Parser**: YAML/JSON auto-detected, defaults applied for missing sections
- **Validator**: WCAG contrast, type scale, spacing grid, font fallbacks
- **Resolver**: Computes derived values (dark mode, fluid clamp, rem conversion)
- **Output**: Each platform is a separate crate (`output-css`, `output-tailwind`, ...)

## Colors

Colors use **oklch** — a perceptually uniform color space that produces predictable contrast ratios and works across wide-gamut (P3) displays:

```yaml
primary:
  light: { l: 0.55, c: 0.25, h: 264 }  # l=lightness, c=chroma, h=hue
  dark:  { l: 0.65, c: 0.20, h: 264 }
```

## Typography

13 semantic roles generated from a base size and scale ratio:

| Tier | Roles | Font | Description |
|------|-------|------|-------------|
| **Display** | display, h1, h2, h3 | Display | Headings. Fluid `clamp()` for display/h1/h2. |
| **Label** | label-lg, label-md, label-sm | Body | UI chrome. Medium weight, tighter leading. |
| **Body** | body-lg, body-md, body-sm | Body | Prose. Regular weight, relaxed leading. |
| **Utility** | overline, caption, code | Body/Mono | Specialized. Overline is uppercase + wide tracking. |

## License

MIT OR Apache-2.0
