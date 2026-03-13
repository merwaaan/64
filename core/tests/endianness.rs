use n64_core::value::Value;

mod byte {
    use super::*;

    #[test]
    fn read_mem() {
        let mem = [0x12, 0x34, 0x56, 0x78, 0xAA];

        assert_eq!(u8::read_mem(&mem, 0), 0x12);
        assert_eq!(u8::read_mem(&mem, 1), 0x34);
        assert_eq!(u8::read_mem(&mem, 2), 0x56);
        assert_eq!(u8::read_mem(&mem, 3), 0x78);
        assert_eq!(u8::read_mem(&mem, 4), 0xAA);
    }

    #[test]
    fn write_mem() {
        let mut mem = [0x12, 0x34, 0x56, 0x78, 0xAA];

        u8::write_mem(0x00, &mut mem, 0);
        assert_eq!(mem, [0x00, 0x34, 0x56, 0x78, 0xAA]);

        u8::write_mem(0x01, &mut mem, 1);
        assert_eq!(mem, [0x00, 0x01, 0x56, 0x78, 0xAA]);

        u8::write_mem(0x03, &mut mem, 4);
        assert_eq!(mem, [0x00, 0x01, 0x56, 0x78, 0x03]);
    }

    #[test]
    fn read_reg() {
        let regs = [0x1234_5678, 0xAABB_CCDD];

        assert_eq!(u8::read_reg(&regs, 0), 0x12);
        assert_eq!(u8::read_reg(&regs, 1), 0x34);
        assert_eq!(u8::read_reg(&regs, 3), 0x78);
        assert_eq!(u8::read_reg(&regs, 4), 0xAA);
        assert_eq!(u8::read_reg(&regs, 7), 0xDD);
    }

    #[test]
    fn write_reg() {
        let mut regs = [0x1234_5678, 0xAABB_CCDD];

        u8::write_reg(0x00, &mut regs, 0);
        assert_eq!(regs, [0x0034_5678, 0xAABB_CCDD]);

        u8::write_reg(0x01, &mut regs, 1);
        assert_eq!(regs, [0x0001_5678, 0xAABB_CCDD]);

        u8::write_reg(0x02, &mut regs, 3);
        assert_eq!(regs, [0x0001_5602, 0xAABB_CCDD]);

        u8::write_reg(0x03, &mut regs, 6);
        assert_eq!(regs, [0x0001_5602, 0xAABB_03DD]);
    }
}

mod half {
    use super::*;

    #[test]
    fn read_mem() {
        let mem = [0x12, 0x34, 0x56, 0x78];

        assert_eq!(u16::read_mem(&mem, 0), 0x1234);
        assert_eq!(u16::read_mem(&mem, 2), 0x5678);
    }

    #[test]
    fn write_mem() {
        let mut mem = [0x12, 0x34, 0x56, 0x78];

        u16::write_mem(0xABCD, &mut mem, 0);
        assert_eq!(mem, [0xAB, 0xCD, 0x56, 0x78]);

        u16::write_mem(0x6667, &mut mem, 2);
        assert_eq!(mem, [0xAB, 0xCD, 0x66, 0x67]);
    }

    #[test]
    fn read_reg() {
        let regs = [0x1234_5678, 0xAABB_CCDD];

        assert_eq!(u16::read_reg(&regs, 0), 0x1234);
        assert_eq!(u16::read_reg(&regs, 2), 0x5678);
        assert_eq!(u16::read_reg(&regs, 4), 0xAABB);
        assert_eq!(u16::read_reg(&regs, 6), 0xCCDD);
    }

    #[test]
    fn write_reg() {
        let mut regs = [0x1234_5678, 0xAABB_CCDD];

        u16::write_reg(0xABCD, &mut regs, 0);
        assert_eq!(regs, [0xABCD_5678, 0xAABB_CCDD]);

        u16::write_reg(0xEF00, &mut regs, 2);
        assert_eq!(regs, [0xABCD_EF00, 0xAABB_CCDD]);

        u16::write_reg(0x6667, &mut regs, 4);
        assert_eq!(regs, [0xABCD_EF00, 0x6667_CCDD]);

        u16::write_reg(0x1234, &mut regs, 6);
        assert_eq!(regs, [0xABCD_EF00, 0x6667_1234]);
    }
}

mod word {
    use super::*;

    #[test]
    fn read_mem() {
        let mem = [0x12, 0x34, 0x56, 0x78, 0xAA, 0xBB, 0xCC, 0xDD];

        assert_eq!(u32::read_mem(&mem, 0), 0x1234_5678);
        assert_eq!(u32::read_mem(&mem, 4), 0xAABB_CCDD);
    }

    #[test]
    fn write_mem() {
        let mut mem = [0x12, 0x34, 0x56, 0x78, 0xAA, 0xBB, 0xCC, 0xDD];

        u32::write_mem(0x0011_2233, &mut mem, 0);
        assert_eq!(mem, [0x00, 0x11, 0x22, 0x33, 0xAA, 0xBB, 0xCC, 0xDD]);

        u32::write_mem(0xEEEE_FFFF, &mut mem, 4);
        assert_eq!(mem, [0x00, 0x11, 0x22, 0x33, 0xEE, 0xEE, 0xFF, 0xFF]);
    }

    #[test]
    fn read_reg() {
        let regs = [0x1234_5678, 0xAABB_CCDD];

        assert_eq!(u32::read_reg(&regs, 0), 0x1234_5678);
        assert_eq!(u32::read_reg(&regs, 4), 0xAABB_CCDD);
    }

    #[test]
    fn write_reg() {
        let mut regs = [0x1234_5678, 0xAABB_CCDD];

        u32::write_reg(0x0011_2233, &mut regs, 0);
        assert_eq!(regs, [0x0011_2233, 0xAABB_CCDD]);

        u32::write_reg(0x9876_5432, &mut regs, 4);
        assert_eq!(regs, [0x0011_2233, 0x9876_5432]);
    }
}

mod double {
    use super::*;

    #[test]
    fn read_mem() {
        let mem = [
            0x12, 0x34, 0x56, 0x78, 0xAA, 0xBB, 0xCC, 0xDD, //
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        ];

        assert_eq!(u64::read_mem(&mem, 0), 0x1234_5678_AABB_CCDD);
        assert_eq!(u64::read_mem(&mem, 8), 0x0011_2233_4455_6677);
    }

    #[test]
    fn write_mem() {
        let mut mem = [
            0x12, 0x34, 0x56, 0x78, 0xAA, 0xBB, 0xCC, 0xDD, //
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        ];

        u64::write_mem(0xFEDC_BA98_7654_3210, &mut mem, 0);
        assert_eq!(
            mem,
            [
                0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10, //
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            ]
        );

        u64::write_mem(0x1234_5678_AABB_CCDD, &mut mem, 8);
        assert_eq!(
            mem,
            [
                0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10, //
                0x12, 0x34, 0x56, 0x78, 0xAA, 0xBB, 0xCC, 0xDD,
            ]
        );
    }

    #[test]
    fn read_reg() {
        let regs = [0x1234_5678, 0xAABB_CCDD, 0x0011_2233, 0x4455_6677];

        assert_eq!(u64::read_reg(&regs, 0), 0x1234_5678_AABB_CCDD);
        assert_eq!(u64::read_reg(&regs, 8), 0x0011_2233_4455_6677);
    }

    #[test]
    fn write_reg() {
        let mut regs = [0x1234_5678, 0xAABB_CCDD, 0x0011_2233, 0x4455_6677];

        u64::write_reg(0xFEDC_BA98_7654_3210, &mut regs, 0);
        assert_eq!(regs, [0xFEDC_BA98, 0x7654_3210, 0x0011_2233, 0x4455_6677]);

        u64::write_reg(0x1234_5678_AABB_CCDD, &mut regs, 8);
        assert_eq!(regs, [0xFEDC_BA98, 0x7654_3210, 0x1234_5678, 0xAABB_CCDD]);
    }
}

mod registers {
    use arbitrary_int::prelude::*;
    use n64_core::vi::Registers;

    //TODO use offset macros

    #[test]
    fn read() {
        let mut regs = Registers::default();
        regs.interrupt_line.set_value(u10::new(0x173));
        regs.current_line.set_line(u9::new(0x1A7));
        regs.current_line.set_field(u1::new(1));

        assert_eq!(regs.read::<u8>(0), 0);
        assert_eq!(regs.read::<u8>(1), 0);
        assert_eq!(regs.read::<u8>(2), 0);
        assert_eq!(regs.read::<u8>(3), 0);
        assert_eq!(regs.read::<u8>(3 * 4), 0);
        assert_eq!(regs.read::<u8>(3 * 4 + 1), 0);
        assert_eq!(regs.read::<u8>(3 * 4 + 2), 1);
        assert_eq!(regs.read::<u8>(3 * 4 + 3), 0x73);
        assert_eq!(regs.read::<u8>(4 * 4), 0);
        assert_eq!(regs.read::<u8>(4 * 4 + 1), 0);
        assert_eq!(regs.read::<u8>(4 * 4 + 2), 3);
        assert_eq!(regs.read::<u8>(4 * 4 + 3), 0x4F);

        assert_eq!(regs.read::<u16>(0), 0);
        assert_eq!(regs.read::<u16>(3 * 4).to_be_bytes(), [0, 0]);
        assert_eq!(regs.read::<u16>(3 * 4 + 1).to_be_bytes(), [0, 1]);
        assert_eq!(regs.read::<u16>(3 * 4 + 2).to_be_bytes(), [1, 0x73]);
        assert_eq!(regs.read::<u16>(3 * 4 + 3).to_be_bytes(), [0x73, 0]);
        assert_eq!(regs.read::<u16>(4 * 4).to_be_bytes(), [0, 0]);
        assert_eq!(regs.read::<u16>(4 * 4 + 1).to_be_bytes(), [0, 3]);
        assert_eq!(regs.read::<u16>(4 * 4 + 2).to_be_bytes(), [3, 0x4F]);
        assert_eq!(regs.read::<u16>(4 * 4 + 3).to_be_bytes(), [0x4F, 0]);

        assert_eq!(regs.read::<u32>(0), 0);
        assert_eq!(regs.read::<u32>(3 * 4).to_be_bytes(), [0, 0, 1, 0x73]);
        assert_eq!(regs.read::<u32>(3 * 4 + 3).to_be_bytes(), [0x73, 0, 0, 3]); // ----------------
        assert_eq!(regs.read::<u32>(4 * 4).to_be_bytes(), [0, 0, 3, 0x4F]);

        assert_eq!(regs.read::<u64>(0).to_be_bytes(), [0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            regs.read::<u64>(2 * 4).to_be_bytes(),
            [0, 0, 0, 0, 0, 0, 1, 0x73]
        );
        assert_eq!(
            regs.read::<u64>(3 * 4).to_be_bytes(),
            [0, 0, 1, 0x73, 0, 0, 3, 0x4F]
        );
        assert_eq!(
            regs.read::<u64>(3 * 4 + 3).to_be_bytes(),
            [0x73, 0, 0, 3, 0x4F, 0, 0, 0]
        );
        assert_eq!(
            regs.read::<u64>(4 * 4).to_be_bytes(),
            [0, 0, 3, 0x4F, 0, 0, 0, 0]
        );
    }

    // TODO write
}
