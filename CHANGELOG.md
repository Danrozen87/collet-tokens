# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-10

### Added

- **Core engine**: Parser (YAML/JSON auto-detect), Validator, Resolver
- **oklch color math**: Full oklch → oklab → linear sRGB → gamma sRGB pipeline
- **WCAG contrast validation**: Checks every text/background pair in both light and dark modes
- **Type scale validation**: Verifies ratio consistency across all scale steps
- **Spacing grid validation**: Catches values not aligned to the base grid unit
- **Font fallback validation**: Warns when font stacks lack system fallback families
- **13 typography roles**: display, h1-h3, label-lg/md/sm, body-lg/md/sm, overline, caption, code
- **Fluid headings**: `clamp()` sizing for display/h1/h2 tiers
- **CSS output**: Custom properties + typography classes + modifier classes + dark mode
- **Tailwind output**: Theme config mapping CSS vars for colors, fonts, spacing, radius
- **CLI**: `build`, `validate`, `init` commands with colored output and actionable error messages
- **75 tests**: Core logic, output generators, and edge cases
