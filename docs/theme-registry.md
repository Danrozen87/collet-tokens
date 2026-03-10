# Collet Theme Registry — Technical Scope

## What It Is

A service that manages branded themes for white-label applications built with Collet Components. Each customer gets their own token file, validated and versioned. The service generates CSS on upload and serves the correct version at runtime.

## The Problem It Solves

Without a registry, multi-tenant theming looks like this:

1. Customer requests a brand change
2. Developer manually edits a tokens.yaml
3. Developer runs `collet-tokens build` locally
4. Developer commits the generated CSS
5. Developer deploys
6. Repeat for every customer, every change

With a registry:

1. Customer (or their designer) uploads brand values via API or dashboard
2. Registry validates (contrast, scale, grid — rejects bad tokens immediately)
3. Registry generates CSS + stores versioned output
4. App loads the CSS at runtime via customer ID
5. Done. No developer in the loop for brand changes.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Theme Registry Service                     │
│                                                               │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌────────┐ │
│  │  Upload   │───▸│ Validate │───▸│ Generate │───▸│ Store  │ │
│  │  API      │    │ (WASM)   │    │ (WASM)   │    │ (S3)   │ │
│  └──────────┘    └──────────┘    └──────────┘    └────────┘ │
│                                                       │       │
│  ┌──────────┐                                         │       │
│  │  Serve   │◂────────────────────────────────────────┘       │
│  │  API     │                                                 │
│  └──────────┘                                                 │
└─────────────────────────────────────────────────────────────┘

Upload API:  POST /themes/{customer-id}  (token YAML/JSON body)
Serve API:   GET  /themes/{customer-id}/tokens.css
             GET  /themes/{customer-id}/tokens.css?v=1.2
Version API: GET  /themes/{customer-id}/versions
Diff API:    GET  /themes/{customer-id}/diff?from=1.1&to=1.2
```

## How the App Consumes It

### Option A: CDN URL (simplest)

```html
<!-- In index.html or layout.tsx -->
<link rel="stylesheet" href="https://themes.collet.dev/{customer-id}/tokens.css" />
```

The browser caches it. Cache-bust with version: `?v=1.2`.

### Option B: Runtime injection (SPA)

```typescript
// In your app's initialization
const customerId = getCurrentTenant();
const link = document.createElement('link');
link.rel = 'stylesheet';
link.href = `https://themes.collet.dev/${customerId}/tokens.css`;
document.head.appendChild(link);
```

### Option C: Build-time (SSR/SSG)

```typescript
// In Next.js layout.tsx
import { headers } from 'next/headers';

export default async function RootLayout({ children }) {
  const customerId = headers().get('x-tenant-id');
  return (
    <html>
      <head>
        <link rel="stylesheet" href={`https://themes.collet.dev/${customerId}/tokens.css`} />
      </head>
      <body>{children}</body>
    </html>
  );
}
```

## Data Model

```
Theme {
  customer_id: String       // "acme-corp"
  version: SemVer           // "1.2.0"
  tokens_yaml: String       // raw input
  tokens_css: String         // generated output (cached)
  tailwind_config: String    // generated Tailwind output
  ios_swift: String          // generated iOS output
  android_xml: Vec<String>   // generated Android outputs
  validation: ValidationResult {
    passed: bool
    errors: Vec<Issue>
    warnings: Vec<Issue>
  }
  created_at: DateTime
  created_by: String        // user/API key
}
```

## Version Management

Every upload creates a new version. Old versions are never deleted — immutable history.

```
POST /themes/acme-corp
  Body: tokens.yaml content
  Response: { version: "1.3.0", validation: { passed: true, warnings: [...] } }

GET /themes/acme-corp/tokens.css          → latest version
GET /themes/acme-corp/tokens.css?v=1.2.0  → specific version
GET /themes/acme-corp/versions            → [{ version: "1.3.0", created_at: "..." }, ...]
GET /themes/acme-corp/diff?from=1.2.0&to=1.3.0  → what changed
```

## Implementation Options

### Option 1: Serverless (recommended for v1)

- **Cloudflare Workers** for the API (WASM runs natively in Workers)
- **R2 Storage** for generated CSS (S3-compatible, free egress)
- **KV** for version metadata

Why: The collet-tokens-core crate compiles to WASM. Cloudflare Workers execute WASM natively. Validation and generation happen at the edge — upload in Tokyo, validated in Tokyo, no round-trip to a central server. Sub-50ms response times.

Cost: Free tier handles thousands of themes. $5/mo Workers plan handles production traffic.

### Option 2: Simple API server

- **Axum** (Rust) API server
- **S3** for storage
- **PostgreSQL** for version metadata

Why: Full control, self-hostable, no vendor lock-in. The collet-tokens-core crate links directly (no WASM indirection). Enterprise customers who can't use SaaS can run it themselves.

### Option 3: Git-based (zero infrastructure)

- Themes stored as files in a GitHub repo
- GitHub Actions validates on PR
- GitHub Pages or Netlify serves the CSS
- Version = git tag

Why: No service to maintain. Works today with what we have. The "registry" is just a repo with a folder structure.

## Security

| Concern | Mitigation |
|---------|-----------|
| Token file injection | Parsed by serde (structured data), not evaluated. No code execution. |
| CSS injection | Output generated from validated Rust structs, not templated from input strings. |
| Unauthorized uploads | API key per customer. Rate limiting on upload endpoint. |
| Version tampering | Versions are immutable. Once stored, content never changes. |
| Customer data isolation | Each customer ID is a separate namespace. No cross-tenant access. |

## API Authentication

```
# Upload (requires API key)
POST /themes/acme-corp
Authorization: Bearer ct_live_abc123...
Content-Type: text/yaml

# Serve (public — CSS is not secret)
GET /themes/acme-corp/tokens.css
# No auth needed — these are CSS custom properties, not credentials
```

## Dashboard (future)

A simple web UI where customers can:
- See their current theme (live preview with Collet components)
- Upload new token files (drag-and-drop YAML/JSON)
- See validation results inline
- Preview changes before publishing
- Roll back to previous versions
- Export for iOS/Android/Tailwind

Built with Collet Components + Collet Tokens. Dogfooding the entire stack.

## Pricing Model

| Tier | Themes | Features | Price |
|------|--------|----------|-------|
| Free | 1 | CSS output, manual upload | $0 |
| Starter | 5 | All outputs, API access, version history | $29/mo |
| Team | 25 | Dashboard, preview, rollback, CI integration | $99/mo |
| Enterprise | Unlimited | Self-hosted option, SSO, audit log, SLA | $499/mo |

## What To Build First

1. **Git-based registry (Option 3)** — works today, zero infrastructure
   - Create a `themes/` directory structure in the repo
   - GitHub Action validates + generates on PR
   - Serve via GitHub Pages or any static host
   - This is the MVP that proves the concept

2. **Upload API (Option 1)** — when customers need self-service
   - Cloudflare Worker + R2
   - One afternoon of work once WASM build exists

3. **Dashboard** — when visual management matters
   - Built with Collet Components
   - Live preview of theme changes
