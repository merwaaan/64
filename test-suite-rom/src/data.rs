// Recognizable values to spot uninitialized data

use n64_specs::cpu::registers::Register;

pub const INIT_16: u16 = 0x0BAD;
pub const INIT_32: u32 = 0x0BAD_0BAD;
pub const INIT_64: u64 = 0x0BAD_0BAD_0BAD_0BAD;

// Notable corner case values for tests.
// Defining them once here to avoid repeating them in each test.

pub const CORNER_CASES_16: &[u16] = &[
    // Start
    0x0000, //
    0x0001, //
    // Sign boundary
    0x7FFF, //
    0x8000, //
    0x8001, //
    // End
    0xFFFE, //
    0xFFFF,
];

pub const CORNER_CASES_32: &[u32] = &[
    // Start
    0x0000_0000,
    0x0000_0001,
    // 16-bit sign boundary
    0x0000_7FFF,
    0x0000_8000,
    0x0000_8001,
    // 16-bit end
    0x0000_FFFE,
    0x0000_FFFF,
    // 32-bit sign boundary
    0x7FFF_FFFF,
    0x8000_0000,
    0x8000_0001,
    // 32-bit end
    0xFFFF_FFFE,
    0xFFFF_FFFF,
];

pub const CORNER_CASES_64: &[u64] = &[
    // Start
    0x0000_0000_0000_0000,
    0x0000_0000_0000_0001,
    // 32-bit sign boundary
    0x0000_0000_7FFF_FFFF,
    0x0000_0000_8000_0000,
    0x0000_0000_8000_0001,
    // 32-bit end
    0x0000_0000_FFFF_FFFE,
    0x0000_0000_FFFF_FFFF,
    // 64-bit sign boundary
    0x7FFF_FFFF_FFFF_FFFE,
    0x8000_0000_0000_0000,
    0x8000_0000_0000_0001,
    // 64-bit end
    0xFFFF_FFFF_FFFF_FFFE,
    0xFFFF_FFFF_FFFF_FFFF,
];

// Helpers to get the corner cases + some extra values
// TODO also use narrower values for each width or too many tests?

pub fn corner_cases_16(extra: &[u16]) -> impl Iterator<Item = u16> + Clone {
    CORNER_CASES_16.iter().copied().chain(extra.iter().copied())
}

pub fn corner_cases_32(extra: &[u32]) -> impl Iterator<Item = u32> + Clone {
    CORNER_CASES_32.iter().copied().chain(extra.iter().copied())
}

pub fn corner_cases_64(extra: &[u64]) -> impl Iterator<Item = u64> + Clone {
    CORNER_CASES_64.iter().copied().chain(extra.iter().copied())
}

// Helper to generate register combinations.
// Include standard cases (eg. separate RD, RT, RS) as well as corner cases (R0, RT = RS, etc).

#[derive(Clone, Copy, Debug)]
pub struct RdRtRs {
    pub rd: Register,
    pub rd_value: u64,

    pub rt: Register,
    pub rt_value: u64,

    pub rs: Register,
    pub rs_value: u64,
}

pub fn rd_rt_rs_combinations(
    values: impl Iterator<Item = u64> + Clone,
) -> impl Iterator<Item = RdRtRs> + Clone {
    let basic =
        itertools::iproduct!(values.clone(), values.clone()).map(|(rs_value, rt_value)| RdRtRs {
            rd: Register::T0,
            rd_value: INIT_64,
            rs: Register::T1,
            rs_value,
            rt: Register::T2,
            rt_value,
        });

    let rd_is_r0 =
        itertools::iproduct!(values.clone(), values.clone()).map(|(rs_value, rt_value)| RdRtRs {
            rd: Register::R0,
            rd_value: 0,
            rs: Register::T0,
            rs_value,
            rt: Register::T1,
            rt_value,
        });

    let rs_is_r0 = values.clone().map(|rt_value| RdRtRs {
        rd: Register::T0,
        rd_value: INIT_64,
        rs: Register::R0,
        rs_value: 0,
        rt: Register::T1,
        rt_value,
    });

    let rt_is_r0 = values.clone().map(|rs_value| RdRtRs {
        rd: Register::T0,
        rd_value: INIT_64,
        rs: Register::T1,
        rs_value,
        rt: Register::R0,
        rt_value: 0,
    });

    let rd_is_rt =
        itertools::iproduct!(values.clone(), values.clone()).map(|(rs_value, rt_value)| RdRtRs {
            rd: Register::T0,
            rd_value: rt_value,
            rs: Register::T1,
            rs_value,
            rt: Register::T0,
            rt_value,
        });

    let rd_is_rs =
        itertools::iproduct!(values.clone(), values.clone()).map(|(rs_value, rt_value)| RdRtRs {
            rd: Register::T0,
            rd_value: rs_value,
            rs: Register::T0,
            rs_value,
            rt: Register::T1,
            rt_value,
        });

    let rs_is_rt = values.clone().map(|rt_value| RdRtRs {
        rd: Register::T0,
        rd_value: INIT_64,
        rs: Register::T1,
        rs_value: rt_value,
        rt: Register::T1,
        rt_value,
    });

    let rd_is_rs_is_rt = values.clone().map(|value| RdRtRs {
        rd: Register::T0,
        rd_value: value,
        rs: Register::T0,
        rs_value: value,
        rt: Register::T0,
        rt_value: value,
    });

    basic
        .chain(rd_is_r0)
        .chain(rs_is_r0)
        .chain(rt_is_r0)
        .chain(rd_is_rt)
        .chain(rd_is_rs)
        .chain(rs_is_rt)
        .chain(rd_is_rs_is_rt)
}

#[derive(Clone, Copy, Debug)]
pub struct RtRs {
    pub rt: Register,
    pub rt_value: u64,

    pub rs: Register,
    pub rs_value: u64,
}

pub fn rt_rs_combinations(
    reg_values: impl Iterator<Item = u64> + Clone,
) -> impl Iterator<Item = RtRs> + Clone {
    let basic =
        itertools::iproduct!(reg_values.clone(), reg_values.clone()).map(|(rs_value, rt_value)| {
            RtRs {
                rs: Register::T0,
                rs_value,
                rt: Register::T1,
                rt_value,
            }
        });

    let rt_is_r0 = reg_values.clone().map(|rs_value| RtRs {
        rs: Register::T0,
        rs_value,
        rt: Register::R0,
        rt_value: 0,
    });

    let rs_is_r0 = reg_values.clone().map(|rt_value| RtRs {
        rs: Register::R0,
        rs_value: 0,
        rt: Register::T0,
        rt_value,
    });

    let rt_is_rs = reg_values.clone().map(|value| RtRs {
        rs: Register::T0,
        rs_value: value,
        rt: Register::T0,
        rt_value: value,
    });

    basic.chain(rt_is_r0).chain(rs_is_r0).chain(rt_is_rs)
}

#[derive(Clone, Copy, Debug)]
pub struct RtRsImm {
    pub rt: Register,
    pub rt_value: u64,

    pub rs: Register,
    pub rs_value: u64,

    pub imm: u16,
}

pub fn rt_rs_imm_combinations(
    reg_values: impl Iterator<Item = u64> + Clone,
    imm_values: impl Iterator<Item = u16> + Clone,
) -> impl Iterator<Item = RtRsImm> + Clone {
    itertools::iproduct!(rt_rs_combinations(reg_values.clone()), imm_values.clone()).map(
        |(
            RtRs {
                rs,
                rs_value,
                rt,
                rt_value,
            },
            imm,
        )| RtRsImm {
            rs,
            rs_value,
            rt,
            rt_value,
            imm,
        },
    )
}
