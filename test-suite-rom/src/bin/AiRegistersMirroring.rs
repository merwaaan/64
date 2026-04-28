#![no_std]
#![no_main]

test_suite_rom::define_test! {
    AiRegistersMirroring {
        type Params = ();

        fn run_case(_params: &Self::Params, result: &mut TestCaseResult) {
            // TODO write length/status

            let mut regs = n64_specs::ai::Registers::default();

            // TODO pick values
            // regs.dma_length.set_value(100);
            // regs.dma_ram_address.set_value(0x10000000);
            // regs.control.set_value(1);
            // regs.status.set_value(0);
            // regs.dac_rate.set_value(1000);
            // regs.bit_rate.set_value(1000);

            // TODO to end (but it freezes right now, uses too much mem?)

            for address in (n64_specs::ai::START..n64_specs::ai::START + 0x00_1000).step_by(4) {
                let ptr = (0xA000_0000 | address) as *const u32;

                let value = unsafe { ptr.read_volatile() };

                result.states.push(State::Memory { address, value });
            }
        }
    }
}

// TODO Ai Status masking
// TODO Ai Length masking
// TODO ai DMA enabled
