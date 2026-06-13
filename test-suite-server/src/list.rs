use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use regex::Regex;
use tracing::{info, instrument, warn};

use crate::{
    Mode, RECORD_ROM_SUFFIX, REPLAY_ROM_SUFFIX, Source, SourceArgs, SourceMatches, Test,
    release_dir, rom_tests_dir,
};

#[derive(Args, Debug)]
pub struct List {
    #[command(flatten)]
    pub source: SourceArgs,
}

impl List {
    #[instrument(name = "List tests", skip_all, fields(source = %self.source))]
    pub fn run(&self) -> Result<()> {
        let source = self.source.clone().into();

        let tests = Self::find_tests(&source)?;

        info!(
            "{} tests{}",
            tests.len(),
            if source.is_filtering() {
                format!(" (source: {})", source)
            } else {
                "".to_string()
            }
        );

        for test in tests {
            info!("- {}", test.path());
        }

        Ok(())
    }

    /// Finds all the matching tests registered via the `register_test!` macro.
    pub fn find_tests(source: &Source) -> Result<Vec<Test>> {
        let mut tests = Vec::new();

        // List the test modules

        let mut modules = Vec::new();

        for entry in fs::read_dir(rom_tests_dir())? {
            let path = entry?.path();

            if !matches!(path.extension(), Some(ext) if ext == "rs") {
                continue;
            }

            let module = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("failed to get test file name")?
                .to_string();

            modules.push((module, path));
        }

        // fs::read_dir() seems to iterate through files in alphabetical order but this is not actually guaranteed,
        // so let's enforce it explicitly

        modules.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Read the modules and extract the tests registered with the macro
        // TODO something more robust? expanded macro? more sophisticated?

        let register_test_regex =
            Regex::new(r"(?m)^\s*register_test!\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)")?;

        for (module, module_path) in modules {
            let module_contents = fs::read_to_string(&module_path)
                .with_context(|| format!("failed to read {}", module_path.display()))?;

            for capture in register_test_regex.captures_iter(&module_contents) {
                let test = Test {
                    name: capture[1].to_string(),
                    module: module.clone(),
                };

                // Only keep matching tests

                if source.matches(&test) {
                    tests.push(test);
                }
            }
        }

        Ok(tests)
    }

    /// Finds all the matching test ROMs.
    #[instrument(name = "Find test ROMs", level = "debug", skip_all, fields(source = %source, mode = %mode))]
    pub fn find_roms(source: &Source, mode: Mode) -> Result<Vec<PathBuf>> {
        let mut roms = Vec::new();

        if !release_dir().is_dir() {
            return Ok(roms);
        }

        for entry in fs::read_dir(release_dir())? {
            let path = entry?.path();

            let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            let suffix = match mode {
                Mode::Record => RECORD_ROM_SUFFIX,
                Mode::Replay => REPLAY_ROM_SUFFIX,
            };

            if !file_name.ends_with(suffix) {
                continue;
            }

            // Replay: also exclude *.record.z64 as it ends with .z64 too
            if matches!(mode, Mode::Replay) && file_name.ends_with(RECORD_ROM_SUFFIX) {
                continue;
            }

            let Some(stem) = file_name.strip_suffix(suffix) else {
                continue;
            };

            let matches = match source {
                Source::All => true,
                Source::Exact { name } => stem == name,
                Source::Matching(SourceMatches { matches }) => {
                    matches.iter().any(|m| stem.contains(m))
                }
            };

            if matches {
                roms.push(path);
            }
        }

        Ok(roms)
    }
}
