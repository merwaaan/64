use std::{fs, path::Path, process::Command};

use anyhow::{Result, anyhow, bail};

use crate::{Mode, list_tests, package_dir, rom_bin_dir, rom_crate_dir, rom_target_dir};

pub fn run(mode: &Mode, test_name: &Option<String>) -> Result<()> {
    log::info!("Building tests in {mode:?} mode...");

    // Use the provided test or list all the available tests

    let test_paths = if let Some(test_name) = test_name {
        let path = rom_bin_dir().join(format!("{test_name}.rs"));

        if !path.is_file() {
            bail!("no test source for {test_name}");
        }

        vec![path]
    } else {
        list_tests()?
    };

    // Build each test

    for path in test_paths {
        build_test(mode, &path)?;
    }

    Ok(())
}

fn build_test(mode: &Mode, test_path: &Path) -> Result<()> {
    let test_name = test_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("test path has no file name: {}", test_path.display()))?;

    log::info!("Building test \"{test_name}\" in {mode:?} mode...");

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

    fs::create_dir_all(package_dir())?;

    let packaged_path = package_dir().join(format!(
        "{test_name}_{}.z64",
        mode.to_string().to_lowercase()
    ));

    fs::copy(&target_path, &packaged_path)?;

    log::info!("  -> {}", packaged_path.display());

    Ok(())
}
