# Collet Tokens — Vision Statement

## The Problem

Design tokens are the atoms of every design system — colors, fonts, spacing, radii, motion. Today, the lifecycle of a design token looks like this:

1. A designer chooses values in Figma
2. A developer manually translates them into CSS custom properties
3. Another developer translates them again for iOS (Swift UIColor)
4. Another developer translates them again for Android (XML resources)
5. A QA engineer eventually discovers that the grey text is unreadable on the light background
6. Someone files an accessibility bug 6 months after launch

**Every step is manual. Every step introduces drift. Nobody validates until it's too late.**

The incumbent tool — Style Dictionary — is a JSON transformer. It takes token JSON and outputs platform files. It has zero opinions about whether your tokens are correct. It won't tell you that your `text-secondary` color fails WCAG contrast against your `bg-surface`. It won't catch that your type scale broke the 1.25 ratio at step 7. It won't warn that your spacing values don't align to the 4px grid.

Style Dictionary transforms. It does not validate.

## The Solution

**Collet Tokens is a design token compiler with a built-in validation engine.**

One input file. Validated. Multi-platform output.

```bash
collet-tokens build --input tokens.yaml

✓ 24 colors validated (WCAG AA contrast: all pass)
✗ Error: "text-muted" (#999) fails 4.5:1 contrast against "bg-surface" (#fff) — ratio 2.85:1
✓ Type scale: 1.25 ratio, 10 steps, base 16px
✓ Spacing: 4px grid, 14 steps, no violations
✓ Generated: tokens.css (4.2KB)
✓ Generated: tailwind.config.ts (1.1KB)
✓ Generated: Colors.swift (0.8KB)
✓ Generated: colors.xml (0.6KB)
```

### Core Principles

1. **Validate, don't just transform.** Every color pair is checked for WCAG contrast. Every type scale step is verified. Every spacing value is validated against the grid. Errors are caught at build time, not in production.

2. **One file, every platform.** CSS custom properties, Tailwind v4 config, iOS Swift extensions, Android XML resources, Figma Tokens JSON — all generated from one `tokens.yaml`. Zero manual translation.

3. **Rust-powered, WASM-portable.** The compiler is written in Rust. It runs as a CLI (`cargo install collet-tokens`), as an npm package (`npx @collet/tokens build`), in CI (GitHub Action), and in the browser (WASM for playground/Figma plugin). Same engine everywhere.

4. **W3C Design Tokens compatible.** The input format extends the W3C Design Tokens Community Group draft spec. Standard tokens work. Collet-specific extensions add validation rules and semantic roles.

5. **Progressive adoption.** Start with CSS output only. Add platforms as you need them. The free CLI covers 90% of use cases. Team/enterprise tiers add CI integration, audit logs, and bidirectional Figma sync.

## What Makes This Different

| Feature | Style Dictionary | Collet Tokens |
|---------|-----------------|---------------|
| Contrast validation | No | WCAG AA/AAA, every text/bg pair |
| Type scale validation | No | Ratio consistency, minimum sizes, fluid bounds |
| Spacing grid validation | No | Base unit alignment, gap detection |
| Color space | Hex/RGB | oklch (perceptually uniform, wider gamut) |
| Dark mode | Manual duplication | Automatic — one color, both modes derived |
| Implementation | JavaScript | Rust (CLI) + WASM (browser/Figma) |
| Typography roles | No | 13 semantic roles (display → code) |
| Font pair presets | No | Editorial, Product, Geometric (swappable) |
| Output: CSS | Yes | Yes (custom properties + utility classes) |
| Output: Tailwind | Plugin needed | Native v4 config |
| Output: iOS | Community plugin | Built-in Swift extensions |
| Output: Android | Community plugin | Built-in XML resources |
| Output: Figma | No | Figma Tokens JSON + plugin |

## Architecture

```
tokens.yaml (input)
    │
    ▼
┌─────────────┐
│   Parser     │  ← YAML/JSON → Rust structs
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Validator   │  ← Contrast, scale, grid, a11y checks
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Resolver    │  ← Compute derived values (dark mode, fluid clamp, etc.)
└──────┬──────┘
       │
       ├──► CSS output (tokens.css, tokens-shadow.css)
       ├──► Tailwind output (tailwind.config.ts)
       ├──► iOS output (Colors.swift, Typography.swift)
       ├──► Android output (colors.xml, dimens.xml, type.xml)
       └──► Figma output (figma-tokens.json)
```

The core engine (Parser → Validator → Resolver) is a pure Rust library crate with zero platform dependencies. Output generators are separate crates. The CLI crate ties them together.

## The Token Format

```yaml
collet: "1.0"

# ─── Fonts ───
fonts:
  display: "'Plus Jakarta Sans', system-ui, sans-serif"
  body: "'Inter', system-ui, sans-serif"
  mono: "'JetBrains Mono', monospace"

# ─── Colors (oklch) ───
colors:
  # Surfaces
  surface:
    light: { l: 1.0, c: 0.0, h: 0 }       # pure white
    dark:  { l: 0.13, c: 0.01, h: 280 }    # near black
  surface-raised:
    light: { l: 0.985, c: 0.003, h: 90 }
    dark:  { l: 0.17, c: 0.01, h: 280 }

  # Text
  text-primary:
    light: { l: 0.27, c: 0.003, h: 90 }    # near black
    dark:  { l: 0.87, c: 0.0, h: 0 }       # near white
  text-muted:
    light: { l: 0.55, c: 0.02, h: 75 }
    dark:  { l: 0.65, c: 0.02, h: 75 }

  # Semantic
  primary:
    light: { l: 0.55, c: 0.25, h: 264 }    # indigo
    dark:  { l: 0.65, c: 0.20, h: 264 }
  success:
    light: { l: 0.55, c: 0.18, h: 155 }
    dark:  { l: 0.65, c: 0.15, h: 155 }
  warning:
    light: { l: 0.75, c: 0.18, h: 85 }
    dark:  { l: 0.80, c: 0.15, h: 85 }
  danger:
    light: { l: 0.55, c: 0.22, h: 25 }
    dark:  { l: 0.65, c: 0.18, h: 25 }

# ─── Typography ───
typography:
  scale-ratio: 1.25          # major third
  base-size: 16              # px
  fluid-headings: true       # clamp() for display tier
  roles: auto                # generate all 13 from scale

# ─── Spacing ───
spacing:
  base: 4                    # px
  scale: [0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 14, 16, 20, 24, 32, 40, 48, 64]

# ─── Radius ───
radius:
  none: "0"
  sm: "0.25rem"
  md: "0.5rem"
  lg: "0.75rem"
  xl: "1rem"
  full: "9999px"

# ─── Motion ───
motion:
  duration:
    fast: "100ms"
    normal: "200ms"
    smooth: "350ms"
    slow: "500ms"
  easing:
    linear: "linear"
    ease-out: "cubic-bezier(0.16, 1, 0.3, 1)"
    spring: "linear(0, 0.009, 0.035 2.1%, 0.141, 0.281 6.7%, 0.723 12.9%, 0.938 16.7%, 1.017, 1.077, 1.121, 1.149 24.3%, 1.159, 1.163, 1.161, 1.154 29.9%, 1.129 32%, 1.051 36.4%, 1.017 38.5%, 0.991, 0.977 42%, 0.974 43.5%, 0.975 44.7%, 0.978 46.2%, 0.993 49.8%, 1.001 51.5%, 1.007 53.8%, 1.009 58.3%, 1.004 63.1%, 0.998 70%, 1)"

# ─── Validation ───
validation:
  contrast: "AA"              # AA (4.5:1) or AAA (7:1)
  min-body-size: 16           # px — WCAG guidance
  max-font-families: 3        # performance guard
  spacing-grid: 4             # enforce 4px grid
```

## Revenue Model

| Tier | Price | Features |
|------|-------|----------|
| **Free** | $0 | CLI, CSS + Tailwind output, local validation |
| **Team** | $99/mo | All outputs (iOS, Android, Figma), GitHub Action, audit log |
| **Enterprise** | $499/mo | Custom outputs, doc site generator, Figma plugin sync, multi-brand |

## Milestones

1. **CLI + CSS output + validator** — the minimum viable product
2. **Tailwind output** — captures the largest web audience
3. **iOS + Android output** — cross-platform teams
4. **Figma plugin** — closes the designer-developer loop
5. **CI GitHub Action** — enterprise integration
6. **SaaS playground** — browser-based token editor with live preview

## Origin

Collet Tokens is extracted from the Collet component library's design system engine (`crates/design-system/`). The color math, type scale, spacing grid, and motion system were built and battle-tested across 48 production components with 2,979 tests. This is not a greenfield project — it's a proven engine being repackaged for standalone use.
