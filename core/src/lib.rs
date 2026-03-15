#![feature(portable_simd)]

use std::path::Path;

pub mod ai;
pub(crate) mod bits;
pub mod breakpoints;
pub mod cart;
pub mod controller;
pub mod cop0;
pub mod cop1;
pub mod cpu;
pub mod dd;
pub mod dp;
pub mod events;
pub mod exception;
pub mod isviewer;
pub mod location;
pub mod mi;
pub mod openbus;
pub mod pi;
pub mod pif;
pub mod ram;
pub mod registers;
pub mod rendering;
pub mod si;
pub mod sp;
pub mod system;
pub mod tlb;
pub mod value;
pub mod vi;

pub fn is_supported_rom_file(path: &Path) -> bool {
    const ROM_EXTENSIONS: &[&str] = &["n64", "z64", "v64"];

    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            ROM_EXTENSIONS
                .iter()
                .any(|supported| supported.eq_ignore_ascii_case(ext))
        })
}

pub fn get_supported_file_extensions() -> &'static [&'static str] {
    const FILE_EXTENSIONS: &[&str] = &["n64", "z64", "v64", "zip"];

    FILE_EXTENSIONS
}

pub fn is_supported_file(path: &Path) -> bool {
    get_supported_file_extensions().iter().any(|supported| {
        supported.eq_ignore_ascii_case(path.extension().and_then(|ext| ext.to_str()).unwrap_or(""))
    })
}

// TODO bugs:
// - Indy Racing 2000 + Army Men Air Combat: out of bounds (open bus?)
// - Monaco Grand Prix: read invalid address 840C0000 (unmapped)
// - Quake: access invalid reg #13 (si or pi?)
// - Chopper Attack: PIF out of range

// TODO next steps:
// - count/compare: interrupt cleared when compare changed??
// - DMA double-buffering (AI ok, others now)
// - exception on reserved/unknown instructions?
