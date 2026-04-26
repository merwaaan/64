use crate::{State, Test, TestResult};

pub struct RegistersMirroring;

impl Test for RegistersMirroring {
    type Params = ();

    fn run() -> TestResult {
        // TODO disable DMA first?

        let mut regs = n64_specs::ai::Registers::default();

        // TODO pick values
        // regs.dma_length.set_value(100);
        // regs.dma_ram_address.set_value(0x10000000);
        // regs.control.set_value(1);
        // regs.status.set_value(0);
        // regs.dac_rate.set_value(1000);
        // regs.bit_rate.set_value(1000);

        let mut result = TestResult::default();

        for address in n64_specs::ai::START..=n64_specs::ai::END {
            let value = 0; // read(addr);

            result.states.push(State::Memory { address, value });
        }

        result
    }
}
