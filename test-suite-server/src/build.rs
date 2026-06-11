use std::{fmt::Debug, fs, io, path::PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use object::{Object, ObjectSection, ObjectSymbol};
use test_suite_common::Step;
use tracing::{debug, debug_span, info, instrument, warn};

use crate::{
    Mode, RecordRom, RecordRomOutput, ReplayRom, SourceArgs, TestSet, list::List, release_dir,
    rom_crate_dir, test_rom_name, tools_dir,
};

#[derive(Args, Debug)]
pub struct Build {
    #[arg(long, value_enum)]
    pub mode: Mode,

    #[command(flatten)]
    pub source: SourceArgs,

    /// Merges the tests into a single ROM.
    ///
    /// Otherwise, builds a separate ROM for each test.
    #[arg(long)]
    pub merge: Option<String>,
}

impl Build {
    #[instrument(name = "Build tests", skip_all, fields(mode = %self.mode, source = %self.source, merge = ?self.merge))]
    pub fn run(&self) -> Result<()> {
        let source = self.source.clone().into();

        let tests = List::find_tests(&source)?;

        if tests.is_empty() {
            if source.is_filtering() {
                bail!("no matching tests for {source}")
            } else {
                bail!("no matching tests")
            }
        }

        let test_sets = TestSet::resolve(&tests, &self.merge);

        match self.mode {
            Mode::Record => {
                for test_set in test_sets {
                    Self::record(&test_set)?;
                }
            }
            Mode::Replay => {
                for test_set in test_sets {
                    let record_rom_output = RecordRomOutput::resolve(&test_set)?;

                    info!("record_rom_output: {:?}", record_rom_output);
                    Self::replay(&record_rom_output)?;
                }
            }
        }

        Ok(())
    }

    pub fn record(test_set: &TestSet) -> Result<RecordRom> {
        let path = Self::build_rom(&BuildSource::Record(test_set.clone()))?;

        Ok(RecordRom {
            test_set: test_set.clone(),
            rom_path: path,
        })
    }

    pub fn replay(record_rom_output: &RecordRomOutput) -> Result<ReplayRom> {
        let path = Self::build_rom(&BuildSource::Replay(record_rom_output.clone()))?;

        Ok(ReplayRom {
            recorded: record_rom_output.clone(),
            rom_path: path,
        })
    }

    #[instrument(name = "Build test", skip_all, fields(source = ?source))]
    fn build_rom(source: &BuildSource) -> Result<PathBuf> {
        // We're going to make a bootable N64 ROM out of our rust program
        //
        // To do so, we concatenate a few parts:
        // 1 - the Libdragon IPL3, which initializes the system
        // 2 - our compiled rust binary, which actually is an ELF file that will be parsed by the IPL to copy its contents into RAM
        // 3 - for replay-mode ROMs only: the recorded test results that the program will compare to its own results
        //
        // This is adapted from https://github.com/rust-n64/nust64

        let mut z64 = Vec::new();

        // Helper to pad the ROM to a given alignment

        let pad = |z64: &mut Vec<u8>, alignment: usize| {
            let misalignment = (alignment - (z64.len() % alignment)) % alignment;

            if misalignment > 0 {
                z64.extend(&vec![0x00; misalignment]);

                debug!(
                    "Aligned to {:0X?} with {:0X?} padding bytes (total size: {:0X?})",
                    alignment,
                    misalignment,
                    z64.len()
                );
            }
        };

        // Add the Libdragon IPL

        debug_span!("Add IPL3 to ROM").in_scope(|| -> Result<()> {
            let ipl_path = ipl3_path()?;

            debug!("path: {}", ipl_path.display());

            let ipl = fs::read(&ipl_path)?;

            debug!("size: {:0X?} bytes", ipl.len());

            z64.extend(ipl);

            // Add some padding, IPL3 scans 256-byte aligned blocks of memory

            pad(&mut z64, 256);

            Ok(())
        })?;

        // Build the ELF

        debug_span!("Build test program").in_scope(|| -> Result<_> {
            let test_paths = source.paths().join(",");

            debug!("TEST_PATHS: {}", test_paths);

            let build_result = duct::cmd!(
                "cargo",
                "build",
                "--release",
                "--no-default-features",
                "--features",
                match source {
                    BuildSource::Record(_) => "record",
                    BuildSource::Replay(_) => "replay",
                },
            )
            .env("TEST_PATHS", &test_paths)
            .env_remove("RUSTUP_TOOLCHAIN") // Scrub the current crate's toolchain TODO not needed if separate proj?
            .dir(rom_crate_dir())
            .stderr_capture()
            .unchecked()
            .run()?;

            if !build_result.status.success() {
                let stderr = String::from_utf8_lossy(&build_result.stderr);
                bail!("build failed: {stderr}");
            }

            Ok(())
        })?;

        // Add the test program

        let program_offset = z64.len();

        let program = debug_span!("Add test program to ROM").in_scope(|| -> Result<_> {
            let program_path = format!(
                "{}/../target/mips-nintendo64-none/release/test_suite_rom", // TODO any way t o specify the output name to avoid mixups?
                env!("CARGO_MANIFEST_DIR")
            );

            debug!("path: {}", program_path);

            let program = fs::read(program_path)?;

            debug!("size: {:0X?} bytes", program.len());

            z64.extend(&program);

            // Add some padding

            pad(&mut z64, 16);

            Ok(program)
        })?;

        // Add the embedded test results

        if let BuildSource::Replay(RecordRomOutput { steps_path, .. }) = source {
            debug_span!("Add recorded test results to ROM").in_scope(|| -> Result<_> {
                // Load the JSON steps

                let results_string = fs::read_to_string(&steps_path).with_context(|| {
                    format!("failed to read test results from {}", steps_path.display())
                })?;

                let steps: Vec<Step> =
                    serde_json::from_str(&results_string).with_context(|| {
                        format!(
                            "failed to parse JSON test results from {}",
                            steps_path.display()
                        )
                    })?;

                // Serialize the steps to a binary buffer
                // (each step one after another, not the vector, to allow streaming without loading the whole vector in the program)

                let embedded_data_offset = z64.len() as u32;

                let mut steps_data = Vec::new();

                for step in &steps {
                    steps_data.extend(postcard::to_allocvec(&step)?);
                }

                debug!("steps: {}", steps.len());

                debug!("size: {:0X?} bytes", steps_data.len());

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
                    let symbol = program_elf
                        .symbol_by_name(symbol_name)
                        .ok_or_else(|| anyhow!("{symbol_name} symbol not found"))?;

                    let section_index = symbol
                        .section_index()
                        .ok_or_else(|| anyhow!("{symbol_name} has no section index"))?;

                    let section = program_elf.section_by_index(section_index)?;

                    let (section_file_offset, _section_file_size) = section
                        .file_range()
                        .ok_or_else(|| anyhow!("{symbol_name} section has no file range"))?;

                    let in_section_offset = symbol
                        .address()
                        .checked_sub(section.address())
                        .ok_or_else(|| anyhow!("{symbol_name} address is before section start"))?
                        as usize;

                    let patch_rom_offset = section_file_offset as usize + in_section_offset;

                    let z64_patch_offset = program_offset + patch_rom_offset;

                    z64[z64_patch_offset..z64_patch_offset + 4]
                        .copy_from_slice(&patch_value.to_be_bytes());

                    debug!(
                        "Patched {symbol_name} at {:08X?} to {:08X?}",
                        z64_patch_offset, patch_value
                    );
                }

                Ok(())
            })?;
        }

        // Output the ROM to the release directory

        fs::create_dir_all(release_dir())?;

        let rom_path = release_dir().join(source.test_rom_name());

        fs::write(&rom_path, &z64)?;

        info!("💾 {} ({:0X?} bytes)", rom_path.display(), z64.len());

        Ok(rom_path)
    }
}

#[derive(Clone, Debug)]
enum BuildSource {
    Record(TestSet),
    Replay(RecordRomOutput),
}

impl BuildSource {
    fn paths(&self) -> Vec<String> {
        match self {
            BuildSource::Record(test_set) => test_set.paths(),
            BuildSource::Replay(record_rom_output) => record_rom_output.record_rom.test_set.paths(),
        }
    }

    fn test_rom_name(&self) -> String {
        match self {
            BuildSource::Record(test_set) => test_rom_name(test_set.name(), Mode::Record),
            BuildSource::Replay(record_rom_output) => {
                test_rom_name(record_rom_output.record_rom.test_set.name(), Mode::Replay)
            }
        }
    }
}

fn ipl3_path() -> Result<PathBuf> {
    let path = tools_dir().join("ipl3_prod.z64");

    if !path.exists() {
        download_ipl3(&path)?;
    }

    Ok(path)
}

#[instrument(name = "Download IPL3", level = "debug", skip_all)]
fn download_ipl3(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let url = "https://github.com/DragonMinded/libdragon/raw/07f1977bbb66a8f61d949983342c27915932d5a5/boot/bin/ipl3_prod.z64";

    debug!("url: {url}...");

    let response = ureq::get(url).call()?;

    io::copy(
        &mut response.into_body().as_reader(),
        &mut fs::File::create(&path)?,
    )?;

    Ok(())
}
