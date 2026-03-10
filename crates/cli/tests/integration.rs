//! Integration tests for the collet-tokens CLI.

use std::path::PathBuf;
use std::process::Command;

/// Path to the workspace root (two levels up from the cli crate).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root should exist")
        .to_owned()
}

/// Path to a test fixture file.
fn fixture(name: &str) -> PathBuf {
    workspace_root().join("tests/fixtures").join(name)
}

/// Run the `collet-tokens` binary with the given arguments.
fn collet_tokens(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_collet-tokens"))
        .args(args)
        .output()
        .expect("failed to execute collet-tokens binary")
}

#[test]
fn validate_valid_minimal_succeeds() {
    let output = collet_tokens(&[
        "validate",
        "--input",
        fixture("valid-minimal.yaml").to_str().expect("valid path"),
    ]);

    assert!(
        output.status.success(),
        "validate should succeed for valid-minimal.yaml, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn validate_invalid_contrast_fails() {
    let output = collet_tokens(&[
        "validate",
        "--input",
        fixture("invalid-contrast.yaml")
            .to_str()
            .expect("valid path"),
    ]);

    assert!(
        !output.status.success(),
        "validate should fail for invalid-contrast.yaml"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("contrast"),
        "error output should mention contrast, got: {stderr}"
    );
}

#[test]
fn build_valid_full_produces_output_files() {
    let outdir = tempfile::tempdir().expect("create temp dir");
    let outdir_path = outdir.path();

    let output = collet_tokens(&[
        "build",
        "--input",
        fixture("valid-full.yaml").to_str().expect("valid path"),
        "--outdir",
        outdir_path.to_str().expect("valid path"),
        "--output",
        "css,tailwind",
    ]);

    assert!(
        output.status.success(),
        "build should succeed for valid-full.yaml, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that output files were created
    assert!(
        outdir_path.join("tokens.css").exists(),
        "tokens.css should be generated"
    );

    assert!(
        outdir_path.join("tailwind.config.ts").exists(),
        "tailwind.config.ts should be generated"
    );

    // Verify tokens.css is non-empty and contains expected content
    let css = std::fs::read_to_string(outdir_path.join("tokens.css")).expect("read tokens.css");
    assert!(css.contains(":root {"), "tokens.css should contain :root");
    assert!(
        css.contains("--font-display:"),
        "tokens.css should contain font vars"
    );
    assert!(
        css.contains("--color-primary:"),
        "tokens.css should contain color vars"
    );
    assert!(
        css.contains("--space-"),
        "tokens.css should contain spacing vars"
    );
    assert!(
        css.contains("--radius-"),
        "tokens.css should contain radius vars"
    );
    assert!(
        css.contains("--duration-"),
        "tokens.css should contain motion duration vars"
    );
    assert!(
        css.contains("--ease-"),
        "tokens.css should contain motion easing vars"
    );
}
