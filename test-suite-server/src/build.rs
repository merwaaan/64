use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, anyhow, bail};

use crate::{Mode, package_dir};

pub fn run(mode: &Mode, test_name: &Option<String>) -> Result<()> {
    // Use the provided test or list all the available tests

    let mut test_paths = Vec::new();

    if let Some(test_name) = test_name {
        let path = rom_bin_dir().join(format!("{test_name}.rs"));

        if !path.is_file() {
            bail!("no test source at {}", path.display());
        }

        test_paths.push(path);
    } else {
        test_paths.extend(list_tests()?);
    }

    // Build each test

    for path in test_paths {
        build_test(mode, &path)?;
    }

    Ok(())
}

fn rom_crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-suite-rom")
}

fn rom_bin_dir() -> PathBuf {
    rom_crate_dir().join("src/bin")
}

fn rom_target_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/mips-nintendo64-none/release")
}

fn list_tests() -> Result<Vec<PathBuf>> {
    log::info!("Listing all tests...");

    let bin_dir = rom_bin_dir();

    let mut paths = Vec::new();

    for entry in fs::read_dir(&bin_dir)? {
        let path = entry?.path();

        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            paths.push(path);
        }
    }

    paths.sort_by_key(|p| p.to_string_lossy().into_owned()); // TODO needed?

    log::info!("Found {} tests:", paths.len());

    for path in &paths {
        log::info!("  - {}", path.display());
    }

    Ok(paths)
}

fn build_test(mode: &Mode, test_path: &Path) -> Result<()> {
    let test_name = test_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("test path has no file name: {}", test_path.display()))?;

    log::info!("Building test \"{test_name}\"...");

    // Compare mode: check that results have been recorded beforehand

    if matches!(mode, Mode::Compare) {
        let results_path = package_dir().join(format!("{test_name}.json"));

        if !results_path.is_file() {
            bail!(
                "no recorded results at {}, results must be collected by running the record-mode ROM on hardware first",
                results_path.display()
            );
        }
    }

    // Build

    // TODO use duct
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--bin")
        .arg(test_name)
        .arg("--no-default-features")
        .arg("--features")
        .arg(match mode {
            Mode::Record => "record",
            Mode::Compare => "compare",
        })
        .arg("--message-format=short") // Shorter errors
        .env_remove("RUSTUP_TOOLCHAIN") // Scrub the current crate's toolchain
        .current_dir(rom_crate_dir())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("build failed: {stderr}");
    }

    // Copy to the output directory

    let target_path = rom_target_dir().join(format!("{test_name}.z64"));

    if !target_path.is_file() {
        bail!("no output ROM at {}", target_path.display());
    }

    fs::create_dir_all(&package_dir())?;

    let packaged_path = package_dir().join(format!(
        "{test_name}_{}.z64",
        mode.to_string().to_lowercase()
    ));

    fs::copy(&target_path, &packaged_path)?;

    log::info!("  -> {}", packaged_path.display());

    Ok(())
}
