//! CLI entry point for the collet-tokens design token compiler.
//!
//! Provides three subcommands:
//! - `build` — parse, validate, and generate output files
//! - `validate` — parse and validate without generating output
//! - `init` — create a starter `tokens.yaml`

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;

use collet_tokens_core::issue::{Issue, Severity};

/// Collet Tokens — design token compiler with built-in validation.
#[derive(Parser)]
#[command(name = "collet-tokens", version, about)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Command,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
enum Command {
    /// Parse, validate, and generate output files from a token file.
    Build {
        /// Path to the input token file (YAML or JSON).
        #[arg(short, long)]
        input: PathBuf,

        /// Comma-separated list of output formats to generate.
        ///
        /// Supported: `css`, `tailwind`, `ios`, `android`. Defaults to `css,tailwind`.
        #[arg(short, long, value_delimiter = ',', default_values_t = vec!["css".to_owned(), "tailwind".to_owned()])]
        output: Vec<String>,

        /// Output directory. Created if it does not exist.
        #[arg(long, default_value = "./dist")]
        outdir: PathBuf,
    },

    /// Parse and validate a token file without generating output.
    Validate {
        /// Path to the input token file (YAML or JSON).
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Create a starter `tokens.yaml` in the current directory.
    Init,
}

/// CLI error type for I/O and compilation failures.
#[derive(Debug)]
enum CliError {
    /// Failed to read or write a file.
    Io {
        /// What operation was being performed.
        context: String,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Token compilation produced error-level issues.
    Compilation {
        /// The issues that caused the failure.
        issues: Vec<Issue>,
    },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { context, source } => {
                write!(f, "{context}: {source}")
            }
            Self::Compilation { issues } => {
                let count = issues.iter().filter(|i| i.is_error()).count();
                write!(f, "Compilation failed with {count} error(s)")
            }
        }
    }
}

impl CliError {
    /// Create an I/O error with context.
    fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Build {
            input,
            output,
            outdir,
        } => run_build(&input, &output, &outdir),
        Command::Validate { input } => run_validate(&input),
        Command::Init => run_init(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            match &e {
                CliError::Io { .. } => {
                    eprintln!("{} {e}", "error:".red().bold());
                }
                CliError::Compilation { issues } => {
                    print_issues(issues);
                }
            }
            ExitCode::FAILURE
        }
    }
}

/// Execute the `build` subcommand.
fn run_build(input: &Path, outputs: &[String], outdir: &Path) -> Result<(), CliError> {
    let contents = read_input(input)?;

    let (resolved, warnings) = collet_tokens_core::compile(&contents)
        .map_err(|issues| CliError::Compilation { issues })?;

    // Print warnings (non-blocking).
    if !warnings.is_empty() {
        print_issues(&warnings);
        eprintln!();
    }

    // Create output directory.
    fs::create_dir_all(outdir).map_err(|e| {
        CliError::io(
            format!("Failed to create output directory {}", outdir.display()),
            e,
        )
    })?;

    // Generate and write output files.
    let mut file_count = 0;

    for format_name in outputs {
        let files = match format_name.as_str() {
            "css" => collet_tokens_output_css::generate(&resolved),
            "tailwind" => collet_tokens_output_tailwind::generate(&resolved),
            "ios" => collet_tokens_output_ios::generate(&resolved),
            "android" => collet_tokens_output_android::generate(&resolved),
            other => {
                eprintln!(
                    "{} Unknown output format: {other} (supported: css, tailwind, ios, android)",
                    "warning:".yellow().bold()
                );
                continue;
            }
        };

        for file in &files {
            let dest = outdir.join(&file.path);
            // Create parent directories for nested output paths (e.g., values/colors.xml).
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    CliError::io(
                        format!("Failed to create directory {}", parent.display()),
                        e,
                    )
                })?;
            }
            fs::write(&dest, &file.content)
                .map_err(|e| CliError::io(format!("Failed to write {}", dest.display()), e))?;
            let size = file.content.len();
            eprintln!(
                "{} {} ({})",
                "  Generated:".green(),
                dest.display(),
                format_size(size)
            );
            file_count += 1;
        }
    }

    eprintln!(
        "\n{} {} file(s) written to {}",
        "Done!".green().bold(),
        file_count,
        outdir.display()
    );

    Ok(())
}

/// Execute the `validate` subcommand.
fn run_validate(input: &Path) -> Result<(), CliError> {
    let contents = read_input(input)?;

    let token_input =
        collet_tokens_core::parse(&contents).map_err(|issues| CliError::Compilation { issues })?;

    let issues = collet_tokens_core::validate(&token_input);

    if issues.is_empty() {
        eprintln!("{} No issues found.", "All checks passed!".green().bold());
        return Ok(());
    }

    print_issues(&issues);

    let has_errors = issues.iter().any(Issue::is_error);
    if has_errors {
        Err(CliError::Compilation { issues })
    } else {
        let warning_count = issues.iter().filter(|i| i.is_warning()).count();
        let info_count = issues.len() - warning_count;
        eprintln!(
            "\n{} {warning_count} warning(s), {info_count} info(s).",
            "Validation passed".green().bold()
        );
        Ok(())
    }
}

/// Execute the `init` subcommand.
fn run_init() -> Result<(), CliError> {
    let path = Path::new("tokens.yaml");

    if path.exists() {
        eprintln!(
            "{} tokens.yaml already exists — not overwriting.",
            "Skipped:".yellow().bold()
        );
        return Ok(());
    }

    fs::write(path, DEFAULT_TOKENS_YAML)
        .map_err(|e| CliError::io("Failed to write tokens.yaml", e))?;

    eprintln!(
        "{} Created tokens.yaml with sensible defaults.",
        "Done!".green().bold()
    );
    eprintln!("  Edit the file, then run: collet-tokens build --input tokens.yaml");

    Ok(())
}

/// Read and return the contents of the input file.
fn read_input(path: &Path) -> Result<String, CliError> {
    fs::read_to_string(path)
        .map_err(|e| CliError::io(format!("Failed to read {}", path.display()), e))
}

/// Print issues with colored severity labels.
fn print_issues(issues: &[Issue]) {
    for issue in issues {
        let label = match issue.severity {
            Severity::Error => "error:".red().bold(),
            Severity::Warning => "warning:".yellow().bold(),
            Severity::Info => "info:".blue().bold(),
        };

        eprintln!(
            "{label} [{code}] {msg}",
            code = issue.code,
            msg = issue.message
        );
        eprintln!("       at {loc}", loc = issue.location.dimmed());

        if let Some(ref suggestion) = issue.suggestion {
            eprintln!("       {}: {suggestion}", "fix".cyan());
        }
    }
}

/// Format a byte size into a human-readable string.
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else {
        #[expect(
            clippy::cast_precision_loss,
            reason = "file sizes fit comfortably in f64"
        )]
        let kb = bytes as f64 / 1024.0;
        format!("{kb:.1}KB")
    }
}

/// Default token file content for `init` subcommand.
const DEFAULT_TOKENS_YAML: &str = r#"# Collet Tokens — Design Token Configuration
# Documentation: https://github.com/Danrozen87/collet-tokens

# ─── Fonts ───
fonts:
  display: "'Inter', system-ui, sans-serif"
  body: "'Inter', system-ui, sans-serif"
  mono: "'JetBrains Mono', ui-monospace, monospace"

# ─── Colors (oklch) ───
colors:
  surface:
    light: { l: 1.0, c: 0.0, h: 0 }
    dark: { l: 0.13, c: 0.01, h: 280 }
  surface-raised:
    light: { l: 0.985, c: 0.003, h: 90 }
    dark: { l: 0.17, c: 0.01, h: 280 }
  text-primary:
    light: { l: 0.27, c: 0.003, h: 90 }
    dark: { l: 0.87, c: 0.0, h: 0 }
  text-muted:
    light: { l: 0.45, c: 0.02, h: 75 }
    dark: { l: 0.70, c: 0.02, h: 75 }
  primary:
    light: { l: 0.55, c: 0.25, h: 264 }
    dark: { l: 0.65, c: 0.20, h: 264 }
  success:
    light: { l: 0.55, c: 0.18, h: 155 }
    dark: { l: 0.65, c: 0.15, h: 155 }
  warning:
    light: { l: 0.75, c: 0.18, h: 85 }
    dark: { l: 0.80, c: 0.15, h: 85 }
  danger:
    light: { l: 0.55, c: 0.22, h: 25 }
    dark: { l: 0.65, c: 0.18, h: 25 }

# ─── Typography ───
typography:
  scale_ratio: 1.25
  base_size: 16
  fluid_headings: true

# ─── Spacing ───
spacing:
  base: 4
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
  durations:
    fast: "100ms"
    normal: "200ms"
    smooth: "350ms"
    slow: "500ms"
  easings:
    linear: "linear"
    ease-out: "cubic-bezier(0.16, 1, 0.3, 1)"
    ease-in: "cubic-bezier(0.7, 0, 0.84, 0)"
    spring: "cubic-bezier(0.34, 1.56, 0.64, 1)"

# ─── Validation ───
validation:
  contrast_level: "AA"
  min_body_size: 16
  spacing_grid: 4
"#;
