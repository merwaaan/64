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
#[case::add("ADD/CPUADD.N64")]
#[case::addu("ADDU/CPUADDU.N64")]
#[case::and("AND/CPUAND.N64")]
#[case::daddu("DADDU/CPUDADDU.N64")]
//#[case::ddiv("DDIV/CPUDDIV.N64")] // TODO pass but wrong values?
//#[case::ddivu("DDIVU/CPUDDIVU.N64")] // TODO pass but wrong values?
#[case::div("DIV/CPUDIV.N64")]
#[case::divu("DIVU/CPUDIVU.N64")]
//#[case::dmult("DMULT/CPUDMULT.N64")] // TODO pass but wrong values?
//#[case::dmultu("DMULTU/CPUDMULTU.N64")] // TODO pass but wrong values?
#[case::dsub("DSUB/CPUDSUB.N64")]
#[case::dsubu("DSUBU/CPUDSUBU.N64")]
#[case::lb("LOADSTORE/LB/CPULB.N64")]
#[case::ld("LOADSTORE/LD/CPULD.N64")]
#[case::lh("LOADSTORE/LH/CPULH.N64")]
#[case::lw("LOADSTORE/LW/CPULW.N64")]
#[case::ll_lld_sc_scd("LOADSTORE/LL_LLD_SC_SCD/LL_LLD_SC_SCD.N64")]
//#[case::sb("LOADSTORE/SB/CPUSB.N64")] // TODO ref image has weird line
#[case::sd("LOADSTORE/SD/CPUSD.N64")]
//#[case::sh("LOADSTORE/SH/CPUSH.N64")] // TODO ref image has weird line
#[case::sw("LOADSTORE/SW/CPUSW.N64")]
#[case::mult("MULT/CPUMULT.N64")]
#[case::multu("MULTU/CPUMULTU.N64")]
#[case::nor("NOR/CPUNOR.N64")]
#[case::or("OR/CPUOR.N64")]
#[case::dsll("SHIFT/DSLL/CPUDSLL.N64")]
#[case::dsll32("SHIFT/DSLL32/CPUDSLL32.N64")]
#[case::dsllv("SHIFT/DSLLV/CPUDSLLV.N64")]
#[case::dsra("SHIFT/DSRA/CPUDSRA.N64")]
#[case::dsra32("SHIFT/DSRA32/CPUDSRA32.N64")]
#[case::dsrav("SHIFT/DSRAV/CPUDSRAV.N64")]
#[case::dsrl("SHIFT/DSRL/CPUDSRL.N64")]
#[case::dsrl32("SHIFT/DSRL32/CPUDSRL32.N64")]
#[case::dsrlv("SHIFT/DSRLV/CPUDSRLV.N64")]
#[case::sll("SHIFT/SLL/CPUSLL.N64")]
#[case::sllv("SHIFT/SLLV/CPUSLLV.N64")]
#[case::sra("SHIFT/SRA/CPUSRA.N64")]
#[case::srav("SHIFT/SRAV/CPUSRAV.N64")]
#[case::srl("SHIFT/SRL/CPUSRL.N64")]
#[case::srlv("SHIFT/SRLV/CPUSRLV.N64")]
#[case::subu("SUBU/CPUSUBU.N64")]
#[case::xor("XOR/CPUXOR.N64")]
fn cpu(#[case] test_name: &str) {
    test(format!("CPUTest/CPU/{test_name}"), |_| {});
}

#[rstest]
#[case::cause("COP0Cause/COP0Cause.N64")]
fn cop0(#[case] test_name: &str) {
    test(format!("CPUTest/CP0/{test_name}"), |s| {
        // The test starts with coprocessor error set to 3, the actual cause is unclear
        // https://github.com/PeterLemon/N64/blob/7085543e4a19d8c539fc9e0a4d2869e788b4ed4b/CPUTest/CP0/COP0Cause/COP0Cause.asm
        s.cop0.set_coprocessor_error(3);
    });
}

#[rstest]
#[case::abs("ABS/CP1ABS.N64")]
#[case::add("ADD/CP1ADD.N64")]
#[case::ceil("CEIL/CP1CEIL.N64")]
#[case::fullmode("COP1FullMode/COP1FullMode.N64")]
#[case::c_eq("C/EQ/CP1CEQ.N64")]
#[case::c_f("C/F/CP1CF.N64")]
#[case::c_le("C/LE/CP1CLE.N64")]
#[case::c_lt("C/LT/CP1CLT.N64")]
#[case::c_nge("C/NGE/CP1CNGE.N64")]
#[case::c_ngle("C/NGLE/CP1CNGLE.N64")]
#[case::c_ngt("C/NGT/CP1CNGT.N64")]
#[case::c_ole("C/OLE/CP1COLE.N64")]
#[case::c_olt("C/OLT/CP1COLT.N64")]
#[case::c_seq("C/SEQ/CP1CSEQ.N64")]
#[case::c_sf("C/SF/CP1CSF.N64")]
#[case::c_ueq("C/UEQ/CP1CUEQ.N64")]
#[case::c_ule("C/ULE/CP1CULE.N64")]
#[case::c_ult("C/ULT/CP1CULT.N64")]
#[case::c_un("C/UN/CP1CUN.N64")]
// TODO FPUCompare?
#[case::cvt("CVT/CP1CVT.N64")]
#[case::div("DIV/CP1DIV.N64")]
#[case::floor("FLOOR/CP1FLOOR.N64")]
#[case::mul("MUL/CP1MUL.N64")]
#[case::neg("NEG/CP1NEG.N64")]
#[case::round("ROUND/CP1ROUND.N64")]
#[case::sqrt("SQRT/CP1SQRT.N64")]
#[case::sub("SUB/CP1SUB.N64")]
#[case::trunc("TRUNC/CP1TRUNC.N64")]
fn cop1(#[case] test_name: &str) {
    test(format!("CPUTest/CP1/{test_name}"), |_| {});
}

// TODO rf images with same names conflict (single/double)
#[rstest]
#[case::julia_320_double("320X240/Julia/Double/Julia32BPP320X240.N64")]
#[case::julia_320_single("320X240/Julia/Single/Julia32BPP320X240.N64")]
#[case::julia_320_input_double("320X240/JuliaInput/Double/JuliaInput32BPP320X240.N64")]
#[case::julia_320_input_single("320X240/JuliaInput/Single/JuliaInput32BPP320X240.N64")]
#[case::mandelbrot_320_double("320X240/Mandelbrot/Double/Mandelbrot32BPP320X240.N64")]
#[case::mandelbrot_320_single("320X240/Mandelbrot/Single/Mandelbrot32BPP320X240.N64")]
#[case::mandelbrot_320_input_double(
    "320X240/MandelbrotInput/Double/MandelbrotInput32BPP320X240.N64"
)]
#[case::mandelbrot_320_input_single(
    "320X240/MandelbrotInput/Single/MandelbrotInput32BPP320X240.N64"
)]
#[case::julia_640_double("640X480/Julia/Double/Julia32BPP640X480.N64")]
#[case::julia_640_single("640X480/Julia/Single/Julia32BPP640X480.N64")]
#[case::mandelbrot_640_double("640X480/Mandelbrot/Double/Mandelbrot32BPP640X480.N64")]
#[case::mandelbrot_640_single("640X480/Mandelbrot/Single/Mandelbrot32BPP640X480.N64")]
fn cop1_fractals(#[case] test_name: &str) {
    test(format!("CP1/Fractal/32BPP/{test_name}"), |_| {});
}

#[rstest]
#[case::dma("DMAAlignment-PI-ROM-FROM.N64")]
#[case::dma_large_2("DMAAlignment-PI-ROM-FROM_large_2.N64")]
#[case::dma_large_4("DMAAlignment-PI-ROM-FROM_large_4.N64")]
#[case::dma_large_6("DMAAlignment-PI-ROM-FROM_large_6.N64")]
fn pi_dma(#[case] test_name: &str) {
    test(format!("CPUTest/DMAAlignment-PI-cart/{test_name}"), |_| {});
}

#[rstest]
#[case::compare_disabled("Compare/ExceptionCompareDisabled.n64")]
#[case::compare_registers("Compare/ExceptionCompareRegisters.n64")]
#[case::syscall("Syscall/ExceptionSyscall.n64")]
#[case::syscall_delay("Syscall/ExceptionSyscallDelay.n64")]
#[case::syscall_delay_2("Syscall/ExceptionSyscallDelay2.n64")]
#[case::syscall_while_in_exception("Syscall/ExceptionSyscallWhileInException.n64")]
#[case::tlb_read_miss("TLB/ExceptionTLBReadMiss.n64")]
#[case::tlb_read_miss_delay("TLB/ExceptionTLBReadMissDelay.n64")]
#[case::tlb_read_miss_nested("TLB/ExceptionTLBReadMissNested.n64")]
#[case::tlb_read_miss_nested_delay("TLB/ExceptionTLBReadMissNestedDelay.n64")]
#[case::tlb_write_miss("TLB/ExceptionTLBWriteMiss.n64")]
#[case::tlb_write_miss_delay("TLB/ExceptionTLBWriteMissDelay.n64")]
#[case::trap_teq("Trap/ExceptionTEQ.n64")]
#[case::trap_teq_delay("Trap/ExceptionTEQDelay.n64")]
#[case::unaligned("Unaligned/ExceptionUnaligned.n64")]
#[case::unaligned_delay("Unaligned/ExceptionUnalignedDelay.n64")]
#[case::vii_intr_disabled("VIIntr/ExceptionVIIntrDisabled.n64")]
fn exceptions(#[case] test_name: &str) {
    test(format!("CPUTest/Exceptions/{test_name}"), |s| {
        // Tests start with error registers set to 0xFFFFFFFF TODO investigate why
        s.cop0.set_exception_pc(u32::MAX);
        s.cop0.set_error_pc(u32::MAX);
        s.cop0.set_bad_virtual_address(u32::MAX);
        // Tests start with a weird STATUS TODO investigate why
        s.cop0.set_status(0x2410_00E0);
    });
}

#[rstest]
#[case::registers("RDRAMTest/RDRAMTest.N64")]
fn rdram(#[case] test_name: &str) {
    test(test_name, |_| {});
}

#[rstest]
#[case::version("Version/RCPVersion.N64")]
#[case::vi_coverage("VI/CoverageTest/CoverageTest.N64")]
fn rcp(#[case] test_name: &str) {
    test(format!("RCP/{test_name}"), |_| {});
}

#[rstest]
#[case::framebuffer_16_cpu("16BPP/FrameBufferCPU320x240/FrameBufferCPU16BPP320X240.N64")]
#[case::framebuffer_16_dma("16BPP/FrameBufferDMA320x240/FrameBufferDMA16BPP320X240.N64")]
#[case::framebuffer_32_cpu("32BPP/FrameBufferCPU640x480/FrameBufferCPU32BPP640X480.N64")]
#[case::framebuffer_32_dma("32BPP/FrameBufferDMA640x480/FrameBufferDMA32BPP640X480.N64")]
fn framebuffer(#[case] test_name: &str) {
    test(format!("FrameBuffer/{test_name}"), |_| {});
}

#[rstest]
#[case::hello_world_16_cpu("16BPP/HelloWorldCPU320x240/HelloWorldCPU16BPP320X240.N64")]
#[case::hello_world_16_rdp("16BPP/HelloWorldRDP320x240/HelloWorldRDP16BPP320X240.N64")]
#[case::hello_world_32_cpu("32BPP/HelloWorldCPU320x240/HelloWorldCPU32BPP320X240.N64")]
#[case::hello_world_32_rdp("32BPP/HelloWorldRDP320x240/HelloWorldRDP32BPP320X240.N64")]
fn hello_world(#[case] test_name: &str) {
    test(format!("HelloWorld/{test_name}"), |_| {});
}

// TODO fractals
// TODO rsp
// TODO rdp

fn test(test_name: impl AsRef<str>, setup: impl FnOnce(&mut System)) {
    // Download the ROM and reference image

    let rom_path = download(test_name.as_ref());

    let image_name = Path::new(test_name.as_ref()).with_extension("png");
    let ref_image_path = download(image_name.to_string_lossy());

    // Run the ROM

    let cart = Cart::load(&rom_path).unwrap();

    let mut system = System::with_cart(cart);

    setup(&mut system);

    for _ in 0..100_000_000 {
        system.step();
    }

    // Save the framebuffer

    let (framebuffer_data, framebuffer_width, framebuffer_height) =
        Vi::extract_framebuffer(&mut system);

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
