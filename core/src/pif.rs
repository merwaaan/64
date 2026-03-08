use std::cell::RefCell;

use crate::{data::Value, location::Location, system::System};

const START: u32 = 0x1FC0_07C0;
const END: u32 = 0x1FC0_0800;

pub type PifRamLocation = Location<START, END>;

/// https://n64brew.dev/wiki/Joybus_Protocol
///

pub struct Pif {
    ram: RefCell<[u8; 0x40]>,

    channel_offsets: RefCell<[Option<usize>; 4]>,
}

impl Default for Pif {
    fn default() -> Self {
        Self {
            ram: RefCell::new([0; 0x40]),
            channel_offsets: RefCell::new([None; 4]),
        }
    }
}

impl Pif {
    pub fn read<T: Value>(s: &System, addr: PifRamLocation) -> T {
        {
            Self::random_inputs(s);
        }

        T::read_mem(s.pif.ram.borrow().as_ref(), addr.relative())
    }

    // TODO method?
    pub fn write<T: Value>(s: &mut System, addr: PifRamLocation, data: T) {
        data.write_mem(s.pif.ram.borrow_mut().as_mut(), addr.relative());

        // Last byte written to: process the command buffer

        if addr.relative() <= 0x3F && 0x3F < addr.relative() + T::BYTES as u32 {
            Self::process_command_buffer(s);
        }
    }

    fn process_command_buffer(s: &mut System) {
        let mut ram: std::cell::RefMut<'_, [u8; 64]> = s.pif.ram.borrow_mut();

        let control_byte = ram[0x3F];

        //log::error!("PIF COMMAND {:08X} @ {:08X}", control_byte, s.cpu.regs.pc);

        // for i in s.pif.data.iter() {
        //     log::error!("PIF - {:X}", i);
        // }

        if (control_byte & 1) != 0 {
            // Reset the channel offsets
            let mut channel_offsets = [None; 4];

            let mut offset = 0;
            let mut channel = 0;

            while offset < 0x40 {
                let tx = ram[offset] as usize; //TODO mask 3f?
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
                // ???
                // else if tx == 0xFD {
                //     //log::error!("  WEIRD SKIP");
                //     offset += 1;
                // }
                // End of command buffer
                else if tx == 0xFE {
                    //log::error!("  END");
                    break;
                }
                // Command
                else {
                    let rx = (ram[offset + 1] & 0x3F) as usize;
                    // TODO Bit 7 = no response, bit 6 = error???
                    //log::error!("   COMMAND {} {}", tx, rx);

                    let tx_data = &ram[offset + 2..offset + 2 + tx];
                    //log::error!("   TX DATA {:?}", tx_data);

                    // Info
                    if tx_data[0] == 0 {
                        log::error!(" PIF INFO");
                        ram[offset + 2 + tx] = 0x05;
                        ram[offset + 2 + tx + 1] = 0x00;
                        ram[offset + 2 + tx + 2] = 0x02;
                    }
                    // Controller state
                    else if tx_data[0] == 1 {
                        log::error!(" PIF CONTROLLER");
                        // s.pif.ram[offset + 2 + tx] = s.cpu.cycles as u8; // TODO temp, 0x10 = start
                        // s.pif.ram[offset + 2 + tx + 1] = 0;
                        // s.pif.ram[offset + 2 + tx + 2] = 0;
                        // s.pif.ram[offset + 2 + tx + 3] = 0;

                        channel_offsets[channel] = Some(offset + 2 + tx);
                        channel += 1;
                    } else {
                        log::error!("  UNKNOWN COMMAND TYPE {:08X}", tx_data[0]);
                        break;
                    }

                    offset += 2 + tx + rx;
                }

                s.pif.channel_offsets.replace(channel_offsets);
            }
        } else {
            log::error!("  UNKNOWN COMMAND {:08X}", control_byte);
        }

        // TODO raise SI here in case CPU writes directly???

        // Clear the command byte

        ram[0x3F] = 0;
    }

    fn random_inputs(s: &System) {
        let mut ram = s.pif.ram.borrow_mut();
        let channel_offsets = s.pif.channel_offsets.borrow();
        //log::error!("PIF RANDOM INPUTS {:?}", channel_offsets);

        for channel in 0..4 {
            if let Some(offset) = channel_offsets[channel] {
                let data = &mut ram[offset..offset + 4];

                data[0] = 1 << ((offset * 3 + 15 + offset + s.cpu.cycles) % 8) as u8; // TODO temp, 0x10 = start
                data[1] = 0;
                data[2] = 0;
                data[3] = 0;
            }
        }
    }
}
