use std::{env, fs, io, path::PathBuf};

use anyhow::{Result, bail};
use clap::Args;
use tracing::{debug, instrument, warn};

use crate::tools_dir;

#[derive(Args, Debug)]
pub struct Upload {
    /// The path of the ROM to upload.
    pub path: PathBuf,
}

impl Upload {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[instrument(name = "Upload ROM", skip_all, fields(path = %self.path.display()))]
    pub fn run(&self) -> Result<()> {
        upload_rom_to_sc64(&self.path)
    }
}

fn sc64deployer_path() -> Result<PathBuf> {
    let path = tools_dir().join(sc64_deployer_name());

    if !path.exists() {
        download_sc64deployer(&path)?;
    }

    Ok(path)
}

fn sc64_deployer_name() -> &'static str {
    if env::consts::OS == "windows" {
        "sc64deployer.exe"
    } else {
        "sc64deployer"
    }
}

#[instrument(name = "Download sc64deployer", skip_all)]
fn download_sc64deployer(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let url = match env::consts::OS {
        "windows" => {
            "https://github.com/Polprzewodnikowy/SummerCart64/releases/download/v2.20.2/sc64-deployer-windows-v2.20.2.zip"
        }
        "linux" => {
            "https://github.com/Polprzewodnikowy/SummerCart64/releases/download/v2.20.2/sc64-deployer-linux-v2.20.2.tar.gz"
        }
        "macos" => {
            "https://github.com/Polprzewodnikowy/SummerCart64/releases/download/v2.20.2/sc64-deployer-macos-v2.20.2.tgz"
        }
        _ => bail!("unsupported platform"),
    };

    debug!("url: {url}...");

    let response = ureq::get(url).call()?;

    let reader = io::Cursor::new(response.into_body().read_to_vec()?);
    let mut archive = zip::ZipArchive::new(reader)?;

    let mut archive_file = archive.by_name(sc64_deployer_name())?;

    let mut file = fs::File::create(&path)?;
    io::copy(&mut archive_file, &mut file)?;

    Ok(())
}

fn upload_rom_to_sc64(path: &PathBuf) -> Result<()> {
    let result = duct::cmd!(sc64deployer_path()?, "upload", path, "--reboot")
        .stderr_to_stdout()
        .stdout_capture()
        .unchecked()
        .run()?;

    if !result.status.success() {
        let stdout = String::from_utf8_lossy(&result.stdout);
        bail!("sc64deployer error, {}", stdout);
    }

    Ok(())
}
