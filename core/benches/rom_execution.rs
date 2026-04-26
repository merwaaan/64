use std::{hint::black_box, path::Path};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use n64_core::{cart::Cart, system::System};

const ROM_PATH: &str = "_roms/Super Mario 64 (USA).n64.zip";
const STEPS: u64 = 1_000_000;

fn bench_system_step(c: &mut Criterion) {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let rom_path = workspace_root.join(ROM_PATH);

    if !rom_path.is_file() {
        panic!("missing ROM {}", rom_path.display());
    }

    c.bench_function("rom_execution", |b: &mut criterion::Bencher<'_>| {
        b.iter_batched(
            || {
                let cart = Cart::load(&rom_path).unwrap_or_else(|e| {
                    panic!("failed to load ROM {}: {e}", rom_path.display());
                });

                System::with_cart(cart)
            },
            |mut system| {
                for _ in 0..STEPS {
                    black_box(system.step());
                }
            },
            BatchSize::PerIteration,
        );
    });
}

criterion_group!(benches, bench_system_step);
criterion_main!(benches);
