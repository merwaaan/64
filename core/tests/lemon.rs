///! Peter Lemon/krom's "Bare Metal" tests
///! https://github.com/PeterLemon/N64
///!
///! Many small ROMs, each testing a specific instruction/feature.
///! Also comes with reference images, which is nice.
///!
///! We download and run those ROMs, and compare our final framebuffer to the reference image.
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder, open};
use n64_core::{cart::Cart, system::System, vi::Vi};
use rstest::rstest;

#[rstest]
#[case::add("ADD/CPUADD")]
#[case::addu("ADDU/CPUADDU")]
#[case::and("AND/CPUAND")]
#[case::daddu("DADDU/CPUDADDU")]
//#[case::ddiv("DDIV/CPUDDIV")] // TODO pass but wrong values?
//#[case::ddivu("DDIVU/CPUDDIVU")] // TODO pass but wrong values?
#[case::div("DIV/CPUDIV")]
#[case::divu("DIVU/CPUDIVU")]
//#[case::dmult("DMULT/CPUDMULT")] // TODO pass but wrong values?
//#[case::dmultu("DMULTU/CPUDMULTU")] // TODO pass but wrong values?
#[case::dsub("DSUB/CPUDSUB")]
#[case::dsubu("DSUBU/CPUDSUBU")]
#[case::lb("LOADSTORE/LB/CPULB")]
#[case::ld("LOADSTORE/LD/CPULD")]
#[case::lh("LOADSTORE/LH/CPULH")]
#[case::lw("LOADSTORE/LW/CPULW")]
#[case::ll_lld_sc_scd("LOADSTORE/LL_LLD_SC_SCD/LL_LLD_SC_SCD")]
//#[case::sb("LOADSTORE/SB/CPUSB")] // TODO ref image has weird line
#[case::sd("LOADSTORE/SD/CPUSD")]
//#[case::sh("LOADSTORE/SH/CPUSH")] // TODO ref image has weird line
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
#[case::xor("XOR/CPUXOR")]
fn cpu(#[case] test_name: &str) {
    test(format!("CPUTest/CPU/{test_name}"));
}

#[rstest]
#[case::cause("COP0Cause/COP0Cause")]
fn cop0(#[case] test_name: &str) {
    test(format!("CPUTest/CP0/{test_name}"));
}

#[rstest]
#[case::abs("ABS/CP1ABS")]
#[case::add("ADD/CP1ADD")]
#[case::ceil("CEIL/CP1CEIL")]
#[case::fullmode("COP1FullMode/COP1FullMode")]
#[case::c_eq("C/EQ/CP1CEQ")]
#[case::c_f("C/F/CP1CF")]
#[case::c_le("C/LE/CP1CLE")]
#[case::c_lt("C/LT/CP1CLT")]
#[case::c_nge("C/NGE/CP1CNGE")]
#[case::c_ngle("C/NGLE/CP1CNGLE")]
#[case::c_ngt("C/NGT/CP1CNGT")]
#[case::c_ole("C/OLE/CP1COLE")]
#[case::c_olt("C/OLT/CP1COLT")]
#[case::c_seq("C/SEQ/CP1CSEQ")]
#[case::c_sf("C/SF/CP1CSF")]
#[case::c_ueq("C/UEQ/CP1CUEQ")]
#[case::c_ule("C/ULE/CP1CULE")]
#[case::c_ult("C/ULT/CP1CULT")]
#[case::c_un("C/UN/CP1CUN")]
// TODO FPUCompare?
#[case::cvt("CVT/CP1CVT")]
#[case::div("DIV/CP1DIV")]
#[case::floor("FLOOR/CP1FLOOR")]
#[case::mul("MUL/CP1MUL")]
#[case::neg("NEG/CP1NEG")]
#[case::round("ROUND/CP1ROUND")]
#[case::sqrt("SQRT/CP1SQRT")]
#[case::sub("SUB/CP1SUB")]
#[case::trunc("TRUNC/CP1TRUNC")]

fn cop1(#[case] test_name: &str) {
    test(format!("CPUTest/CP1/{test_name}"));
}

// TODO rf images with same names conflict (single/double)
#[rstest]
#[case::julia_320_double("320X240/Julia/Double/Julia32BPP320X240")]
#[case::julia_320_single("320X240/Julia/Single/Julia32BPP320X240")]
#[case::julia_320_input_double("320X240/JuliaInput/Double/JuliaInput32BPP320X240")]
#[case::julia_320_input_single("320X240/JuliaInput/Single/JuliaInput32BPP320X240")]
#[case::mandelbrot_320_double("320X240/Mandelbrot/Double/Mandelbrot32BPP320X240")]
#[case::mandelbrot_320_single("320X240/Mandelbrot/Single/Mandelbrot32BPP320X240")]
#[case::mandelbrot_320_input_double("320X240/MandelbrotInput/Double/MandelbrotInput32BPP320X240")]
#[case::mandelbrot_320_input_single("320X240/MandelbrotInput/Single/MandelbrotInput32BPP320X240")]
#[case::julia_640_double("640X480/Julia/Double/Julia32BPP640X480")]
#[case::julia_640_single("640X480/Julia/Single/Julia32BPP640X480")]
#[case::mandelbrot_640_double("640X480/Mandelbrot/Double/Mandelbrot32BPP640X480")]
#[case::mandelbrot_640_single("640X480/Mandelbrot/Single/Mandelbrot32BPP640X480")]
fn cop1_fractals(#[case] test_name: &str) {
    test(format!("CP1/Fractal/32BPP/{test_name}"));
}

#[rstest]
#[case::dma("DMAAlignment-PI-ROM-FROM")]
#[case::dma_large_2("DMAAlignment-PI-ROM-FROM_large_2")]
#[case::dma_large_4("DMAAlignment-PI-ROM-FROM_large_4")]
#[case::dma_large_6("DMAAlignment-PI-ROM-FROM_large_6")]
fn pi_dma(#[case] test_name: &str) {
    test(format!("CPUTest/DMAAlignment-PI-cart/{test_name}"));
}

#[rstest]
#[case::compare_disabled("Compare/ExceptionCompareDisabled")]
#[case::compare_registers("Compare/ExceptionCompareRegisters")]
#[case::syscall("Syscall/ExceptionSyscall")]
#[case::syscall_delay("Syscall/ExceptionSyscallDelay")]
#[case::syscall_delay_2("Syscall/ExceptionSyscallDelay2")]
#[case::syscall_while_in_exception("Syscall/ExceptionSyscallWhileInException")]
#[case::tlb_read_miss("TLB/ExceptionTLBReadMiss")]
#[case::tlb_read_miss_delay("TLB/ExceptionTLBReadMissDelay")]
#[case::tlb_read_miss_nested("TLB/ExceptionTLBReadMissNested")]
#[case::tlb_read_miss_nested_delay("TLB/ExceptionTLBReadMissNestedDelay")]
#[case::tlb_write_miss("TLB/ExceptionTLBWriteMiss")]
#[case::tlb_write_miss_delay("TLB/ExceptionTLBWriteMissDelay")]
#[case::trap_teq("Trap/ExceptionTEQ")]
#[case::trap_teq_delay("Trap/ExceptionTEQDelay")]
#[case::unaligned("Unaligned/ExceptionUnaligned")]
#[case::unaligned_delay("Unaligned/ExceptionUnalignedDelay")]
#[case::vii_intr_disabled("VIIntr/ExceptionVIIntrDisabled")]
fn exceptions(#[case] test_name: &str) {
    test(format!("CPUTest/Exceptions/{test_name}"));
}

#[rstest]
#[case::registers("RDRAMTest/RDRAMTest")]
fn rdram(#[case] test_name: &str) {
    test(test_name);
}

#[rstest]
#[case::version("Version/RCPVersion")]
#[case::vi_coverage("VI/CoverageTest/CoverageTest")]
fn rcp(#[case] test_name: &str) {
    test(format!("RCP/{test_name}"));
}

#[rstest]
#[case::framebuffer_16_cpu("16BPP/FrameBufferCPU320x240/FrameBufferCPU16BPP320X240")]
#[case::framebuffer_16_dma("16BPP/FrameBufferDMA320x240/FrameBufferDMA16BPP320X240")]
#[case::framebuffer_32_cpu("32BPP/FrameBufferCPU640x480/FrameBufferCPU32BPP640X480")]
#[case::framebuffer_32_dma("32BPP/FrameBufferDMA640x480/FrameBufferDMA32BPP640X480")]
fn framebuffer(#[case] test_name: &str) {
    test(format!("FrameBuffer/{test_name}"));
}

#[rstest]
#[case::hello_world_16_cpu("16BPP/HelloWorldCPU320x240/HelloWorldCPU16BPP320X240")]
#[case::hello_world_16_rdp("16BPP/HelloWorldRDP320x240/HelloWorldRDP16BPP320X240")]
#[case::hello_world_32_cpu("32BPP/HelloWorldCPU320x240/HelloWorldCPU32BPP320X240")]
#[case::hello_world_32_rdp("32BPP/HelloWorldRDP320x240/HelloWorldRDP32BPP320X240")]
fn hello_world(#[case] test_name: &str) {
    test(format!("HelloWorld/{test_name}"));
}

// TODO fractals
// TODO rsp
// TODO rdp

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

    // Save the framebuffer

    let (framebuffer_data, framebuffer_width, framebuffer_height) =
        Vi::extract_framebuffer(&system);

    if framebuffer_width > 0 && framebuffer_height > 0 {
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
    }

    // Compare the framebuffer to the reference image

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
        BufWriter::new(file).write_all(&data).unwrap();
    }

    file_path
}
