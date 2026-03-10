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

#[test]
fn build_ios_output() {
    let outdir = tempfile::tempdir().expect("create temp dir");
    let outdir_path = outdir.path();

    let output = collet_tokens(&[
        "build",
        "--input",
        fixture("valid-full.yaml").to_str().expect("valid path"),
        "--outdir",
        outdir_path.to_str().expect("valid path"),
        "--output",
        "ios",
    ]);

    assert!(
        output.status.success(),
        "build should succeed with ios output, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check Swift files were created.
    assert!(
        outdir_path.join("Colors.swift").exists(),
        "Colors.swift should be generated"
    );
    assert!(
        outdir_path.join("Typography.swift").exists(),
        "Typography.swift should be generated"
    );

    // Verify Colors.swift contains UIColor extension.
    let colors =
        std::fs::read_to_string(outdir_path.join("Colors.swift")).expect("read Colors.swift");
    assert!(
        colors.contains("extension UIColor {"),
        "Colors.swift should contain UIColor extension"
    );
    assert!(
        colors.contains("userInterfaceStyle == .dark"),
        "Colors.swift should have dark mode handling"
    );

    // Verify Typography.swift contains type roles.
    let typo = std::fs::read_to_string(outdir_path.join("Typography.swift"))
        .expect("read Typography.swift");
    assert!(
        typo.contains("struct TypographyRole"),
        "Typography.swift should contain TypographyRole"
    );
    assert!(
        typo.contains("enum DesignTypography"),
        "Typography.swift should contain DesignTypography enum"
    );
}

#[test]
fn build_android_output() {
    let outdir = tempfile::tempdir().expect("create temp dir");
    let outdir_path = outdir.path();

    let output = collet_tokens(&[
        "build",
        "--input",
        fixture("valid-full.yaml").to_str().expect("valid path"),
        "--outdir",
        outdir_path.to_str().expect("valid path"),
        "--output",
        "android",
    ]);

    assert!(
        output.status.success(),
        "build should succeed with android output, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check XML files were created.
    assert!(
        outdir_path.join("values/colors.xml").exists(),
        "values/colors.xml should be generated"
    );
    assert!(
        outdir_path.join("values-night/colors.xml").exists(),
        "values-night/colors.xml should be generated"
    );
    assert!(
        outdir_path.join("values/dimens.xml").exists(),
        "values/dimens.xml should be generated"
    );
    assert!(
        outdir_path.join("values/type.xml").exists(),
        "values/type.xml should be generated"
    );

    // Verify light colors.xml is valid XML structure.
    let colors =
        std::fs::read_to_string(outdir_path.join("values/colors.xml")).expect("read colors.xml");
    assert!(
        colors.contains("<resources>"),
        "colors.xml should have resources tag"
    );
    assert!(
        colors.contains("<color name="),
        "colors.xml should have color entries"
    );

    // Verify night colors differ from day colors.
    let night = std::fs::read_to_string(outdir_path.join("values-night/colors.xml"))
        .expect("read night colors.xml");
    assert!(
        night.contains("<color name="),
        "night colors.xml should have color entries"
    );
    assert_ne!(colors, night, "light and dark color files should differ");

    // Verify dimens.xml has spacing and radius.
    let dimens =
        std::fs::read_to_string(outdir_path.join("values/dimens.xml")).expect("read dimens.xml");
    assert!(
        dimens.contains("<dimen name="),
        "dimens.xml should have dimen entries"
    );

    // Verify type.xml has typography styles.
    let types =
        std::fs::read_to_string(outdir_path.join("values/type.xml")).expect("read type.xml");
    assert!(
        types.contains("<style name=\"TextAppearance_"),
        "type.xml should have TextAppearance styles"
    );
}

#[test]
fn build_all_formats() {
    let outdir = tempfile::tempdir().expect("create temp dir");
    let outdir_path = outdir.path();

    let output = collet_tokens(&[
        "build",
        "--input",
        fixture("valid-full.yaml").to_str().expect("valid path"),
        "--outdir",
        outdir_path.to_str().expect("valid path"),
        "--output",
        "css,tailwind,ios,android",
    ]);

    assert!(
        output.status.success(),
        "build should succeed with all formats, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // All format files should exist.
    assert!(outdir_path.join("tokens.css").exists());
    assert!(outdir_path.join("tailwind.config.ts").exists());
    assert!(outdir_path.join("Colors.swift").exists());
    assert!(outdir_path.join("Typography.swift").exists());
    assert!(outdir_path.join("values/colors.xml").exists());
    assert!(outdir_path.join("values-night/colors.xml").exists());
    assert!(outdir_path.join("values/dimens.xml").exists());
    assert!(outdir_path.join("values/type.xml").exists());
}

#[test]
fn build_dtcg_input() {
    let outdir = tempfile::tempdir().expect("create temp dir");
    let outdir_path = outdir.path();

    let output = collet_tokens(&[
        "build",
        "--input",
        fixture("valid-dtcg.tokens.json")
            .to_str()
            .expect("valid path"),
        "--outdir",
        outdir_path.to_str().expect("valid path"),
        "--output",
        "css",
    ]);

    assert!(
        output.status.success(),
        "build should succeed with DTCG input, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Should produce tokens.css from DTCG format.
    assert!(
        outdir_path.join("tokens.css").exists(),
        "tokens.css should be generated from DTCG input"
    );

    let css = std::fs::read_to_string(outdir_path.join("tokens.css")).expect("read tokens.css");
    assert!(css.contains(":root {"), "tokens.css should contain :root");
    assert!(
        css.contains("--color-"),
        "tokens.css should contain color vars from DTCG"
    );
}

#[test]
fn build_dtcg_with_ios_android() {
    let outdir = tempfile::tempdir().expect("create temp dir");
    let outdir_path = outdir.path();

    let output = collet_tokens(&[
        "build",
        "--input",
        fixture("valid-dtcg.tokens.json")
            .to_str()
            .expect("valid path"),
        "--outdir",
        outdir_path.to_str().expect("valid path"),
        "--output",
        "ios,android",
    ]);

    assert!(
        output.status.success(),
        "build should succeed with DTCG + ios/android, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(outdir_path.join("Colors.swift").exists());
    assert!(outdir_path.join("values/colors.xml").exists());
}
