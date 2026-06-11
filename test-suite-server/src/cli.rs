use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{all::All, build::Build, clean::Clean, execute::Execute, list::List, upload::Upload};

#[derive(Subcommand, Debug)]
enum Command {
    /// Lists all the tests.
    List(List),
    /// Builds the test ROMs in either record or replay mode.
    ///
    /// Replay mode requires the test data to have been recorded beforehand.
    Build(Build),
    /// Uploads a test ROM to the SC64.
    Upload(Upload),
    /// Executes test on hardware.
    Execute(Execute),
    /// Builds the record-mode ROMs, executes them to collect results and builds the replay-mode ROMs.
    All(All),
    /// Deletes the output directory.
    Clean(Clean),
}

#[derive(Parser, Debug)]
#[command(about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::List(list) => list.run(),
            Command::Build(build) => build.run().map(|_| ()),
            Command::Upload(upload) => upload.run(),
            Command::Execute(record) => record.run(),
            Command::All(all) => all.run(),
            Command::Clean(clean) => clean.run(),
        }
    }
}
