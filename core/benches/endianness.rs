use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use n64_core::value::Value;
use rand::{RngExt, rng};

fn bench_xxxx(c: &mut Criterion) {
    let mut group = c.benchmark_group("MemoryAccess");

    group.sample_size(100);

    let mut mem = vec![0u8; 4 * 1024 * 1024]; // 4MB
    rng().fill(&mut mem[..]);

    group.bench_function("u8::read_mem", |b| {
        b.iter(|| {
            for i in 0..mem.len() {
                black_box(u8::read_mem(&mem, i as u32));
            }
        })
    });

    group.bench_function("u16::read_mem", |b| {
        b.iter(|| {
            for i in 0..(mem.len() - 2) {
                black_box(u16::read_mem(&mem, i as u32));
            }
        })
    });

    group.bench_function("u32::read_mem", |b| {
        b.iter(|| {
            for i in 0..(mem.len() - 4) {
                black_box(u32::read_mem(&mem, i as u32));
            }
        })
    });

    group.bench_function("u64::read_mem", |b| {
        b.iter(|| {
            for i in 0..(mem.len() - 8) {
                black_box(u64::read_mem(&mem, i as u32));
            }
        })
    });

    group.bench_function("u8::write_mem", |b| {
        b.iter(|| {
            for i in 0..mem.len() {
                black_box(u8::MAX.write_mem(&mut mem, i as u32));
            }
        })
    });

    group.bench_function("u16::write_mem", |b| {
        b.iter(|| {
            for i in 0..(mem.len() - 2) {
                black_box(u16::MAX.write_mem(&mut mem, i as u32));
            }
        })
    });

    group.bench_function("u32::write_mem", |b| {
        b.iter(|| {
            for i in 0..(mem.len() - 4) {
                black_box(u32::MAX.write_mem(&mut mem, i as u32));
            }
        })
    });

    group.bench_function("u64::write_mem", |b| {
        b.iter(|| {
            for i in 0..(mem.len() - 8) {
                black_box(u64::MAX.write_mem(&mut mem, i as u32));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_xxxx);
criterion_main!(benches);
