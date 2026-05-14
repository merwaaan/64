mod build;
mod record;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Clone, Debug, strum::Display, clap::ValueEnum)]
pub enum Mode {
    /// The ROM runs its test and sends back results.
    Record,
    /// The ROM runs its test and compares its own results to embedded results recorded on hardware.
    Compare,
}

#[derive(Parser, Debug)]
#[command(
    name = "test_suite_server", // TODO
    about = "SC64 serial test message listener"// TODO
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Builds the test ROMs in either record or compare mode.
    /// Compare mode requires the test data to have been recorded beforehand.
    Build {
        #[arg(value_enum)]
        mode: Mode,

        /// Specific test name.
        /// Builds all the available tests if not specified.
        test_name: Option<String>,
    },
    /// Records the test results by executing the test ROMs
    Record {
        /// Specific test name.
        /// Records all the available tests if not specified.
        test_name: Option<String>,
        // TODO repetitions to validate determinism?
        // TODO run recorded on the same hardware to validate determinism?
    },
    /// Compares the test results by executing the compare-mode ROMs
    Compare {
        /// Specific test name.
        /// Records all the available tests if not specified.
        test_name: Option<String>,
    },
    /// Builds the record-mode ROMs, executes them to collect results and builds the compare-mode ROMs.
    All {
        /// Specific test name.
        /// Builds all the available tests if not specified.
        test_name: Option<String>,

        /// Clears the release directory to start fresh.
        #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        clear: bool,
    },
    /// Clears any previously generated files.
    Clear,
}

fn main() -> ExitCode {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let args = Args::parse();

    let result = match args.command {
        Command::Build { mode, test_name } => build::run(&mode, &test_name),
        Command::Record { test_name } => record::run(&test_name),
        Command::Compare { test_name: _ } => todo!("compare subcommand"),
        Command::All { test_name, clear } => run_all(&test_name, clear),
        Command::Clear => clear_release_dir(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_all(test_name: &Option<String>, clear: bool) -> Result<()> {
    if clear {
        clear_release_dir().context("failed to clear release directory")?;
    }

    build::run(&Mode::Record, test_name).context("failed to build record-mode ROMs")?;
    record::run(test_name).context("failed to record results on hardware")?;
    build::run(&Mode::Compare, test_name).context("failed to build compare-mode ROMs")
}

pub fn list_tests() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(rom_bin_dir())? {
        let path = entry?.path();

        if path.extension().is_some_and(|ext| ext == "rs") {
            paths.push(path);
        }
    }

    Ok(paths)
}

pub fn rom_crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-suite-rom")
}

pub fn rom_bin_dir() -> PathBuf {
    rom_crate_dir().join("src/bin")
}

pub fn rom_target_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/mips-nintendo64-none/release")
}

pub fn release_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../_test_suite_output")
}

fn clear_release_dir() -> Result<()> {
    log::info!("Clearing release directory...");

    if release_dir().is_dir() {
        fs::remove_dir_all(release_dir()).with_context(|| "failed to clear release directory")?;
    }

    Ok(())
}
