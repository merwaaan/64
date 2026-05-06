mod build;
mod record;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::Result;
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

        Command::Compare { test_name } => todo!(),

        Command::All { test_name } => {
            // TODO
            //clear_package_dir()?;
            //build::run(&Mode::Record, &test_name)?;
            //record::run(&test_name)?;
            //compare::run(&Mode::Compare, &test_name)
            todo!()
        }

        Command::Clear => clear_package_dir(),

        _ => todo!(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

pub fn package_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../_test_suite_output")
}

fn clear_package_dir() -> Result<()> {
    fs::remove_dir_all(package_dir())?;
    Ok(())
}
