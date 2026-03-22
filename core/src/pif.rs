use crate::{controller::Button, location::Location, system::System, value::Value};

const START: u32 = 0x1FC0_07C0;
const END: u32 = 0x1FC0_0800;

pub type PifRamLocation = Location<START, END>;

/// https://n64brew.dev/wiki/Joybus_Protocol

pub struct Pif {
    ram: [u8; 0x40],

    channel_offsets: [Option<usize>; 4],
}

impl Default for Pif {
    fn default() -> Self {
        Self {
            ram: [0; 0x40],
            channel_offsets: [None; 4],
        }
    }
}

impl Pif {
    pub fn read<T: Value>(s: &mut System, addr: PifRamLocation) -> T {
        for channel in 0..4 {
            if let Some(offset) = s.pif.channel_offsets[channel] {
                let data = &mut s.pif.ram[offset..offset + 4];

                data[0] = ((s.controllers[channel].pressed(Button::A) as u8) << 7)
                    | ((s.controllers[channel].pressed(Button::B) as u8) << 6)
                    | ((s.controllers[channel].pressed(Button::Z) as u8) << 5)
                    | ((s.controllers[channel].pressed(Button::Start) as u8) << 4)
                    | ((s.controllers[channel].pressed(Button::DUp) as u8) << 3)
                    | ((s.controllers[channel].pressed(Button::DDown) as u8) << 2)
                    | ((s.controllers[channel].pressed(Button::DLeft) as u8) << 1)
                    | (s.controllers[channel].pressed(Button::DRight) as u8);

                data[1] = ((s.controllers[channel].pressed(Button::LeftTrigger) as u8) << 5)
                    | ((s.controllers[channel].pressed(Button::RightTrigger) as u8) << 4);

                data[2] = 0;
                data[3] = 0;
            }
        }

        T::read_mem(&s.pif.ram, addr.relative())
    }

    // TODO method?
    pub fn write<T: Value>(s: &mut System, addr: PifRamLocation, data: T) {
        data.write_mem(&mut s.pif.ram, addr.relative());

        // Last byte written to: process the command buffer

        if addr.relative() <= 0x3F && 0x3F < addr.relative() + T::BYTES as u32 {
            Self::process_command_buffer(s);
        }
    }

    fn process_command_buffer(s: &mut System) {
        let control_byte = s.pif.ram[0x3F];

        //log::error!("PIF: COMMAND {:08X} @ {:08X}", control_byte, s.cpu.regs.pc);

        // for i in s.pif.data.iter() {
        //     log::error!("PIF: - {:X}", i);
        // }

        // https://n64brew.dev/wiki/PIF-NUS

        match control_byte {
            //Configure Joybus frame
            1 => {
                s.pif.channel_offsets = [None; 4];

                let mut offset = 0;
                let mut channel = 0;

                while offset < 0x40 {
                    let tx = s.pif.ram[offset] as usize; //TODO mask 3f?
                    //log::debug!("PIF??? {:X}", tx);

                    // Skip channel
                    if tx == 0 {
                        //log::debug!("   SKIP");
                        channel += 1;
                        offset += 1;
                    }
                    // ???
                    else if tx == 0xFF {
                        //log::debug!("  WEIRD SKIP");
                        offset += 1;
                    }
                    // ???
                    // else if tx == 0xFD {
                    //     //log::debug!("  WEIRD SKIP");
                    //     offset += 1;
                    // }
                    // End of command buffer
                    else if tx == 0xFE {
                        //log::debug!("  END");
                        break;
                    }
                    // Command
                    else {
                        let rx = (s.pif.ram[offset + 1] & 0x3F) as usize;
                        // TODO Bit 7 = no response, bit 6 = error???
                        //log::debug!("   COMMAND {} {}", tx, rx);

                        let tx_data = &s.pif.ram[offset + 2..offset + 2 + tx];
                        //log::debug!("   TX DATA {:?}", tx_data);

                        // Info
                        if tx_data[0] == 0 {
                            s.pif.ram[offset + 2 + tx] = 0x05;
                            s.pif.ram[offset + 2 + tx + 1] = 0x00;
                            s.pif.ram[offset + 2 + tx + 2] = 0x02;
                        }
                        // Controller state
                        else if tx_data[0] == 1 {
                            // s.pif.ram[offset + 2 + tx] = s.cpu.cycles as u8; // TODO temp, 0x10 = start
                            // s.pif.ram[offset + 2 + tx + 1] = 0;
                            // s.pif.ram[offset + 2 + tx + 2] = 0;
                            // s.pif.ram[offset + 2 + tx + 3] = 0;

                            s.pif.channel_offsets[channel] = Some(offset + 2 + tx);
                            channel += 1;
                        } else {
                            log::warn!("PIF: UNKNOWN COMMAND TYPE {:08X}", tx_data[0]);
                            break;
                        }

                        offset += 2 + tx + rx;
                    }
                }
            }
            8 => log::info!("PIF: terminate boot process"),
            0x10 => log::info!("PIF: ROM lockout"),
            0x20 => log::info!("PIF: acquire checksum"),
            0x40 => log::info!("PIF: run checksum"),
            _ => log::warn!("PIF: unknown command {:08X}", control_byte),
        }

        // TODO raise SI here in case CPU writes directly???

        // Clear the command byte

        s.pif.ram[0x3F] = 0;
    }
}
