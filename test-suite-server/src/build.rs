use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use object::{Object, ObjectSymbol};
use test_suite_common::{Step, strip_comments};

use crate::{Mode, Test, find_test, list_tests, release_dir, rom_crate_dir};

/// Builds either a specific test ROM of all of them, in either record or replay mode.
pub fn run(mode: &Mode, test_name: &Option<String>) -> Result<()> {
    log::info!("Building tests in {mode:?} mode...");

    let tests = if let Some(test_name) = test_name {
        vec![find_test(test_name)?.ok_or_else(|| anyhow::anyhow!("no test named {test_name}"))?]
    } else {
        list_tests()?
    };

    for test in tests {
        build_test(&test, mode)?;
    }

    Ok(())
}

fn build_test(test: &Test, mode: &Mode) -> Result<()> {
    log::info!("Building test \"{}\" in {mode:?} mode...", test.name);

    // Replay mode: check that results have been recorded beforehand

    if matches!(mode, Mode::Replay) {
        let results_path = release_dir().join(format!("{}.json", test.name));

        if !results_path.is_file() {
            bail!(
                "no recorded results at {}, results must be collected by running the record-mode ROM on hardware first",
                results_path.display()
            );
        }
    }

    // We're going to make a bootable N64 ROM out of our rust program
    //
    // To do so, we simply concatenate a few parts:
    // 1 - the Libdragon IPL, which initializes the system
    // 2 - our compiled rust binary, which actually is an ELF file that will be parsed by the IPL to copy its contents into RAM
    // 3 - for replay-mode ROMs only: the recorded test results that the program will compare to its own results
    //
    // This is adapted from https://github.com/rust-n64/nust64

    let mut z64 = Vec::new();

    // Build the test

    log::debug!("  Building test...");

    let build_result = duct::cmd!(
        "cargo",
        "build",
        "--release",
        "--no-default-features",
        "--features",
        match mode {
            Mode::Record => "record",
            Mode::Replay => "replay",
        },
    )
    .env("TEST_MODULE", &test.module)
    .env("TEST_NAME", &test.name)
    .env_remove("RUSTUP_TOOLCHAIN") // Scrub the current crate's toolchain TODO not needed if separate proj?
    .dir(rom_crate_dir())
    .stderr_capture()
    .unchecked()
    .run()?;

    if !build_result.status.success() {
        let stderr = String::from_utf8_lossy(&build_result.stderr);
        bail!("build failed: {stderr}");
    }

    // Add the Libdragon IPL
    // TODO download helper

    const IPL: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../ipl3_prod.z64"));

    log::debug!("  Adding Libdragon IPL to ROM ({:0X?} bytes)...", IPL.len());

    z64.extend(IPL);

    // Add some padding, IPL3 looks for 256-byte aligned blocks of memory

    let pad = |z64: &mut Vec<u8>, alignment: usize| {
        let misalignment = (alignment - (z64.len() % alignment)) % alignment;

        if misalignment > 0 {
            z64.extend(&vec![0x00; misalignment]);

            log::debug!(
                "    Aligned to {:0X?} with {:0X?} padding bytes (total size: {:0X?})",
                alignment,
                misalignment,
                z64.len()
            );
        }
    };

    pad(&mut z64, 256);

    // Add the test binary

    let program_offset = z64.len();

    let program_path = format!(
        "{}/../target/mips-nintendo64-none/release/test_suite_rom",
        env!("CARGO_MANIFEST_DIR")
    );

    let program = fs::read(program_path)?;

    log::debug!("  Adding test ELF to ROM ({:0X?} bytes)...", program.len());

    z64.extend(&program);

    // Add some padding

    pad(&mut z64, 16);

    // Add the embedded test results

    if matches!(mode, Mode::Replay) {
        log::debug!("  Adding embedded test results to ROM...");

        // Load the JSON steps

        let results_path = release_dir().join(format!("{}.json", test.name));

        let results_string = fs::read_to_string(&results_path).with_context(|| {
            format!(
                "failed to read test results from {}",
                results_path.display()
            )
        })?;

        let steps: Vec<Step> = serde_json::from_str(&results_string).with_context(|| {
            format!(
                "failed to parse JSON test results from {}",
                results_path.display()
            )
        })?;

        // Strip the comments to save memory

        let steps = strip_comments(&steps);

        // Serialize the steps to a binary buffer
        // (each step one after another, not the vector, to allow streaming without loading the whole vector in the program)

        let embedded_data_offset = z64.len() as u32;

        let mut steps_data = Vec::new();

        for step in &steps {
            steps_data.extend(postcard::to_allocvec(&step)?);
        }

        log::debug!(
            "    Serialized {} steps to {:0X?} bytes",
            steps.len(),
            steps_data.len()
        );

        z64.extend_from_slice(&steps_data);

        // Add some padding

        pad(&mut z64, 16);

        // For the program to be able to access the embedded data, it needs to know the offset and size of that data in the final ROM.
        // There are symbols exposed in the ELF file for those, we're going to patch them with the actual values.

        let program_elf = object::File::parse(program.as_slice())?;

        for (symbol_name, patch_value) in [
            ("EMBEDDED_DATA_ROM_OFFSET", embedded_data_offset),
            ("EMBEDDED_DATA_ROM_SIZE", steps_data.len() as u32),
        ] {
            let patch_rom_offset = program_elf
                .symbols()
                .find(|s| s.name() == Ok(symbol_name))
                .ok_or_else(|| anyhow!("{symbol_name} symbol not found"))?
                .address() as usize
                & 0x1FFF_FFFF; // linked as in KSEG0 but we need a base offset

            let z64_patch_offset = program_offset + patch_rom_offset;

            z64[z64_patch_offset..z64_patch_offset + 4].copy_from_slice(&patch_value.to_be_bytes());

            log::debug!(
                "    Patched {symbol_name} at {:08X?} to {:08X?}",
                z64_patch_offset,
                patch_value
            );
        }
    }

    // Output the ROM to the release directory

    fs::create_dir_all(release_dir())?;

    let release_dir_path = release_dir().join(format!(
        "{}_{}.z64",
        test.name,
        mode.to_string().to_lowercase()
    ));

    fs::write(&release_dir_path, &z64)?;

    log::info!(
        "  -> {} ({:0X?} bytes)",
        release_dir_path.display(),
        z64.len()
    );

    Ok(())
}
