//! This tests records how the RSP registers are mirrored over the whole range they're accessible from.
//!
//! No surprises:
//! - the registers are mirrored every 8 words without gaps or unexpected patterns

#![no_std]
#![no_main]

test_suite_rom::run_test! {
    TestNoParams RspRegistersMirroring {
        fn run(result: &mut TestCaseResult) {
            // Give known values to the registers when possible to make them a bit more recognizable in the output

            unsafe {
                // TODO no readback of addr regs?

                reg_mut_ptr(specs::rsp::Status::ADDRESS).write_volatile(
                    specs::rsp::StatusWrite::default()
                        .with_clear_sig7(true)
                        .with_set_sig6(true)
                        .with_clear_sig5(true)
                        .with_set_sig4(true)
                        .with_clear_sig3(true)
                        .with_set_sig2(true)
                        .with_clear_sig1(true)
                        .with_set_sig0(true)
                        .with_set_interrupt_on_break(true)
                        .with_clear_single_step(true)
                        .with_set_halt(true)
                        .raw_value()
                );

                reg_mut_ptr(specs::rsp::Semaphore::ADDRESS).read_volatile(); // Switch the semaphore to 1
            }

            // TODO just begin and end?

            for address in (specs::rsp::REGISTERS_START..specs::rsp::REGISTERS_END).step_by(4) {
                result.push_memory(address);
            }
        }
    }
}
