use std::fs;

/// Builds the test ROM for the tests specified in the `TEST_PATHS` environment variable.
fn main() {
    println!("cargo:rerun-if-env-changed=TEST_PATHS");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RECORD");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_REPLAY");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");

    let test_paths = std::env::var("TEST_PATHS").unwrap_or("Dummy::Dummy".to_string()); //.expect("TEST_PATHS environment variable is not set");

    // if test_paths.is_empty() {
    //     panic!("TEST_PATHS environment variable is empty");
    // }

    // Generate the test plan's code

    let test_paths: Vec<&str> = test_paths.split(',').collect();

    let test_count = test_paths.len();

    let test_runs = test_paths
        .iter()
        .map(|path| {
            format!(
                "if !app.run_test::<{}>()? {{ failed_tests.push({}::name()); }}",
                path, path
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let test_case_counts = test_paths
        .iter()
        .map(|path| format!("count += {}::cases().count();", path))
        .collect::<Vec<_>>()
        .join("\n    ");

    let code = format!(
        "
use anyhow::Result;
use alloc::vec::Vec;

use crate::test::Test;

/// Run all the selected tests.
/// Returns the names of the tests that failed.
pub fn run_tests(app: &mut crate::app::App) -> Result<Vec<&'static str>> {{
    let mut failed_tests = Vec::new();

    {test_runs}

    Ok(failed_tests)
}}

pub fn test_count() -> usize {{
    {test_count}
}}

pub fn test_case_count() -> usize {{
    let mut count = 0;

    {test_case_counts}

    count
}}"
    );

    fs::write(
        format!("{}/plan.rs", std::env::var("OUT_DIR").unwrap()),
        code,
    )
    .expect("failed to generate test module"); // TODO err msg
}
