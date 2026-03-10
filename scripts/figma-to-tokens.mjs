#!/usr/bin/env node

// Fetches Figma Variables via REST API and converts to DTCG JSON format.
//
// Usage:
//   FIGMA_TOKEN=figd_xxxxx node scripts/figma-to-tokens.mjs <file-key>
//
// Output: DTCG JSON written to stdout. Pipe to a file:
//   FIGMA_TOKEN=figd_xxxxx node scripts/figma-to-tokens.mjs abc123 > tokens.json

const FIGMA_API = "https://api.figma.com/v1";

function usage() {
  console.error("Usage: FIGMA_TOKEN=<pat> node figma-to-tokens.mjs <file-key>");
  console.error("");
  console.error("  file-key   The Figma file key (from the file URL)");
  console.error("  FIGMA_TOKEN  Personal Access Token with file_variables:read scope");
  process.exit(1);
}

function die(message) {
  console.error(`Error: ${message}`);
  process.exit(1);
}

// Map Figma RGBA (0-1 floats) to oklch CSS string.
// Uses a simplified sRGB -> Oklab conversion for portable output.
function rgbaToOklch(r, g, b, a) {
  // sRGB to linear
  const linearize = (c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const lr = linearize(r);
  const lg = linearize(g);
  const lb = linearize(b);

  // Linear sRGB to Oklab (using the Oklab matrix)
  const l_ = 0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb;
  const m_ = 0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb;
  const s_ = 0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb;

  const l_cbrt = Math.cbrt(l_);
  const m_cbrt = Math.cbrt(m_);
  const s_cbrt = Math.cbrt(s_);

  const L = 0.2104542553 * l_cbrt + 0.7936177850 * m_cbrt - 0.0040720468 * s_cbrt;
  const A = 1.9779984951 * l_cbrt - 2.4285922050 * m_cbrt + 0.4505937099 * s_cbrt;
  const B = 0.0259040371 * l_cbrt + 0.7827717662 * m_cbrt - 0.8086757660 * s_cbrt;

  const C = Math.sqrt(A * A + B * B);
  let H = (Math.atan2(B, A) * 180) / Math.PI;
  if (H < 0) H += 360;

  const Lr = Math.round(L * 1000) / 1000;
  const Cr = Math.round(C * 1000) / 1000;
  const Hr = Math.round(H * 10) / 10;

  if (a !== undefined && a < 1) {
    return `oklch(${Lr} ${Cr} ${Hr} / ${Math.round(a * 100) / 100})`;
  }
  return `oklch(${Lr} ${Cr} ${Hr})`;
}

// Resolve a Figma variable value, handling aliases.
function resolveValue(value, variablesById) {
  if (value && typeof value === "object" && value.type === "VARIABLE_ALIAS") {
    const aliased = variablesById[value.id];
    if (!aliased) return undefined;
    // Use the first available mode value
    const modes = Object.values(aliased.valuesByMode);
    return modes.length > 0 ? resolveValue(modes[0], variablesById) : undefined;
  }
  return value;
}

// Convert a resolved Figma variable to a DTCG token entry.
function toToken(resolvedType, value) {
  if (resolvedType === "COLOR" && typeof value === "object" && "r" in value) {
    return { $value: rgbaToOklch(value.r, value.g, value.b, value.a), $type: "color" };
  }
  if (resolvedType === "FLOAT" && typeof value === "number") {
    return { $value: `${value}px`, $type: "dimension" };
  }
  if (resolvedType === "STRING" && typeof value === "string") {
    return { $value: value, $type: "string" };
  }
  if (resolvedType === "BOOLEAN" && typeof value === "boolean") {
    return { $value: value, $type: "boolean" };
  }
  return { $value: String(value), $type: "string" };
}

// Convert a slash-separated variable name to nested object path.
// e.g., "color/primary/500" -> ["color", "primary", "500"]
function namePath(name) {
  return name.split("/").map((s) => s.trim().replace(/\s+/g, "-").toLowerCase());
}

// Set a value at a nested path in an object.
function setNested(obj, path, value) {
  let current = obj;
  for (let i = 0; i < path.length - 1; i++) {
    if (!(path[i] in current) || typeof current[path[i]] !== "object") {
      current[path[i]] = {};
    }
    current = current[path[i]];
  }
  current[path[path.length - 1]] = value;
}

async function main() {
  const fileKey = process.argv[2];
  if (!fileKey) usage();

  const token = process.env.FIGMA_TOKEN;
  if (!token) die("FIGMA_TOKEN environment variable is required.");

  // Fetch variables from Figma
  const url = `${FIGMA_API}/files/${fileKey}/variables/local`;
  let response;
  try {
    response = await fetch(url, {
      headers: { "X-Figma-Token": token },
    });
  } catch (err) {
    die(`Network error: ${err.message}`);
  }

  if (!response.ok) {
    const body = await response.text().catch(() => "");
    if (response.status === 403) {
      die("403 Forbidden. Ensure your token has file_variables:read scope and your plan supports Variables REST API (Enterprise).");
    }
    die(`Figma API returned ${response.status}: ${body}`);
  }

  const data = await response.json();
  const meta = data.meta;
  if (!meta || !meta.variables) {
    die("Unexpected API response — no variables found. Is this file key correct?");
  }

  // Index variables by ID for alias resolution
  const variablesById = {};
  for (const variable of Object.values(meta.variables)) {
    variablesById[variable.id] = variable;
  }

  // Build DTCG output
  const output = {};

  for (const variable of Object.values(meta.variables)) {
    // Use the first mode's value (typically "Light" or default mode)
    const modeValues = Object.values(variable.valuesByMode);
    if (modeValues.length === 0) continue;

    const resolved = resolveValue(modeValues[0], variablesById);
    if (resolved === undefined) continue;

    const token = toToken(variable.resolvedType, resolved);
    const path = namePath(variable.name);
    setNested(output, path, token);
  }

  process.stdout.write(JSON.stringify(output, null, 2) + "\n");
}

main();
