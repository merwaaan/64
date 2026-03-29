use std::collections::VecDeque;

use arbitrary_int::prelude::*;
use bitbybit::{bitenum, bitfield};

use crate::{
    blocks::{read_block, write_block},
    location::Location,
    mi::Interrupt,
    ram::RamLocation,
    rendering::video::{self, QuadFill},
    sp::SpMemLocation,
    system::System,
    value::Value,
};

const REG_START: u32 = 0x0410_0000;
const REG_END: u32 = 0x0420_0000;
// TODO other regs after 20 0000?

pub type DpLocation = Location<REG_START, REG_END>;

const START_REG: u32 = 0;
const END_REG: u32 = 1;
const CURRENT_REG: u32 = 2;
const STATUS_REG: u32 = 3;
const _CLOCK_REG: u32 = 4;
const _BUF_BUSY_REG: u32 = 5;
const _PIPE_BUSY_REG: u32 = 6;
const _TMEM_BUSY_REG: u32 = 7;

const STATUS_XBUS: u32 = 1;
const STATUS_FREEZE: u32 = 1 << 1;
const STATUS_FLUSH: u32 = 1 << 2;
const _STATUS_GCLK: u32 = 1 << 3;
const _STATUS_TMEM_BUSY: u32 = 1 << 4;
const _STATUS_PIPE_BUSY: u32 = 1 << 5;
const _STATUS_CMD_BUSY: u32 = 1 << 6;
const _STATUS_CBUF_READY: u32 = 1 << 7;
const _STATUS_DMA_BUSY: u32 = 1 << 8;
const STATUS_END_PENDING: u32 = 1 << 9;
const STATUS_START_PENDING: u32 = 1 << 10;

const STATUS_XBUS_CLEAR: u32 = 1;
const STATUS_XBUS_SET: u32 = 1 << 1;
const STATUS_FREEZE_CLEAR: u32 = 1 << 2;
const STATUS_FREEZE_SET: u32 = 1 << 3;
const STATUS_FLUSH_CLEAR: u32 = 1 << 4;
const STATUS_FLUSH_SET: u32 = 1 << 5;
const _STATUS_TMEM_BUSY_CLEAR: u32 = 1 << 6;
const _STATUS_PIPE_BUSY_CLEAR: u32 = 1 << 7;
const _STATUS_BUF_BUSY_CLEAR: u32 = 1 << 8;
const _STATUS_CLK_CLEAR: u32 = 1 << 9;

// TODO double buffering

#[derive(Clone)]
pub struct Dp {
    // TODO struct regs
    pub regs: [u32; 8],

    /// Pending command buffer.
    ///
    /// We decode commands as soon as we receive them.
    /// However, commands might be split across multiple transfers so if we're missing some data to fully decode one,
    /// we buffer what we have until the remaining data is sent via additional transfers.
    command_buffer: VecDeque<u8>,

    /// Decoded commands.
    /// Applied when we receive a Sync full command.
    decoded_commands: Vec<Command>,

    /// Rendering state, updated by applied commands.
    state: State,

    /// Texture memory.
    pub tmem: [u8; 0x1000], // TODO vis
}

#[derive(Default, Clone, Copy)]
struct State {
    fill_color: [f32; 4],
    texture: SetTextureImage,

    tile_slots: [TileSlot; 8],
}

#[derive(Default, Clone, Copy)]
struct TileSlot {
    tile: SetTile,
    size: SetTileSize,
}

impl Default for Dp {
    fn default() -> Self {
        Self {
            regs: [0; 8],

            command_buffer: VecDeque::new(),
            decoded_commands: Vec::new(),

            tmem: [0; 0x1000],

            state: State::default(),
        }
    }
}

impl Dp {
    pub fn read<T: Value>(s: &System, addr: DpLocation) -> T {
        //log::warn!("read DP register {:08X}", addr.relative());

        // TODO possible to read mult regs???
        debug_assert!(T::BYTES <= 4, "Writing to multiple DP registers");

        T::read_reg(&s.dp.regs, addr.relative() & 0x1F)
    }

    pub fn write<T: Value>(s: &mut System, addr: DpLocation, data: T) {
        // log::warn!(
        //     "Write DP register {:08X} = {:X} / {}",
        //     addr.relative(),
        //     data,
        //     T::BYTES
        // );

        // TODO possible to write mult regs???
        debug_assert!(T::BYTES <= 4, "Writing to multiple DP registers");

        match (addr.relative() >> 2) & 7 {
            START_REG => {
                // log::debug!(
                //     "DP: Write START address {:08X} start pending={} end pending={} current={}",
                //     data,
                //     s.dp.regs[STATUS_REG as usize] & STATUS_START_PENDING != 0,
                //     s.dp.regs[STATUS_REG as usize] & STATUS_END_PENDING != 0,
                //     s.dp.regs[CURRENT_REG as usize]
                // );
                // Write the START address and make it "pending", ie. we're waiting for END to be written

                data.write_reg(&mut s.dp.regs, addr.relative() & 0x1F);

                s.dp.regs[START_REG as usize] &= 0x00FF_FFF8;
                s.dp.regs[STATUS_REG as usize] |= STATUS_START_PENDING;

                // TODO set current = start here?? not only when writing END? unclear
            }
            END_REG => {
                // log::debug!(
                //     "DP: Write END address {:08X} start pending={} end pending={} current={}",
                //     data,
                //     s.dp.regs[STATUS_REG as usize] & STATUS_START_PENDING != 0,
                //     s.dp.regs[STATUS_REG as usize] & STATUS_END_PENDING != 0,
                //     s.dp.regs[CURRENT_REG as usize]
                // );
                data.write_reg(&mut s.dp.regs, addr.relative() & 0x1F);

                s.dp.regs[END_REG as usize] &= 0x00FF_FFF8;

                // If START was "pending", set CURRENT to START and clear the pending flag.
                // If END is written again before START, the DMA will continue from CURRENT (ie. the previous END if the DMA completed).

                if s.dp.regs[STATUS_REG as usize] & STATUS_START_PENDING != 0 {
                    s.dp.regs[CURRENT_REG as usize] = s.dp.regs[START_REG as usize];
                    s.dp.regs[STATUS_REG as usize] &= !STATUS_START_PENDING;
                }

                s.dp.regs[STATUS_REG as usize] |= STATUS_END_PENDING;

                Self::start_dma(s);
            }
            STATUS_REG => {
                let mut status = s.dp.regs[STATUS_REG as usize];

                let mut trigger_bits = [0u32];
                data.write_reg(&mut trigger_bits, addr.relative() & 3);

                // XBUS

                if trigger_bits[0] & STATUS_XBUS_CLEAR != 0 {
                    status &= !STATUS_XBUS;
                }
                if trigger_bits[0] & STATUS_XBUS_SET != 0 {
                    status |= STATUS_XBUS;
                }

                // FREEZE

                if trigger_bits[0] & STATUS_FREEZE_CLEAR != 0 {
                    status &= !STATUS_FREEZE;
                }
                if trigger_bits[0] & STATUS_FREEZE_SET != 0 {
                    status |= STATUS_FREEZE;
                    log::warn!("DP FREEZE");
                }

                // FLUSH

                if trigger_bits[0] & STATUS_FLUSH_CLEAR != 0 {
                    status &= !STATUS_FLUSH;
                }
                if trigger_bits[0] & STATUS_FLUSH_SET != 0 {
                    status |= STATUS_FLUSH;
                    log::warn!("DP FLUSH");
                    // TODO do something?
                }

                // TODO?

                // // TMEM_BUSY

                // if trigger_bits[0] & STATUS_TMEM_BUSY_CLEAR != 0 {
                //     status &= !STATUS_TMEM_BUSY;
                // }

                // // PIPE_BUSY

                // if trigger_bits[0] & STATUS_PIPE_BUSY_CLEAR != 0 {
                //     status &= !STATUS_PIPE_BUSY;
                // }

                // // BUF_BUSY

                // if trigger_bits[0] & STATUS_BUF_BUSY_CLEAR != 0 {
                //     s.dp.regs[BUF_BUSY_REG as usize] = 0;
                // }

                // // CLK

                // if trigger_bits[0] & STATUS_CLK_CLEAR != 0 {
                //     s.dp.regs[CLOCK_REG as usize] = 0;
                // }

                s.dp.regs[STATUS_REG as usize] = status;
            }
            _ => {}
        }
    }

    fn start_dma(s: &mut System) {
        let from_sp = s.dp.regs[STATUS_REG as usize] & STATUS_XBUS != 0;

        let current = s.dp.regs[CURRENT_REG as usize];
        let end = s.dp.regs[END_REG as usize];

        // log::debug!(
        //     "DP: DMA (XBus={}): {:08X} -> {:08X} -> {:08X}",
        //     from_sp,
        //     s.dp.regs[START_REG as usize],
        //     current,
        //     end
        // );

        // TODO optim: pass the slice to decode (with pending data appended via iter)? only queue what couldn't be decoded?

        if from_sp {
            s.sp.read_block(
                SpMemLocation::from_relative(current), // TODO relative?
                (end - current) as usize,
                |sp_data| {
                    s.dp.command_buffer.extend(sp_data);
                },
            );
        } else {
            s.ram.read_block(
                RamLocation::from_relative(current), // TODO relative?
                (end - current) as usize,
                |ram_data| {
                    s.dp.command_buffer.extend(ram_data);
                },
            );
        }

        Self::decode_commands(s);

        s.dp.regs[CURRENT_REG as usize] = s.dp.regs[END_REG as usize]; // TODO latest addr? what if not "aligned"?

        s.dp.regs[STATUS_REG as usize] &= !STATUS_END_PENDING;
    }

    // TODO don't pass system, return special cases
    fn decode_commands(s: &mut System) {
        macro_rules! if_ready {
            ($n:expr, $body:block) => {
                if s.dp.command_buffer.len() < $n {
                    break;
                }

                $body

                s.dp.command_buffer.drain(0..$n);
            };
        }

        // TODO nicer decoder with (func, data length)?

        while let Some(first_byte) = s.dp.command_buffer.get(0).copied() {
            //log::warn!("DP:  {:?}", first_byte);
            // TODO fullsync: END_PENDING 0, DP int

            let mut loggg = String::new();

            match first_byte & 0x3F {
                0..=7 | 0x10..=0x23 | 0x31 => {
                    //log::debug!("DP: NOP");
                    if_ready!(8, {});
                }
                0x08..=0x0F => {
                    let shade = first_byte & 4 != 0;
                    let texture = first_byte & 2 != 0;
                    let zbuffer = first_byte & 1 != 0;

                    let mut size = 32;

                    if shade {
                        size += 64;
                    }

                    if texture {
                        size += 64;
                    }

                    if zbuffer {
                        size += 16;
                    }

                    loggg.push_str(&format!(
                        "DP: Fill triangle (S={}, T={}, Z={})",
                        zbuffer, texture, shade
                    ));

                    // TODO
                    if_ready!(size, {});
                }
                0x24 | 0x25 => {
                    if first_byte == 0x25 {
                        log::warn!("TextureRectangleFlip");
                    }

                    if_ready!(16, {
                        s.dp.decoded_commands.push(Command::TextureRectangle(
                            TextureRectangle::new_with_raw_value(u128::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                                *s.dp.command_buffer.get(8).unwrap(),
                                *s.dp.command_buffer.get(9).unwrap(),
                                *s.dp.command_buffer.get(10).unwrap(),
                                *s.dp.command_buffer.get(11).unwrap(),
                                *s.dp.command_buffer.get(12).unwrap(),
                                *s.dp.command_buffer.get(13).unwrap(),
                                *s.dp.command_buffer.get(14).unwrap(),
                                *s.dp.command_buffer.get(15).unwrap(),
                            ])),
                        ));
                    });
                }
                0x26 => {
                    loggg.push_str(&"DP: Sync Load");

                    if_ready!(8, {});
                }
                0x27 => {
                    loggg.push_str(&"DP: Sync Pipe");

                    if_ready!(8, {});
                }
                0x28 => {
                    loggg.push_str(&"DP: Sync Tile");

                    if_ready!(8, {});
                }
                0x29 => {
                    loggg.push_str(&"DP: Sync Full");

                    if_ready!(8, {
                        s.mi.set_pending_interrupt(Interrupt::Dp, &mut s.cop0); // TODO temp

                        Self::apply_command(s);
                    });
                }
                0x2A => {
                    loggg.push_str(&"DP: Set key GB");

                    if_ready!(8, {});
                }
                0x2B => {
                    loggg.push_str(&"DP: Set key R");

                    if_ready!(8, {});
                }
                0x2C => {
                    loggg.push_str(&"DP: Set convert");

                    if_ready!(8, {});
                }
                0x2D => {
                    loggg.push_str(&"DP: Set scissor");

                    if_ready!(8, {});
                }
                0x2E => {
                    loggg.push_str(&"DP: Set prim depth");

                    if_ready!(8, {});
                }
                0x2F => {
                    loggg.push_str(&"DP: Set other mode");

                    if_ready!(8, {});
                }
                0x30 => {
                    loggg.push_str(&"DP: Load TLUT");

                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::LoadTLUT(
                            LoadTile::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x32 => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetTileSize(
                            SetTileSize::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x33 => {
                    loggg.push_str(&"DP: Load block");
                    if_ready!(8, {});
                }
                0x34 => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::LoadTile(
                            LoadTile::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x35 => {
                    if_ready!(8, {
                        s.dp.decoded_commands
                            .push(Command::SetTile(SetTile::new_with_raw_value(
                                u64::from_be_bytes([
                                    *s.dp.command_buffer.get(0).unwrap(),
                                    *s.dp.command_buffer.get(1).unwrap(),
                                    *s.dp.command_buffer.get(2).unwrap(),
                                    *s.dp.command_buffer.get(3).unwrap(),
                                    *s.dp.command_buffer.get(4).unwrap(),
                                    *s.dp.command_buffer.get(5).unwrap(),
                                    *s.dp.command_buffer.get(6).unwrap(),
                                    *s.dp.command_buffer.get(7).unwrap(),
                                ]),
                            )));
                    });
                }
                0x36 => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::FillRectangle(
                            FillRectangle::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x37 => {
                    loggg.push_str(&"DP: set fill color");
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetFillColor(
                            SetFillColor::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x38 => {
                    loggg.push_str(&"DP: set fog color");
                    if_ready!(8, {});
                }
                0x39 => {
                    loggg.push_str(&"DP: set blend color");
                    if_ready!(8, {});
                }
                0x3A => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetPrimitiveColor(
                            SetPrimitiveColor::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x3B => {
                    loggg.push_str(&"DP: set env color");

                    if_ready!(8, {});
                }
                0x3C => {
                    loggg.push_str(&"DP: set combine");

                    if_ready!(8, {});
                }
                0x3D => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetTextureImage(
                            SetTextureImage::new_with_raw_value(u64::from_be_bytes([
                                *s.dp.command_buffer.get(0).unwrap(),
                                *s.dp.command_buffer.get(1).unwrap(),
                                *s.dp.command_buffer.get(2).unwrap(),
                                *s.dp.command_buffer.get(3).unwrap(),
                                *s.dp.command_buffer.get(4).unwrap(),
                                *s.dp.command_buffer.get(5).unwrap(),
                                *s.dp.command_buffer.get(6).unwrap(),
                                *s.dp.command_buffer.get(7).unwrap(),
                            ])),
                        ));
                    });
                }
                0x3E => {
                    loggg.push_str(&"DP: set zimg");

                    if_ready!(8, {});
                }
                0x3F => {
                    loggg.push_str(&"DP: set cimg");

                    if_ready!(8, {});
                }
                x => panic!("Unknown DP DMA command: {:X}", x),
            }

            if false && loggg.len() > 0 {
                log::debug!("{}", loggg);
            }
        }
    }

    fn apply_command(s: &mut System) {
        // if s.dp.decoded_commands.len() > 0 {
        //     log::info!("DP: DMA commands: {:#?}", s.dp.decoded_commands);
        // }

        //log::debug!("Applying commands ---------------------");

        for command in &s.dp.decoded_commands {
            //log::debug!("Applying command: {:#?}", command);

            match command {
                Command::SetFillColor(data) => {
                    s.dp.state.fill_color = convert_color(data.color());
                }

                Command::SetTextureImage(data) => {
                    s.dp.state.texture = *data;
                }

                Command::SetTile(data) => {
                    s.dp.state.tile_slots[data.tile().value() as usize].tile = *data;
                }

                Command::SetTileSize(data) => {
                    s.dp.state.tile_slots[data.tile().value() as usize].size = *data;
                }

                Command::LoadTLUT(data) => {
                    // Load a palette to TMEM

                    let slot = &s.dp.state.tile_slots[data.tile().value() as usize];

                    let left = data.upper_left_x().value();
                    let right = data.lower_right_x().value();

                    let width = (right >> 2).wrapping_sub(left >> 2) + 1; // TODO +1?

                    let bytes_to_copy = width * 2; // Palette colors are 16-bit

                    s.ram.read_block(
                        RamLocation::from_absolute(s.dp.state.texture.ram_address().value()),
                        bytes_to_copy as usize,
                        |data| {
                            write_block(data, &mut s.dp.tmem, slot.tile.tmem_address_byte());
                        },
                    );
                }

                Command::LoadTile(data) => {
                    // Load a tile from RAM to TMEM

                    let slot = &s.dp.state.tile_slots[data.tile().value() as usize];

                    // TODO fast-path for image width = tile width?

                    let image_width = s.dp.state.texture.width().value() as u32 + 1;

                    let texel_bits = slot.tile.texel_size().bits();
                    let image_stride_bits = (image_width * texel_bits as u32 + 7) & !7; // Round up to byte alignment to not miss any 4-bit values
                    let image_stride = image_stride_bits / 8;

                    // TODO helper in command
                    let left = data.upper_left_x().value();
                    let right = data.lower_right_x().value();
                    let top = data.upper_left_y().value();
                    let bottom = data.lower_right_y().value();

                    debug_assert!(left < right);
                    debug_assert!(top < bottom);

                    let tile_width = (right >> 2).wrapping_sub(left >> 2) + 1;
                    let tile_height = (bottom >> 2).wrapping_sub(top >> 2) + 1;

                    // TODO simplify and just copy stride?
                    let tile_bytes_per_row = ((tile_width * texel_bits as u16 + 7) & !7) / 8; // Round up to byte alignment to not miss any 4-bit values

                    let tile_stride = slot.tile.stride() as u32;

                    let mut ram_address = s.dp.state.texture.ram_address().value()
                        + ((top as u32 * image_width) + left as u32) * texel_bits as u32 / 8; // TODO rounding

                    // Copy each row
                    // TODO 4 bits formats: last 4bits copied when they should not sometimes?

                    let mut tmem_address = slot.tile.tmem_address_byte() as u32;

                    // log::error!(
                    //     "Loading tile: {}, {}, {}, {}, {}",
                    //     tile_width,
                    //     tile_height,
                    //     tile_stride,
                    //     texel_bits,
                    //     tile_bytes_per_row
                    // );

                    for _row in 0..tile_height {
                        s.ram.read_block(
                            RamLocation::from_absolute(ram_address),
                            tile_bytes_per_row as usize,
                            |ram_data| {
                                write_block(ram_data, &mut s.dp.tmem, tmem_address as usize);
                            },
                        );

                        ram_address += image_stride;
                        tmem_address += tile_stride;
                    }
                }

                Command::FillRectangle(data) => {
                    // TODO helper in command
                    let left = data.upper_left_x();
                    let top = data.upper_left_y();
                    let right = data.lower_right_x();
                    let bottom = data.lower_right_y();

                    s.video_renderer.push_command(video::Command::PushQuad {
                        vertices: [
                            coord(left, bottom),
                            coord(left, top),
                            coord(right, top),
                            coord(right, bottom),
                        ],
                        fill: QuadFill::Color {
                            color: s.dp.state.fill_color,
                        },
                    });
                }

                Command::TextureRectangle(data) => {
                    // Push the texture to the renderer

                    let slot = &s.dp.state.tile_slots[data.tile().value() as usize];

                    let left = slot.size.upper_left_x().value();
                    let right = slot.size.lower_right_x().value();
                    let top = slot.size.upper_left_y().value();
                    let bottom = slot.size.lower_right_y().value();

                    debug_assert!(left < right);
                    debug_assert!(top < bottom);

                    let tile_width = ((right >> 2).wrapping_sub(left >> 2) + 1) as usize;
                    let tile_height = ((bottom >> 2).wrapping_sub(top >> 2) + 1) as usize;
                    let tile_stride = slot.tile.stride();

                    let mut rgba: Vec<u8> = Vec::with_capacity(tile_width * tile_height * 4); // TODO allocate once? stack?

                    // We copy rows individually to account for the tile's stride which can be different from its width

                    let mut row_address = slot.tile.tmem_address_byte();

                    for _row in 0..tile_height {
                        match (slot.tile.format(), slot.tile.texel_size()) {
                            (ImageFormat::RGBA, TexelSize::B16) => {
                                // 2 bytes per texel: 5 bits red, 5 bits green, 5 bits blue, 1 bit alpha

                                let bytes_per_row = tile_width * 2;

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(
                                        tmem.chunks_exact(2)
                                            .flat_map(|texel| rgba5551_to_8888(texel[0], texel[1])),
                                    );
                                });
                            }

                            (ImageFormat::RGBA, TexelSize::B32) => {
                                // 4 bytes per texel: 8 bits red, 8 bits green, 8 bits blue, 8 bits alpha

                                let bytes_per_row = tile_width * 4;

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend_from_slice(tmem);
                                });
                            }

                            (ImageFormat::ColorIndexed, TexelSize::B4) => {
                                // 4 bits per texel: 4-bit color index into one of the 16-bit palettes

                                let bytes_per_row = tile_width.div_ceil(2);

                                let palette_offset =
                                    0x800 + (slot.tile.palette().value() as usize) * 16;

                                // TODO optim: convert palettes on LoadTLUT? also when writing tex in case games do crazy hacks?

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(
                                        tmem.iter()
                                            // Split each byte into two 4-bit texels
                                            .flat_map(|byte| [byte & 0xF0 >> 4, byte & 0x0F])
                                            // Convert each texel to RGBA
                                            .flat_map(|color_index| {
                                                let color_offset =
                                                    palette_offset + (color_index as usize) * 2;

                                                rgba5551_to_8888(
                                                    s.dp.tmem[color_offset],
                                                    s.dp.tmem[color_offset + 1],
                                                )
                                            }),
                                    );
                                });

                                // If the tile width is odd, we pushed an extraneous 4-bit entry last, so remove it

                                if tile_width & 1 != 0 {
                                    for _ in 0..4 {
                                        rgba.pop();
                                    }
                                }
                            }

                            (ImageFormat::ColorIndexed, TexelSize::B8) => {
                                // 1 byte per texel: 8-bit color index into the full 16-bit palette

                                let bytes_per_row = tile_width;

                                let palette_offset = 0x800;

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(tmem.iter().flat_map(|color_index| {
                                        let color_offset =
                                            palette_offset + (*color_index as usize) * 2;

                                        rgba5551_to_8888(
                                            s.dp.tmem[color_offset],
                                            s.dp.tmem[color_offset + 1],
                                        )
                                    }));
                                });
                            }

                            (ImageFormat::IntensityAlpha, TexelSize::B4) => {
                                // 4 bits per texel: 3 bits intensity, 1 bit alpha

                                let bytes_per_row = tile_width.div_ceil(2);

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(
                                        tmem.iter()
                                            // Split each byte into two 4-bit texels
                                            .flat_map(|byte| [byte & 0xF0 >> 4, byte & 0x0F])
                                            // Convert each texel to RGBA
                                            .flat_map(|texel| {
                                                let intensity = ((texel >> 1) & 7) * 255 / 7; // TODO optim?
                                                let alpha = (texel & 1) * 255; // TODO optim?

                                                [intensity, intensity, intensity, alpha]
                                            }),
                                    );
                                });

                                // If the tile width is odd, we pushed an extraneous 4-bit entry last, so remove it

                                if tile_width & 1 != 0 {
                                    for _ in 0..4 {
                                        rgba.pop();
                                    }
                                }
                            }

                            (ImageFormat::IntensityAlpha, TexelSize::B8) => {
                                // 1 byte per texel: 4 bits intensity, 4 bits alpha

                                let bytes_per_row = tile_width;

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(tmem.iter().flat_map(|texel| {
                                        let intensity = (*texel >> 4) * 255 / 15; // TODO optim?
                                        let alpha = (*texel & 0x0F) * 255 / 15; // TODO optim?

                                        [intensity, intensity, intensity, alpha]
                                    }));
                                });
                            }

                            (ImageFormat::IntensityAlpha, TexelSize::B16) => {
                                // 2 bytes per texel: 8-bit intensity, 8-bit alpha

                                let bytes_per_row = tile_width * 2;

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(tmem.chunks_exact(2).flat_map(|texel| {
                                        let intensity = texel[0];
                                        let alpha = texel[1];

                                        [intensity, intensity, intensity, alpha]
                                    }));
                                });
                            }

                            (ImageFormat::Intensity, TexelSize::B4)
                            | (ImageFormat::Intensity2, TexelSize::B4)
                            | (ImageFormat::Intensity3, TexelSize::B4)
                            | (ImageFormat::Intensity4, TexelSize::B4) => {
                                // 4 bits of intensity per texel

                                let bytes_per_row = tile_width.div_ceil(2);

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(
                                        tmem.iter()
                                            // Split each byte into two 4-bit texels
                                            .flat_map(|byte| [byte & 0xF0 >> 4, byte & 0x0F])
                                            // Convert each texel to RGBA
                                            .flat_map(|texel| {
                                                let intensity = (texel << 4) | texel;

                                                [intensity, intensity, intensity, intensity]
                                            }),
                                    );
                                });

                                // If the tile width is odd, we pushed an extraneous 4-bit entry last, so remove it

                                if tile_width & 1 != 0 {
                                    for _ in 0..4 {
                                        rgba.pop();
                                    }
                                }
                            }

                            (ImageFormat::Intensity, TexelSize::B8)
                            | (ImageFormat::Intensity2, TexelSize::B8)
                            | (ImageFormat::Intensity3, TexelSize::B8)
                            | (ImageFormat::Intensity4, TexelSize::B8) => {
                                // 1 byte per texel: 8-bit intensity

                                let bytes_per_row = tile_width;

                                read_block(&s.dp.tmem, row_address, bytes_per_row, |tmem| {
                                    rgba.extend(tmem.iter().flat_map(|intensity| {
                                        [*intensity, *intensity, *intensity, *intensity]
                                    }));
                                });
                            }

                            _ => panic!(
                                "Unsupported {:?} / {:?} format",
                                slot.tile.format(),
                                slot.tile.texel_size()
                            ),
                        }

                        row_address += tile_stride;
                    }

                    debug_assert_eq!(rgba.len(), tile_width * tile_height * 4);

                    s.video_renderer.push_command(video::Command::PushTile {
                        slot: slot.tile.tile().value(),
                        rgba,
                        width: tile_width as u32,
                        height: tile_height as u32,
                    });

                    // Push the geometry to the renderer

                    let tile_index = slot.tile.tile().value();

                    let rect_left = data.top_left_x();
                    let rect_top = data.top_left_y();
                    let rect_right = data.bottom_right_x();
                    let rect_bottom = data.bottom_right_y();

                    assert!(rect_left < rect_right);
                    assert!(rect_top < rect_bottom);

                    let tile_s_start = data.top_left_s() as f32 / 32.0 / tile_width as f32;
                    let tile_t_start = data.top_left_t() as f32 / 32.0 / tile_height as f32;

                    let tile_dsdx = data.dsdx() as i16 as f32 / 1024.0;
                    let tile_dtdy = data.dtdy() as i16 as f32 / 1024.0;

                    let tile_s_end = tile_s_start + tile_dsdx;
                    let tile_t_end = tile_t_start + tile_dtdy;

                    let uvs = [
                        [tile_s_start, tile_t_start],
                        [tile_s_end, tile_t_start],
                        [tile_s_end, tile_t_end],
                        [tile_s_start, tile_t_end],
                    ];

                    // TODO flip?

                    if data.flip() {
                        panic!("Rectangle flip");
                    }

                    s.video_renderer.push_command(video::Command::PushQuad {
                        vertices: [
                            coord(rect_left, rect_top),
                            coord(rect_right, rect_top),
                            coord(rect_right, rect_bottom),
                            coord(rect_left, rect_bottom),
                        ],
                        fill: QuadFill::Texture {
                            tile_slot: tile_index,
                            uvs,
                        },
                    });
                }

                _ => {}
            }
        }

        // Render a new frame
        // (we should be here because we got a SYNC FULL command)

        if s.dp.decoded_commands.len() > 0 {
            s.video_renderer.push_command(video::Command::Render);
        }

        // Clear the command queue

        s.dp.decoded_commands.clear();
    }
}

// TODO to struct fund
fn convert_color(color: RGBA) -> [f32; 4] {
    let r = (color.red().value() as f32) / 255.0;
    let g = (color.green().value() as f32) / 255.0;
    let b = (color.blue().value() as f32) / 255.0;

    // TODO alpha

    [r, g, b, 1.0]
}

fn coord(x: u12, y: u12) -> [f32; 2] {
    // 10.2 fixed point to float

    let x = (x.value() >> 2) as f32;
    let y = (y.value() >> 2) as f32; // TODO div to keep frac

    // To NDC
    // TODO just handle that in renderer?

    let x = (x / 320.0) * 2.0 - 1.0;
    let y = -((y / 240.0) * 2.0 - 1.0); // flip Y

    [x, y]
}

#[derive(Clone, Copy, Debug)]
enum Command {
    FillTriangle,
    TextureRectangle(TextureRectangle),
    TextureRectangleFlip,
    SyncLoad,
    SyncPipe,
    SyncTile,
    SyncFull,
    SetKeyGB,
    SetKeyR,
    SetConvert,
    SetScissor,
    SetPrimitiveDepth,
    SetOtherModes,
    LoadTLUT(LoadTile),
    SetTileSize(SetTileSize),
    LoadBlock,
    LoadTile(LoadTile),
    SetTile(SetTile),
    FillRectangle(FillRectangle),
    SetFillColor(SetFillColor),
    SetFogColor,
    SetBlendColor,
    SetPrimitiveColor(SetPrimitiveColor),
    SetEnvironmentColor,
    SetCombineMode,
    SetTextureImage(SetTextureImage),
    SetDepthImage,
    SetColorImage,
}

#[bitenum(u3, exhaustive = true)]
#[derive(Debug)]
enum ImageFormat {
    RGBA = 0,
    YUV = 1,
    ColorIndexed = 2,
    IntensityAlpha = 3,
    Intensity = 4,

    // 4+ values also mean Intensity
    Intensity2 = 5,
    Intensity3 = 6,
    Intensity4 = 7,
}

#[bitenum(u2, exhaustive = true)]
#[derive(Debug)]
enum TexelSize {
    B4 = 0,
    B8 = 1,
    B16 = 2,
    B32 = 3,
}

impl TexelSize {
    pub fn bits(&self) -> usize {
        match self {
            TexelSize::B4 => 4,
            TexelSize::B8 => 8,
            TexelSize::B16 => 16,
            TexelSize::B32 => 32,
        }
    }
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct LoadTLUT {
    #[bits(44..=55, r)]
    low_index: u12,

    #[bits(24..=26, r)]
    tile_index: u3,

    #[bits(12..=23, r)]
    high_index: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetTextureImage {
    #[bits(53..=55, r)]
    format: ImageFormat,

    #[bits(51..=52, r)]
    texel_size: TexelSize,

    /// Width in pixels of the texture in RAM, minus one.
    #[bits(32..=41, r)]
    width: u10,

    #[bits(0..=25, r)]
    ram_address: u26,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct FillRectangle {
    #[bits(44..=55, r)]
    lower_right_x: u12,

    #[bits(32..=43, r)]
    lower_right_y: u12,

    #[bits(24..=26, r)]
    tile: u3,

    #[bits(12..=23, r)]
    upper_left_x: u12,

    #[bits(0..=11, r)]
    upper_left_y: u12,
}

#[bitfield(u128, forbid_overlaps, instrospect, default = 0, debug)]
pub struct TextureRectangle {
    /// Flip the texture vertically. TODO vertically?
    ///
    /// TextureRectangle = 0x24
    /// TextureRectangleFlip = 0x25
    ///
    /// So the flip attribute is the LSB of the command.
    #[bit(120, r)]
    flip: bool,

    // NOTE: the official RDP manual seems to be wrong, the bottom coordinates come first
    #[bits(108..=119, r)]
    bottom_right_x: u12,

    #[bits(96..=107, r)]
    bottom_right_y: u12,

    #[bits(88..=90, r)]
    tile: u3,

    #[bits(76..=87, r)]
    top_left_x: u12,

    #[bits(64..=75, r)]
    top_left_y: u12,

    #[bits(48..=63, r)]
    top_left_s: u16,

    #[bits(32..=47, r)]
    top_left_t: u16,

    #[bits(16..=31, r)]
    dsdx: u16,

    #[bits(0..=15, r)]
    dtdy: u16,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct LoadTile {
    #[bits(44..=55, r)]
    upper_left_x: u12,

    #[bits(32..=43, r)]
    upper_left_y: u12,

    #[bits(24..=26, r)]
    tile: u3,

    #[bits(12..=23, r)]
    lower_right_x: u12,

    #[bits(0..=11, r)]
    lower_right_y: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetTile {
    #[bits(53..=55, r)]
    format: ImageFormat,

    #[bits(51..=52, r)]
    texel_size: TexelSize,

    /// Tile stride in 64-bit words
    #[bits(41..=49, r)]
    line_size: u9,

    #[bits(32..=40, r)]
    tmem_address: u9,

    #[bits(24..=26, r)]
    tile: u3,

    #[bits(20..=23, r)]
    palette: u4,

    #[bit(19, r)]
    clamp_y: bool,

    #[bit(18, r)]
    mirror_y: bool,

    #[bits(14..=17, r)]
    mask_y: u4,

    #[bits(10..=13, r)]
    shift_y: u4,

    #[bit(9, r)]
    clamp_x: bool,

    #[bit(8, r)]
    mirror_x: bool,

    #[bits(4..=7, r)]
    mask_x: u4,

    #[bits(0..=3, r)]
    shift_x: u4,
}

impl SetTile {
    /// Returns the tile width in bytes, as it's defined in 64-bit words in the command.
    pub fn stride(&self) -> usize {
        (self.line_size().value() as usize) << 3
    }
}

impl SetTile {
    /// Returns the TMEM address in bytes, as it's defined in 64-bit words in the command.
    pub fn tmem_address_byte(&self) -> usize {
        (self.tmem_address().value() as usize) << 3
    }
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetTileSize {
    #[bits(44..=55, r)]
    upper_left_x: u12,

    #[bits(32..=43, r)]
    upper_left_y: u12,

    #[bits(24..=26, r)]
    tile: u3,

    #[bits(12..=23, r)]
    lower_right_x: u12,

    #[bits(0..=11, r)]
    lower_right_y: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetPrimitiveColor {
    #[bits(40..=44, r)]
    min_level: u5,

    #[bits(32..=39, r)]
    level_fraction: u8,

    #[bits(24..=31, r)]
    red: u8,

    #[bits(16..=23, r)]
    green: u8,

    #[bits(8..=15, r)]
    blue: u8,

    #[bits(0..=7, r)]
    alpha: u8,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetFillColor {
    #[bits(0..=31, r)]
    color: RGBA,
    // TODO other formats overlapped?
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct RGBA {
    #[bits(24..=31, r)]
    red: u8,

    #[bits(16..=23, r)]
    green: u8,

    #[bits(8..=15, r)]
    blue: u8,

    #[bits(0..=7, r)]
    alpha: u8,
}

// TODO used elsewhere, make common
fn b5_to_b8(value: u8) -> u8 {
    (((value as u16 & 0x1F) * 255) / 31) as u8 // TODO correct? optim?
}

pub fn rgba5551_to_8888(hi: u8, lo: u8) -> [u8; 4] {
    [
        b5_to_b8(hi >> 3),
        b5_to_b8(((hi & 7) << 2) | (lo >> 6)),
        b5_to_b8((lo >> 1) & 0x1F),
        0xFF, // TODO
    ]
}
