mod all;
mod build;
mod clean;
mod cli;
mod execute;
mod list;
mod upload;

use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use tracing::error;
use tracing_subscriber::{Registry, layer::SubscriberExt};
use tracing_tree::HierarchicalLayer;

use crate::cli::Cli;

fn main() -> ExitCode {
    let layer = HierarchicalLayer::default()
        .with_writer(std::io::stdout)
        .with_indent_lines(true)
        .with_indent_amount(2)
        .with_verbose_entry(false)
        .with_verbose_exit(false)
        .with_span_modes(false);

    let subscriber = Registry::default().with(layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = Cli::parse();

    let result = cli.run();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Clone, Copy, Debug, strum::Display, clap::ValueEnum)]
enum Mode {
    /// The ROM runs its test and sends back results.
    Record,
    /// The ROM runs its test and compares its own results to embedded results recorded on hardware.
    Replay,
}

#[derive(Args, Clone, Debug)]
struct SourceArgs {
    /// Matches an exact input.
    #[arg(value_name = "NAME", conflicts_with = "matches")]
    name: Option<String>,

    #[command(flatten)]
    matches: SourceMatches,
}

impl Display for SourceArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<Source>::into(self.clone()))
    }
}

#[derive(Subcommand, Clone, Debug)]
enum Source {
    All,
    Exact { name: String },
    Matching(SourceMatches),
}

impl From<SourceArgs> for Source {
    fn from(args: SourceArgs) -> Self {
        if let Some(name) = args.name {
            Source::Exact { name }
        } else if !args.matches.matches.is_empty() {
            Source::Matching(args.matches)
        } else {
            Source::All
        }
    }
}

impl Source {
    fn is_filtering(&self) -> bool {
        !matches!(self, Source::All)
    }

    fn matches(&self, test: &Test) -> bool {
        match self {
            Source::All => true,
            // Exact: match the full path or just the name
            Source::Exact { name } => test.path() == *name || test.name == *name,
            Source::Matching(matches) => matches.matches(test),
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::All => write!(f, "all"),
            Source::Exact { name } => write!(f, "exact({})", name),
            Source::Matching(matches) => write!(f, "matches({})", matches),
        }
    }
}

/// Matches inputs containing the terms.
#[derive(clap::Args, Debug, Clone, Default)]
struct SourceMatches {
    #[arg(long = "match")]
    matches: Vec<String>,
}

impl SourceMatches {
    fn matches(&self, test: &Test) -> bool {
        let path = test.path();

        self.matches.is_empty() || self.matches.iter().any(|filter| path.contains(filter))
    }
}

impl Display for SourceMatches {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matches.join(", "))
    }
}

/// A test provided by the test suite.
#[derive(Clone, Debug)]
struct Test {
    name: String,
    module: String,
}

impl Test {
    fn path(&self) -> String {
        format!("{}::{}", self.module, self.name)
    }
}

/// A set of tests to build a ROM for.
#[derive(Clone, Debug)]
enum TestSet {
    /// ROM for a single test, named after the test.
    Single { test: Test },
    /// ROM for multiple tests, using the specified name.
    Merged { tests: Vec<Test>, name: String },
}

impl TestSet {
    fn new(tests: &[Test], merge: &Option<String>) -> Vec<TestSet> {
        let mut test_sets = Vec::new();

        if let Some(merge) = merge {
            test_sets.push(TestSet::Merged {
                tests: tests.to_vec(),
                name: merge.clone(),
            });
        } else {
            for test in tests {
                test_sets.push(TestSet::Single { test: test.clone() });
            }
        }

        test_sets
    }

    fn name(&self) -> &str {
        match self {
            TestSet::Single { test } => &test.name,
            TestSet::Merged { name, .. } => name,
        }
    }

    fn paths(&self) -> Vec<String> {
        match self {
            TestSet::Single { test } => vec![test.path()],
            TestSet::Merged { tests, .. } => tests.iter().map(|test| test.path()).collect(),
        }
    }
}

impl Display for TestSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestSet::Single { test } => write!(f, "single({})", test.path()),
            TestSet::Merged { tests, .. } => write!(
                f,
                "merged({})",
                tests
                    .iter()
                    .map(|test| test.path())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// A record-mode ROM.
#[derive(Clone, Debug)]
struct RecordRom {
    test_set: TestSet,
    rom_path: PathBuf,
}

impl RecordRom {
    fn resolve(test_set: &TestSet) -> Result<Self> {
        let rom_path = release_dir().join(test_rom_name(test_set.name(), Mode::Record));

        if !rom_path.exists() {
            bail!("record ROM not found: {}", rom_path.display());
        }

        Ok(Self {
            test_set: test_set.clone(),
            rom_path,
        })
    }
}

/// The output of a record-mode ROM.
#[derive(Clone, Debug)]
struct RecordRomOutput {
    record_rom: RecordRom,
    steps_path: PathBuf,
}

impl RecordRomOutput {
    fn resolve(test_set: &TestSet) -> Result<Self> {
        let record_rom = RecordRom::resolve(test_set)?;

        let steps_path = release_dir().join(format!("{}.json", test_set.name()));

        if !steps_path.exists() {
            bail!("recorded steps not found: {}", steps_path.display());
        }

        Ok(Self {
            record_rom,
            steps_path,
        })
    }
}

/// A replay-mode ROM.
#[derive(Clone, Debug)]
struct ReplayRom {
    recorded: RecordRomOutput,
    rom_path: PathBuf,
}

fn rom_crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-suite-rom")
}

fn rom_tests_dir() -> PathBuf {
    rom_crate_dir().join("src/tests")
}

fn release_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../_test_suite")
}

fn tools_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../_tools")
}

pub const RECORD_ROM_SUFFIX: &str = ".record.z64";
pub const REPLAY_ROM_SUFFIX: &str = ".z64";

fn test_rom_name(test_name: &str, mode: Mode) -> String {
    format!(
        "{}{}",
        test_name,
        match mode {
            Mode::Record => RECORD_ROM_SUFFIX,
            Mode::Replay => REPLAY_ROM_SUFFIX,
        }
    )
}
