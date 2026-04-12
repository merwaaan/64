//! TODO doc
//!
//! https://n64brew.dev/wiki/Joybus_Protocol

use crate::{
    blocks::{read_block, write_block},
    controller::{Button, Controller},
    location::Location,
    value::Value,
};

const START: u32 = 0x1FC0_07C0;
const END: u32 = 0x1FC0_0800;

pub type PifRamLocation = Location<START, END>;

pub struct Pif {
    ram: [u8; 0x40],

    channel_offsets: [Option<usize>; 4], // TODO 5?
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
    pub fn read<T: Value>(&mut self, controllers: &[Controller], addr: PifRamLocation) -> T {
        self.probe_controllers(controllers);

        T::read_mem(&self.ram, addr.relative())
    }

    pub fn write<T: Value>(&mut self, addr: PifRamLocation, data: T) {
        data.write_mem(&mut self.ram, addr.relative());

        // Last byte written to: process the command buffer

        if addr.relative() <= 0x3F && 0x3F < addr.relative() + T::BYTES as u32 {
            self.process_command_buffer();
        }
    }

    pub fn read_block(
        &mut self,
        controllers: &[Controller],
        addr: PifRamLocation,
        length: usize,
        callback: impl FnMut(&[u8]),
    ) {
        self.probe_controllers(controllers);

        read_block(&self.ram, addr.relative() as usize, length, callback);
    }

    pub fn write_block(&mut self, addr: PifRamLocation, src: &[u8]) {
        write_block(src, &mut self.ram, addr.relative() as usize);

        // Last byte written to: process the command buffer

        if addr.relative() <= 0x3F && 0x3F < addr.relative() + src.len() as u32 {
            self.process_command_buffer();
        }
    }

    fn probe_controllers(&mut self, controllers: &[Controller]) {
        for channel in 0..4 {
            if let Some(offset) = self.channel_offsets[channel] {
                let data = &mut self.ram[offset..offset + 4];

                data[0] = ((controllers[channel].pressed(Button::A) as u8) << 7)
                    | ((controllers[channel].pressed(Button::B) as u8) << 6)
                    | ((controllers[channel].pressed(Button::Z) as u8) << 5)
                    | ((controllers[channel].pressed(Button::Start) as u8) << 4)
                    | ((controllers[channel].pressed(Button::DUp) as u8) << 3)
                    | ((controllers[channel].pressed(Button::DDown) as u8) << 2)
                    | ((controllers[channel].pressed(Button::DLeft) as u8) << 1)
                    | (controllers[channel].pressed(Button::DRight) as u8);

                data[1] = ((controllers[channel].pressed(Button::LeftTrigger) as u8) << 5)
                    | ((controllers[channel].pressed(Button::RightTrigger) as u8) << 4);

                data[2] = 0;
                data[3] = 0;

                // TODO write 0x80 = no response if no device connected?
            }
        }
    }

    fn process_command_buffer(&mut self) {
        let control_byte = self.ram[0x3F];

        //log::error!("PIF RAM: {:X?}", self.ram);

        // https://n64brew.dev/wiki/PIF-NUS

        match control_byte {
            // Configure Joybus frame
            1 => {
                self.channel_offsets = [None; 4];

                let mut offset = 0;
                let mut channel = 0;

                while offset < 0x40 && channel < 5 {
                    let tx = self.ram[offset] as usize;

                    //log::debug!("PIF byte: {:X}", tx);

                    // TODO should use high bits instead? https://n64brew.dev/wiki/PIF-NUS#RX_byte:_special_flags

                    match tx {
                        // Skip channel
                        0 => {
                            //log::debug!("   SKIP");
                            channel += 1;
                            offset += 1;
                        }

                        // Skip byte
                        // TODO diff between the two values unclear
                        0xFD | 0xFF => {
                            offset += 1;
                        }

                        // End of commands
                        0xFE => {
                            break;
                        }

                        // Command
                        _ => {
                            let rx = self.ram[offset + 1] as usize;

                            // The special "stop" byte is documented as appearing in TX spots, but it sometimes seems to appear in RX spots too.
                            //
                            // This often happens in that particular context:
                            // - a game writes a "write pak" command
                            // - after the (1 + 1 + 35 + 1) command bytes, there's a large value (often B4/F4) in place of the next TX
                            // - the following RX is 0xFE
                            //   -> if we don't stop here, we'll read the next B4/F4 & 3F bytes that follow and overrun the buffer
                            //
                            // Happens in Bust-A-Move 99 , In-Fisherman, and other games.
                            //
                            // It's currently unclear why this happens and my blurb about a "stop byte in RX" might be a misinterpretation of the protocol.

                            if rx == 0xFE {
                                break;
                            }

                            // Mask out the top 2 bits

                            let tx = tx & 0x3F;
                            let rx = rx & 0x3F;

                            // TODO Bit 7 = no response, bit 6 = error???
                            //log::debug!("   COMMAND {} {}", tx, rx);

                            let tx_data = &self.ram[offset + 2..offset + 2 + tx];

                            // Info / Reset & info
                            match tx_data[0] {
                                0x00 | 0xFF => {
                                    // TODO move up in probe?
                                    self.ram[offset + 2 + tx] = 0x05;
                                    self.ram[offset + 2 + tx + 1] = 0x00;
                                    self.ram[offset + 2 + tx + 2] = 0x02; // no accessory
                                }

                                // Controller state
                                1 => {
                                    self.channel_offsets[channel] = Some(offset + 2 + tx);
                                }
                                // Read pak
                                2 => {
                                    // TODO zero out for now

                                    for i in 0..rx {
                                        self.ram[offset + 2 + tx + i] = 0;
                                    }

                                    // TODO write CRC to rx?
                                }
                                // Write pak
                                3 => {
                                    log::warn!("PIF: pak write");

                                    // TODO do it

                                    // TODO write CRC to rx?
                                }

                                _ => {
                                    log::warn!("PIF: UNKNOWN COMMAND TYPE {:08X}", tx_data[0]);
                                    break;
                                }
                            }

                            offset += 2 + tx + rx;
                            channel += 1;
                        }
                    }
                }
            }

            8 => log::debug!("PIF: terminate boot process"),
            0x10 => log::debug!("PIF: ROM lockout"),
            0x20 => log::debug!("PIF: acquire checksum"),
            0x40 => log::debug!("PIF: run checksum"),

            _ => log::warn!("PIF: unknown command {:08X}", control_byte),
        }

        // TODO raise SI here in case CPU writes directly???

        // Clear the command byte

        self.ram[0x3F] = 0;
    }
}
