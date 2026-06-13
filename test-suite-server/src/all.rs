use anyhow::{Context, Result};
use clap::Args;
use tracing::instrument;

use crate::{SourceArgs, TestSet, build::Build, clean::Clean, execute::Execute, list::List};

#[derive(Args, Debug)]
pub struct All {
    #[command(flatten)]
    source: SourceArgs,

    /// Merges the tests into a single ROM.
    #[arg(long)]
    merge: Option<String>,

    /// Executes tests multiple times and to ensure that results are deterministic.
    #[arg(long)]
    repeat: Option<usize>,

    /// Deletes the release directory to start fresh.
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    clean: bool,
}

impl All {
    #[instrument(name = "Full build", skip_all, fields(source = %self.source, merge = ?self.merge, repeat = ?self.repeat))]
    pub fn run(&self) -> Result<()> {
        if self.clean {
            Clean.run()?;
        }

        // Find the targeted tests

        let tests = List::find_tests(&self.source.clone().into())?;

        // TODO fail if empty

        let test_sets = TestSet::new(&tests, &self.merge);

        // Build record-mode ROMs

        let mut record_roms = Vec::new();

        for test_set in test_sets {
            record_roms.push(Build::record(&test_set)?);
        }

        // Collect results on hardware

        let mut record_roms_results = Vec::new();

        for record_rom in record_roms {
            let steps = Execute::record(&record_rom, self.repeat)
                .context("failed to execute record-mode ROM on hardware")?;

            record_roms_results.push(steps);
        }

        // Build replay-mode ROMs

        let mut replay_roms = Vec::new();

        for record_roms_result in record_roms_results {
            let replay_rom =
                Build::replay(&record_roms_result).context("failed to build replay-mode ROM")?;

            replay_roms.push(replay_rom);
        }

        // Execute replay-mode ROMs on the same hardware for validation

        for replay_rom in replay_roms {
            Execute::replay(&replay_rom, self.repeat)
                .context("failed to execute replay-mode ROM on hardware")?;
        }

        Ok(())
    }
}
