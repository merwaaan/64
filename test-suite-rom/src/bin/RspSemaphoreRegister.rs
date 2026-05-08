//! This test records the behavior of the RSP semaphore register.
//!
//! Findings:
//! - The written value is irrelevant, even zero clears the semaphore
//!
//! No surprises:
//! - Reads return the current value and set the register to 1
//! - Writes set the register to 0

// TODO reg masks
// TODO DMA wrapping
// TODO DMA double buff
// TODO reg mirroring

#![no_std]
#![no_main]

test_suite_rom::run_test! {
    TestWithParams RspSemaphoreRegister {
        type Params = u32;

        fn cases() -> Vec<Self::Params> {
            vec![0, 1, 0x1234_5678, 0x8000_0000, 0xFFFF_FFFF]
        }

        fn case_name(value: &u32) -> String {
            format!("Write {:08X}", value)
        }

        fn run(value: &u32, result: &mut TestCaseResult) {
            unsafe {
                let reg_ptr = reg_mut_ptr(specs::rsp::Semaphore::ADDRESS);

                result.push_comment("Clear");
                reg_ptr.write_volatile(0);

                result.push_comment("Read a few times");
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());

                result.push_comment("Write the value and read a few times");
                reg_ptr.write_volatile(*value);
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());

                result.push_comment("Write the value multiple times before reading again");
                reg_ptr.write_volatile(*value);
                reg_ptr.write_volatile(*value);
                reg_ptr.write_volatile(*value);
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());

                result.push_comment("Write different values before reading again");
                reg_ptr.write_volatile(0xAAAA_AAAA);
                reg_ptr.write_volatile(*value);
                reg_ptr.write_volatile(0xBBBB_BBBB);
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());
                result.push_value(reg_ptr.read_volatile());
            };
        }
    }
}
