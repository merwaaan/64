mod build;
mod record;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use regex::Regex;

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
    /// Lists all the available tests.
    List {
        #[command(flatten)]
        filter: TestFilter,
    },
    /// Builds the test ROMs in either record or replay mode.
    /// Replay mode requires the test data to have been recorded beforehand.
    Build {
        #[arg(value_enum)]
        mode: Mode,

        #[command(flatten)]
        filter: TestFilter,
    },
    /// Records test results by executing the record-mode test ROMs
    Record {
        #[command(flatten)]
        filter: TestFilter,

        /// Records multiple times to ensure that the test is deterministic.
        #[arg(long)]
        repeat: Option<usize>,
    },
    /// Replays test results by executing the replay-mode test ROMs
    Replay {
        #[command(flatten)]
        filter: TestFilter,
    },
    /// Builds the record-mode ROMs, executes them to collect results and builds the replay-mode ROMs.
    All {
        #[command(flatten)]
        filter: TestFilter,

        /// Records multiple times to ensure that the test is deterministic.
        #[arg(long)]
        repeat: Option<usize>,

        /// Deletes the release directory to start fresh.
        #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        clean: bool,
    },
    /// Deletes any previously generated files.
    Clean,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct TestFilter {
    /// Test name filters.
    ///
    /// Only considers tests where `module::name` contains one of the filters if specified.
    /// Considers all the available tests if not specified.
    #[arg(long = "filter", short = 'f')]
    pub filters: Vec<String>,
}

impl TestFilter {
    pub fn matches(&self, test: &Test) -> bool {
        let full_name = format!("{}::{}", test.module, test.name);

        self.filters.is_empty() || self.filters.iter().any(|filter| full_name.contains(filter))
    }
}

fn main() -> ExitCode {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .format_timestamp(None)
        .init();

    let args = Args::parse();

    let result = match args.command {
        Command::List { filter } => show_test_list(&filter),
        Command::Build { mode, filter } => build::run(&mode, &filter),
        Command::Record { filter, repeat } => record::run(&filter, repeat),
        Command::Replay { .. } => todo!("replay subcommand"),
        Command::All {
            filter,
            repeat,
            clean: clear,
        } => run_all(&filter, repeat, clear),
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

fn show_test_list(filter: &TestFilter) -> Result<()> {
    let tests = list_tests(filter)?;

    log::info!("{} tests:", tests.len());

    for test in tests {
        log::info!("- {}:{}", test.module, test.name);
    }

    Ok(())
}

fn run_all(filter: &TestFilter, repeat: Option<usize>, clean: bool) -> Result<()> {
    if clean {
        clean_release_dir().context("failed to clear release directory")?;
    }

    build::run(&Mode::Record, filter).context("failed to build record-mode ROMs")?;
    record::run(filter, repeat).context("failed to record results on hardware")?;
    build::run(&Mode::Replay, filter).context("failed to build replay-mode ROMs")

    // TODO replay recording on same hardware to validate
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

fn test_rom_name(test_name: &str, mode: Mode) -> String {
    format!("{}_{}.z64", test_name, mode.to_string().to_lowercase())
}

#[derive(Clone, Debug)]
pub struct Test {
    name: String,
    module: String,
}

/// Returns all the tests registered via the `register_test!` macro.
pub fn list_tests(filter: &TestFilter) -> Result<Vec<Test>> {
    let mut tests = Vec::new();

    let register_test_regex =
        Regex::new(r"(?m)^\s*register_test!\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)")?;

    for entry in fs::read_dir(rom_tests_dir())? {
        let path = entry?.path();

        if path.extension().is_some_and(|ext| ext == "rs") {
            let module = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("non-utf8 test file name")?
                .to_string();

            let source = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;

            for capture in register_test_regex.captures_iter(&source) {
                let test = Test {
                    name: capture[1].to_string(),
                    module: module.clone(),
                };

                if filter.matches(&test) {
                    tests.push(test);
                }
            }
        }
    }

    if !filter.filters.is_empty() && tests.is_empty() {
        log::warn!("No tests matched filters: {}", filter.filters.join(", "));
    }

    Ok(tests)
}

pub fn find_test_rom(test_name: &str, mode: Mode) -> Option<PathBuf> {
    let path = release_dir().join(test_rom_name(test_name, mode));

    if !path.is_file() { None } else { Some(path) }
}
