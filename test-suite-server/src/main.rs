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
    Replay,
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
    /// Builds the test ROMs in either record or replay mode.
    /// Replay mode requires the test data to have been recorded beforehand.
    Build {
        #[arg(value_enum)]
        mode: Mode,

        /// Specific test name.
        /// Builds all the available tests if not specified.
        test_name: Option<String>,
    },
    /// Records test results by executing the record-mode test ROMs
    Record {
        /// Specific test name.
        /// Records all the available tests if not specified.
        test_name: Option<String>,
        // TODO repetitions to validate determinism?
        // TODO run recorded on the same hardware to validate determinism?
    },
    /// Replays test results by executing the replay-mode test ROMs
    Replay {
        /// Specific test name.
        /// Records all the available tests if not specified.
        test_name: Option<String>,
    },
    /// Builds the record-mode ROMs, executes them to collect results and builds the replay-mode ROMs.
    All {
        /// Specific test name.
        /// Builds all the available tests if not specified.
        test_name: Option<String>,

        /// Deletes the release directory to start fresh.
        #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        clean: bool,
    },
    /// Deletes any previously generated files.
    Clean,
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
        Command::Replay { test_name: _ } => todo!("replay subcommand"),
        Command::All {
            test_name,
            clean: clear,
        } => run_all(&test_name, clear),
        Command::Clean => clean_release_dir(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_all(test_name: &Option<String>, clean: bool) -> Result<()> {
    if clean {
        clean_release_dir().context("failed to clear release directory")?;
    }

    build::run(&Mode::Record, test_name).context("failed to build record-mode ROMs")?;
    record::run(test_name).context("failed to record results on hardware")?;
    build::run(&Mode::Replay, test_name).context("failed to build replay-mode ROMs")
}

pub fn list_tests() -> Result<Vec<String>> {
    let mut tests = Vec::new();

    for entry in fs::read_dir(rom_tests_dir())? {
        let path = entry?.path();

        if path.extension().is_some_and(|ext| ext == "rs") {
            tests.push(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap() // TODO unwrap
                    .to_string(),
            );
        }
    }

    Ok(tests)
}

pub fn rom_crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-suite-rom")
}

pub fn rom_tests_dir() -> PathBuf {
    rom_crate_dir().join("src/tests")
}

pub fn release_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../_test_suite_output")
}

fn clean_release_dir() -> Result<()> {
    log::info!("Clearing release directory...");

    if release_dir().is_dir() {
        fs::remove_dir_all(release_dir()).with_context(|| "failed to clear release directory")?;
    }

    Ok(())
}

fn test_rom_name(test: &str, mode: Mode) -> String {
    format!("{}_{}.z64", test, mode.to_string().to_lowercase())
}

pub fn find_test_rom(test: &str, mode: Mode) -> Option<PathBuf> {
    let path = release_dir().join(test_rom_name(test, mode));

    if !path.is_file() { None } else { Some(path) }
}
