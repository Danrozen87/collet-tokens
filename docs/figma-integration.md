# Figma Integration

Three paths to get design tokens from Figma into collet-tokens.

---

## Path 1: Tokens Studio Export (Free — any Figma plan)

The simplest approach. No API keys, no scripting. Works on any Figma plan.

### Setup

1. Install the [Tokens Studio](https://tokens.studio/) plugin in Figma
2. Define your design tokens in the plugin (colors, typography, spacing, etc.)
3. Configure sync to your Git repository (GitHub, GitLab, or local export)

### Workflow

1. Export tokens as **DTCG JSON** from Tokens Studio
2. Commit the exported `tokens.json` to your repo
3. Run validation locally or in CI:

```bash
collet-tokens build --input tokens.json
```

4. CI validates automatically on every PR (see `examples/github-action-usage.yml`)

### Token Format

Tokens Studio exports DTCG (Design Token Community Group) format by default:

```json
{
  "color": {
    "primary": {
      "$value": "oklch(0.65 0.25 264)",
      "$type": "color"
    },
    "surface": {
      "$value": "oklch(0.98 0.005 264)",
      "$type": "color"
    }
  }
}
```

collet-tokens accepts this format natively — no conversion needed.

---

## Path 2: Figma Variables REST API (Enterprise plan required)

Direct programmatic access to Figma Variables. Requires a Figma Enterprise plan
for the Variables REST API.

### Setup

1. Go to [Figma Settings > Personal Access Tokens](https://www.figma.com/developers/api#access-tokens)
2. Create a token with `file_variables:read` scope
3. Note your Figma file key (from the file URL: `figma.com/file/<FILE_KEY>/...`)

### Workflow

Use the included converter script to pull variables and produce DTCG JSON:

```bash
FIGMA_TOKEN=figd_xxxxx node scripts/figma-to-tokens.mjs <file-key> > tokens.json
collet-tokens build --input tokens.json
```

### API Endpoint

The script calls:

```
GET https://api.figma.com/v1/files/:file_key/variables/local
```

This returns all local variables in the file, including color, number, string,
and boolean variables organized by collection.

### Automating in CI

```yaml
# .github/workflows/sync-figma.yml
name: Sync Figma Tokens
on:
  schedule:
    - cron: '0 6 * * 1' # Weekly on Monday at 06:00 UTC
  workflow_dispatch:

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Fetch Figma variables
        run: |
          node scripts/figma-to-tokens.mjs ${{ secrets.FIGMA_FILE_KEY }} > tokens.json
        env:
          FIGMA_TOKEN: ${{ secrets.FIGMA_TOKEN }}
      - name: Validate
        run: collet-tokens validate --input tokens.json
      - name: Commit if changed
        run: |
          git diff --quiet tokens.json || {
            git config user.name "github-actions"
            git config user.email "actions@github.com"
            git add tokens.json
            git commit -m "chore: sync Figma tokens"
            git push
          }
```

---

## Path 3: Figma MCP Server (for AI-assisted workflows)

Use Figma's MCP (Model Context Protocol) server to let an AI assistant extract
tokens directly from your Figma file.

### Setup

1. Install the Figma MCP server in your AI IDE:
   - **Claude Code:** Add to your MCP config
   - **Cursor / Windsurf:** Follow their MCP integration docs
2. Connect to your Figma file

### Workflow

1. Ask your AI assistant to inspect the Figma file:

   > "Extract all color, typography, and spacing tokens from this Figma file
   > and generate a tokens.yaml for collet-tokens."

2. The AI reads the Figma file via MCP and generates a structured token file

3. Validate the output:

```bash
collet-tokens validate --input tokens.yaml
```

4. Iterate with the AI if validation reports issues:

   > "The validator says text-muted fails AA contrast against bg-surface.
   > Adjust the lightness to meet 4.5:1 ratio."

### When to Use This Path

- Exploratory work — pulling tokens from an existing Figma file for the first time
- One-off extractions where scripting is overkill
- Design review sessions where you want live validation feedback

For ongoing sync, Path 1 (Tokens Studio) or Path 2 (REST API) are more reliable
since they do not depend on AI interpretation.

---

## Choosing a Path

| Criteria            | Tokens Studio     | REST API          | MCP Server        |
|---------------------|-------------------|-------------------|-------------------|
| Figma plan required | Any               | Enterprise        | Any (with MCP)    |
| Automation          | Git sync built-in | Scriptable in CI  | Manual / ad-hoc   |
| Setup effort        | Low               | Medium            | Low               |
| Accuracy            | Exact (plugin)    | Exact (API)       | AI-interpreted    |
| Best for            | Teams using Figma  | CI/CD pipelines  | Exploration       |

All three paths produce the same output: a token file that collet-tokens can
validate and compile. Mix and match as needed.
