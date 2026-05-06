#![no_std]
#![no_main]

// TODO same for mask register

test_suite_rom::define_test! {
    MiModeRegisterClearSet {
        type Params = ();

        fn run_case(_params: &Self::Params, result: &mut TestCaseResult) {

            let mode_reg = reg_mut_ptr(specs::mi::Mode::ADDRESS);

            // Initialize as cleared

            result.push_comment("Clear all");

            unsafe {
                mode_reg.write_volatile(0x0000_1280);
            }

            result.push_memory(mode_reg as u32);

            // Set and clear

            result.push_comment("Set + clear");

            unsafe {
                mode_reg.write_volatile(0x0000_3780);
            }

            result.push_memory(mode_reg as u32);

            // Initialize as set

            result.push_comment("Set all");

            unsafe {
                mode_reg.write_volatile(0x0000_2500);
            }

            result.push_memory(mode_reg as u32);

            // Set and clear

            result.push_comment("Set + clear");

            unsafe {
                mode_reg.write_volatile(0x0000_3780);
            }

            result.push_memory(mode_reg as u32);
        }
    }
}
