use crate::{data::Value, map::Location, system::System};

const START: u32 = 0x1FC0_07C0;
const END: u32 = 0x1FC0_0800;

pub type PifRamLocation = Location<START, END>;

pub struct Pif {
    data: [u8; 0x40],
}

impl Default for Pif {
    fn default() -> Self {
        Self { data: [0; 0x40] }
    }
}

impl Pif {
    pub fn read<T: Value>(&self, addr: PifRamLocation) -> T {
        T::read_mem(&self.data, addr.relative())
    }

    // TODO method?
    pub fn write<T: Value>(s: &mut System, addr: PifRamLocation, data: T) {
        data.write_mem(&mut s.map.pif.data, addr.relative());

        // Last byte written to: process the command buffer

        if addr.relative() <= 0x3F && 0x3F < addr.relative() + T::BYTES as u32 {
            Self::process_command_buffer(s);
        }
    }

    fn process_command_buffer(s: &mut System) {
        let control_byte = s.map.pif.data[0x3F];

        //log::error!("PIF COMMAND {:08X} @ {:08X}", control_byte, s.cpu.regs.pc);

        // for i in s.map.pif.data.iter() {
        //     log::error!("PIF - {:X}", i);
        // }

        if control_byte == 1 {
            let mut offset = 0;
            let mut channel = 0;

            while offset < 0x40 {
                let tx = s.map.pif.data[offset] as usize;
                //log::error!("PIF??? {:X}", tx);

                // Skip channel
                if tx == 0 {
                    //log::error!("   SKIP");
                    channel += 1;
                    offset += 1;
                }
                // ???
                else if tx == 0xFF {
                    //log::error!("  WEIRD SKIP");
                    offset += 1;
                }
                // End of command buffer
                else if tx == 0xFE {
                    //log::error!("  END");
                    break;
                }
                // Command
                else {
                    let rx = s.map.pif.data[offset + 1] as usize;
                    //log::error!("   COMMAND {} {}", tx, rx);

                    let tx_data = &s.map.pif.data[offset + 2..offset + 2 + tx];
                    //log::error!("   TX DATA {:?}", tx_data);

                    // Info
                    if tx_data[0] == 0 {
                        log::error!(" PIF INFO");
                        s.map.pif.data[offset + 2 + tx] = 0x05;
                        s.map.pif.data[offset + 2 + tx + 1] = 0x00;
                        s.map.pif.data[offset + 2 + tx + 2] = 0x01;
                    } else if tx_data[0] == 1 {
                        log::error!(" PIF CONTROLLER");
                        s.map.pif.data[offset + 2 + tx] = 0x90; // TODO
                        s.map.pif.data[offset + 2 + tx + 1] = 0;
                        s.map.pif.data[offset + 2 + tx + 2] = 0;
                        s.map.pif.data[offset + 2 + tx + 3] = 0;
                    } else {
                        log::error!("  UNKNOWN COMMAND TYPE {:08X}", tx_data[0]);
                        break;
                    }

                    offset += 2 + tx + rx;
                }
            }
        } else {
            log::error!("  UNKNOWN COMMAND {:08X}", control_byte);
        }

        // TODO raise SI here in case CPU writes directly???

        // Clear the command byte

        s.map.pif.data[0x3F] = 0;
    }
}
