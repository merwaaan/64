#![no_std]
#![no_main]

test_suite_rom::define_test! {
    AiRegistersMirroring {
        type Params = ();

        fn run_case(_params: &Self::Params, result: &mut TestCaseResult) {
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

            for address in (specs::ai::START..specs::ai::START + 0x00_1000).step_by(4) {
                result.push_memory(address);
            }
        }
    }
}
