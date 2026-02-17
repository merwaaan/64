use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder, open};
use n64::{cart::Cart, system::System, vi::Vi};
use rstest::rstest;

/// Peter Lemon/krom "Bare Metal" tests
/// https://github.com/PeterLemon/N64
///
/// Many small ROMs, each testing a specific instruction/feature.
/// Also comes with reference images, which is nice.
///
/// We download and run those ROMs, and compare our final framebuffer to the reference image.

#[rstest]
#[case::add("ADD/CPUADD")]
#[case::addu("ADDU/CPUADDU")]
#[case::and("AND/CPUAND")]
#[case::daddu("DADDU/CPUDADDU")]
#[case::ddiv("DDIV/CPUDDIV")]
#[case::ddivu("DDIVU/CPUDDIVU")]
#[case::div("DIV/CPUDIV")]
#[case::divu("DIVU/CPUDIVU")]
#[case::dmult("DMULT/CPUDMULT")]
#[case::dmultu("DMULTU/CPUDMULTU")]
#[case::dsub("DSUB/CPUDSUB")]
#[case::dsubu("DSUBU/CPUDSUBU")]
#[case::lb("LOADSTORE/LB/CPULB")]
#[case::ld("LOADSTORE/LD/CPULD")]
#[case::lh("LOADSTORE/LH/CPULH")]
#[case::lw("LOADSTORE/LW/CPULW")]
// TODO LL_LLD_SC_SCD.N64
#[case::sb("LOADSTORE/SB/CPUSB")]
#[case::sd("LOADSTORE/SD/CPUSD")]
#[case::sh("LOADSTORE/SH/CPUSH")]
#[case::sw("LOADSTORE/SW/CPUSW")]
#[case::mult("MULT/CPUMULT")]
#[case::multu("MULTU/CPUMULTU")]
#[case::nor("NOR/CPUNOR")]
#[case::or("OR/CPUOR")]
#[case::dsll("SHIFT/DSLL/CPUDSLL")]
#[case::dsll32("SHIFT/DSLL32/CPUDSLL32")]
#[case::dsllv("SHIFT/DSLLV/CPUDSLLV")]
#[case::dsra("SHIFT/DSRA/CPUDSRA")]
#[case::dsra32("SHIFT/DSRA32/CPUDSRA32")]
#[case::dsrav("SHIFT/DSRAV/CPUDSRAV")]
#[case::dsrl("SHIFT/DSRL/CPUDSRL")]
#[case::dsrl32("SHIFT/DSRL32/CPUDSRL32")]
#[case::dsrlv("SHIFT/DSRLV/CPUDSRLV")]
#[case::sll("SHIFT/SLL/CPUSLL")]
#[case::sllv("SHIFT/SLLV/CPUSLLV")]
#[case::sra("SHIFT/SRA/CPUSRA")]
#[case::srav("SHIFT/SRAV/CPUSRAV")]
#[case::srl("SHIFT/SRL/CPUSRL")]
#[case::srlv("SHIFT/SRLV/CPUSRLV")]
#[case::subu("SUBU/CPUSUBU")]
//TODO#[case::timingnstc("TIMINGNSTC")]
#[case::xor("XOR/CPUXOR")]
fn lemon_cpu(#[case] test_name: &str) {
    test(format!("CPUTest/CPU/{test_name}"));
}

// TODO cop0
// TODO cop1

fn test(test_name: impl AsRef<str>) {
    // Download the ROM and reference output

    let rom_path = download(format!("{}.N64", test_name.as_ref()));
    let ref_image_path = download(format!("{}.png", test_name.as_ref()));

    // Run the ROM

    let cart = Cart::load(&rom_path).unwrap();

    let mut system = System::new(cart);
    system.skip_ipl();

    for _ in 0..10_000_000 {
        system.step();
    }

    // Compare the framebuffer to the reference image

    let (framebuffer_data, framebuffer_width, framebuffer_height) =
        Vi::extract_framebuffer(&system);

    let ref_image = open(&ref_image_path).expect("Failed to open reference image");
    let ref_image_data = ref_image.to_rgba8();

    assert_eq!(
        (framebuffer_width, framebuffer_height),
        (
            ref_image_data.width() as usize,
            ref_image_data.height() as usize
        ),
        "Framebuffer size {}x{} does not match reference {}x{}",
        framebuffer_width,
        framebuffer_height,
        ref_image_data.width(),
        ref_image_data.height()
    );

    if framebuffer_data
        .iter()
        .zip(ref_image_data.as_raw().iter())
        .any(|(a, b)| a != b)
    {
        let framebuffer_path = ref_image_path.with_extension("output.png");

        PngEncoder::new(BufWriter::new(
            File::create(&framebuffer_path).expect("Failed to create file"),
        ))
        .write_image(
            &framebuffer_data,
            framebuffer_width as u32,
            framebuffer_height as u32,
            ExtendedColorType::Rgba8,
        )
        .expect("Failed to write image");

        panic!("Framebuffer differs from reference");
    }
}

fn download(file_name: impl AsRef<str>) -> PathBuf {
    // Full path ("CPUTest/CPU/AND/CPUAND.N64") to just the end name ("CPUAND.N64")

    let file_name_short = Path::new(file_name.as_ref())
        .file_name()
        .expect("Failed to get file name");

    // Create the root directory if needed

    let dir_path = Path::new("_test_assets");

    let file_path = dir_path.join(file_name_short);

    if !file_path.exists() {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }

        // Download

        let base_url = "https://raw.githubusercontent.com/PeterLemon/N64/master";

        let file_url = format!("{}/{}", base_url, file_name.as_ref());

        let data = reqwest::blocking::get(file_url)
            .expect("Failed to download file")
            .bytes()
            .expect("Failed to read file");

        let file = File::create(&file_path).expect("Failed to create file");
        let mut writer = BufWriter::new(file);
        writer.write_all(&data).unwrap();
    }

    file_path
}
