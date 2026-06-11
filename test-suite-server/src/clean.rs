use std::fs;

use anyhow::{Context, Result};
use clap::Args;
use tracing::instrument;

use crate::release_dir;

#[derive(Args, Debug)]
pub struct Clean;

impl Clean {
    #[instrument(name = "Clean release directory", skip_all)]
    pub fn run(&self) -> Result<()> {
        if release_dir().is_dir() {
            fs::remove_dir_all(release_dir()).with_context(|| {
                format!(
                    "failed to clear release directory: {}",
                    release_dir().display()
                )
            })?;
        }

        Ok(())
    }
}
