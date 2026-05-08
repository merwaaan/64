#![no_std]
#![no_main]

test_suite_rom::run_test! {
    TestNoParams AiLengthRegisterMasking {
        fn run(result: &mut TestCaseResult) {
            // Disable DMA

            let control_reg = reg_mut_ptr(specs::ai::Control::ADDRESS);

            unsafe {
                control_reg.write_volatile(specs::ai::Control::default().with_dma_enabled(false).raw_value());
            }

            let length_reg = reg_mut_ptr(specs::ai::DmaLength::ADDRESS);

            unsafe {
                // Write 0 to the length

                result.push_comment("Write 0x0000_0000 to the AI DMA length register");

                length_reg.write_volatile(0x0000_0000);

                result.push_memory(length_reg as u32);

                // Write u32::MAX to the length

                result.push_comment("Write 0xFFFF_FFFF to the AI DMA length register");

                length_reg.write_volatile(0xFFFF_FFFF);

                result.push_memory(length_reg as u32);
            }
        }
    }
}
