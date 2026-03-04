use std::{
    env, fs, io, panic,
    path::{Path, PathBuf},
};

use clap::Parser;
use n64_core::{cart::Cart, is_supported_file, system::System, vi::Vi};

#[derive(Parser)]
#[command(about = "Run ROMs for N cycles and dump framebuffers")]
struct Args {
    /// Folder to scan for ROM files
    #[arg()]
    dir: PathBuf,

    /// Number of emulator cycles to run per ROM
    #[arg(short, long, default_value = "1000000000")]
    cycles: usize,
}

fn main() -> Result<(), String> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("runner=info"))
        .init();

    let args = Args::parse();

    if !args.dir.is_dir() {
        return Err(format!("Not a directory: {}", args.dir.display()));
    }

    const CAPTURES_DIR: &str = "_runner_captures";

    let captures_dir = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(CAPTURES_DIR);

    fs::create_dir_all(&captures_dir).map_err(|e| format!("{}: {}", captures_dir.display(), e))?;

    let roms = collect_roms(&args.dir).map_err(|e| e.to_string())?;

    log::info!("Running {} ROMs", roms.len());

    for path in &roms {
        run_rom(path, args.cycles, &captures_dir);
    }

    Ok(())
}

fn collect_roms(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut roms = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if is_supported_file(&path) {
            roms.push(path);
        }
    }

    roms.sort();

    Ok(roms)
}

fn run_rom(path: &Path, cycles: usize, captures_dir: &Path) {
    let mut framebuffer = None;

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        log::info!("Running {} ...", path.display());

        let cart = Cart::load(path).map_err(|e| e.to_string())?;

        let mut system = System::new(cart);
        system.skip_ipl();

        const CAPTURE_INTERVAL: usize = 100_000;
        let mut next_capture = CAPTURE_INTERVAL;

        while system.cycles < cycles {
            system.step();

            if system.cycles >= next_capture {
                if system.map.vi.framebuffer_width() > 0 {
                    framebuffer = Some(Vi::extract_framebuffer(&system));
                }

                next_capture = system.cycles + CAPTURE_INTERVAL;
            }
        }

        Ok(()) as Result<(), String>
    }));

    if let Some((data, width, height)) = framebuffer {
        if let Some(img) = image::RgbaImage::from_raw(width as u32, height as u32, data) {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("frame");
            let framebuffer_path = captures_dir.join(format!("{}.png", stem));

            if img.save(&framebuffer_path).is_ok() {
                log::debug!(
                    "Saved {} ({}x{})",
                    framebuffer_path.display(),
                    width,
                    height,
                );
            }
        }
    }

    match result {
        Ok(Ok(())) => log::info!("{} ok", path.display()),
        Ok(Err(e)) => log::error!("{} {}", path.display(), e),
        Err(panic_payload) => {
            let message = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "panic".to_string()
            };

            log::error!("{} panic: {}", path.display(), message);
        }
    }
}
