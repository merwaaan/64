#![no_std]
#![no_main]

// TODO DOC
// TODO what if writing to empty slots?

test_suite_rom::run_test! {
    TestNoParams AiRegistersMirroring {
        fn run(result: &mut TestCaseResult) {
            // TODO write length/status

            //let mut regs = n64_specs::ai::Registers::default();

            // TODO pick values
            // regs.dma_length.set_value(100);
            // regs.dma_ram_address.set_value(0x10000000);
            // regs.control.set_value(1);
            // regs.status.set_value(0);
            // regs.dac_rate.set_value(1000);
            // regs.bit_rate.set_value(1000);

            // TODO to end (but it freezes right now, uses too much mem?)

            // TODO use specs

            // TODO just begin and end?

            for address in (specs::ai::START..specs::ai::START + 0x00_1000).step_by(4) {
                result.push_memory(address);
            }
        }
    }
}
