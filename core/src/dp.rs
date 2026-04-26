//! Reality Display Processor
//!
//! Resources:
//! - Nintendo Ultra64 RDP Command Summary https://ultra64.ca/files/documentation/silicon-graphics/SGI_RDP_Command_Summary.pdf
//! - N64brew / Reality Display Processor https://n64brew.dev/wiki/Reality_Display_Processor
//! - Notes on the Angrylion implementation https://emudev.org/2021/09/21/Angrylion_RDP_Comments

use std::collections::VecDeque;

use arbitrary_int::prelude::*;
use bitbybit::bitfield;

use crate::{
    blocks::write_block,
    location::Location,
    mi::Interrupt,
    ram::RamLocation,
    rendering::{
        tile_cache::{ImageFormat, TexelSize, TileCache},
        video::{self, QuadFill},
    },
    sp::SpMemLocation,
    system::System,
    value::Value,
};

const REG_START: u32 = 0x0410_0000;
const REG_END: u32 = 0x0420_0000;

pub type DpLocation = Location<REG_START, REG_END>;

const START_REG: u32 = 0;
const END_REG: u32 = 1;
const CURRENT_REG: u32 = 2;
const STATUS_REG: u32 = 3;
const CLOCK_REG: u32 = 4;
const BUF_BUSY_REG: u32 = 5;
const PIPE_BUSY_REG: u32 = 6;
const TMEM_BUSY_REG: u32 = 7;

const STATUS_XBUS: u32 = 1;
const STATUS_FREEZE: u32 = 1 << 1;
const STATUS_FLUSH: u32 = 1 << 2;
const STATUS_GCLK: u32 = 1 << 3;
const STATUS_TMEM_BUSY: u32 = 1 << 4;
const STATUS_PIPE_BUSY: u32 = 1 << 5;
const STATUS_COMMAND_BUSY: u32 = 1 << 6;
const STATUS_COMMAND_BUFFER_READY: u32 = 1 << 7;
const STATUS_DMA_BUSY: u32 = 1 << 8;
const STATUS_END_PENDING: u32 = 1 << 9;
const STATUS_START_PENDING: u32 = 1 << 10;

const STATUS_XBUS_CLEAR: u32 = 1;
const STATUS_XBUS_SET: u32 = 1 << 1;
const STATUS_FREEZE_CLEAR: u32 = 1 << 2;
const STATUS_FREEZE_SET: u32 = 1 << 3;
const STATUS_FLUSH_CLEAR: u32 = 1 << 4;
const STATUS_FLUSH_SET: u32 = 1 << 5;
const STATUS_TMEM_BUSY_CLEAR: u32 = 1 << 6;
const STATUS_PIPE_BUSY_CLEAR: u32 = 1 << 7;
const STATUS_BUF_BUSY_CLEAR: u32 = 1 << 8;
const STATUS_CLK_CLEAR: u32 = 1 << 9;

// For now, we keep the "command buffer ready" bit set as we process commands instantly
const STATUS_DEFAULT: u32 = STATUS_COMMAND_BUFFER_READY;

// TODO double buffering

pub struct Dp {
    // TODO struct regs
    pub regs: [u32; 8],

    /// Texture memory.
    pub tmem: [u8; 0x1000], // TODO vis

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

    /// Cache that stores decoded RGBA tiles.
    tile_cache: TileCache,

    /// Whether a DMA is pending due to a frozen state.
    /// Hardware tests show that END_PENDING is not set if frozen, so we cannot use it to trigger the DMA when unfrozen.
    frozen_dma: bool,
}

#[derive(Default)]
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
            regs: [0, 0, 0, STATUS_DEFAULT, 0, 0, 0, 0],

            tmem: [0; 0x1000],

            command_buffer: VecDeque::new(),
            decoded_commands: Vec::new(),

            state: State::default(),

            tile_cache: TileCache::default(),

            frozen_dma: false,
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
                // Write the START address and make it "pending", ie. we're waiting for END to be written.
                // If START is already pending, ignore the new address (confirmed by n64-systemtest).

                // TODO unclear if this is correct, some docs mention some form of double-buffering?!

                if s.dp.regs[STATUS_REG as usize] & STATUS_START_PENDING == 0 {
                    data.write_reg(&mut s.dp.regs, addr.relative() & 0x1F);
                    s.dp.regs[START_REG as usize] &= 0x00FF_FFF8;

                    s.dp.regs[STATUS_REG as usize] |= STATUS_START_PENDING;
                    s.dp.regs[STATUS_REG as usize] &= !STATUS_END_PENDING;
                } else {
                    log::warn!(
                        "DP: START already pending, ignoring new address {:08X}",
                        data
                    );
                }
            }
            END_REG => {
                data.write_reg(&mut s.dp.regs, addr.relative() & 0x1F);
                s.dp.regs[END_REG as usize] &= 0x00FF_FFF8;

                // If START was "pending", set CURRENT to START and clear the pending flag, this is a new transfer.
                // If END is written again before START, the DMA will continue from CURRENT, this is an increment transfer.

                if s.dp.regs[STATUS_REG as usize] & STATUS_START_PENDING != 0 {
                    s.dp.regs[CURRENT_REG as usize] = s.dp.regs[START_REG as usize];
                }

                s.dp.regs[STATUS_REG as usize] &= !STATUS_START_PENDING;

                // Start the transfer if not frozen, otherwise it will start later when unfrozen
                //
                // Hardware tests show that END_PENDING is not set if frozen

                if s.dp.regs[STATUS_REG as usize] & STATUS_FREEZE == 0 {
                    s.dp.regs[STATUS_REG as usize] |= STATUS_END_PENDING;

                    Self::start_dma(s);
                } else {
                    s.dp.frozen_dma = true;
                }
            }
            STATUS_REG => {
                let mut trigger_bits = [0u32];
                data.write_reg(&mut trigger_bits, addr.relative() & 3);

                // TODO what if both clear/set bits are set? similar to SP (does nothing)?

                // XBUS

                if trigger_bits[0] & STATUS_XBUS_CLEAR != 0 {
                    s.dp.regs[STATUS_REG as usize] &= !STATUS_XBUS;
                }

                if trigger_bits[0] & STATUS_XBUS_SET != 0 {
                    s.dp.regs[STATUS_REG as usize] |= STATUS_XBUS;
                }

                // FREEZE

                if trigger_bits[0] & STATUS_FREEZE_CLEAR != 0 {
                    log::warn!("DP: UNFROZEN",);

                    s.dp.regs[STATUS_REG as usize] &= !STATUS_FREEZE;

                    // Start any pending DMA

                    if s.dp.frozen_dma {
                        s.dp.frozen_dma = false;

                        s.dp.regs[STATUS_REG as usize] |= STATUS_END_PENDING;

                        Self::start_dma(s);
                    }
                }

                if trigger_bits[0] & STATUS_FREEZE_SET != 0 {
                    log::warn!("DP: FREEZE",);

                    s.dp.regs[STATUS_REG as usize] |= STATUS_FREEZE;
                }

                // FLUSH

                if trigger_bits[0] & STATUS_FLUSH_CLEAR != 0 {
                    s.dp.regs[STATUS_REG as usize] &= !STATUS_FLUSH;
                }

                if trigger_bits[0] & STATUS_FLUSH_SET != 0 {
                    s.dp.regs[STATUS_REG as usize] |= STATUS_FLUSH;
                    log::warn!("DP FLUSH");
                    // TODO do something?
                }

                // TMEM_BUSY

                if trigger_bits[0] & STATUS_TMEM_BUSY_CLEAR != 0 {
                    s.dp.regs[TMEM_BUSY_REG as usize] = 0;
                }

                // PIPE_BUSY

                if trigger_bits[0] & STATUS_PIPE_BUSY_CLEAR != 0 {
                    s.dp.regs[PIPE_BUSY_REG as usize] = 0;
                }

                // BUF_BUSY

                if trigger_bits[0] & STATUS_BUF_BUSY_CLEAR != 0 {
                    s.dp.regs[BUF_BUSY_REG as usize] = 0;
                }

                // CLK

                if trigger_bits[0] & STATUS_CLK_CLEAR != 0 {
                    s.dp.regs[CLOCK_REG as usize] = 0;
                }
            }
            _ => {
                // Other registers are read-only
            }
        }
    }

    fn start_dma(s: &mut System) {
        let from_sp = s.dp.regs[STATUS_REG as usize] & STATUS_XBUS != 0;

        let current = s.dp.regs[CURRENT_REG as usize];
        let end = s.dp.regs[END_REG as usize];

        debug_assert!(current <= end, "DP DMA current > end");

        // log::debug!(
        //     "DP: DMA (XBus={}): {:08X} -> {:08X} -> {:08X}",
        //     from_sp,
        //     s.dp.regs[START_REG as usize],
        //     current,
        //     end
        // );

        // Set the busy bits
        // TODO unclear when they get cleared and if this is even accurate

        s.dp.regs[STATUS_REG as usize] |= STATUS_DMA_BUSY | STATUS_GCLK | STATUS_PIPE_BUSY;

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

        s.dp.regs[STATUS_REG as usize] &= !(STATUS_END_PENDING | STATUS_DMA_BUSY);
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
            // TODO fullsync: END_PENDING 0

            let mut loggg = String::new();

            match first_byte & 0x3F {
                0..=7 | 0x10..=0x23 | 0x31 => {
                    loggg.push_str("DP: NOP");

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
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SyncLoad);
                    });
                }
                0x27 => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SyncPipe);
                    });
                }
                0x28 => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SyncTile);
                    });
                }
                0x29 => {
                    // Sync full

                    if_ready!(8, {
                        Self::apply_command(s);

                        s.mi.set_pending_interrupt(Interrupt::Dp, &mut s.cop0); // TODO temp
                    });
                }
                0x2A => {
                    loggg.push_str("DP: Set key GB");

                    if_ready!(8, {});
                }
                0x2B => {
                    loggg.push_str("DP: Set key R");

                    if_ready!(8, {});
                }
                0x2C => {
                    loggg.push_str("DP: Set convert");

                    if_ready!(8, {});
                }
                0x2D => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetScissor(
                            SetScissor::new_with_raw_value(u64::from_be_bytes([
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
                0x2E => {
                    loggg.push_str("DP: Set prim depth");

                    if_ready!(8, {});
                }
                0x2F => {
                    loggg.push_str("DP: Set other mode");

                    if_ready!(8, {});
                }
                0x30 => {
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
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::LoadBlock(
                            LoadBlock::new_with_raw_value(u64::from_be_bytes([
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
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetFogColor(
                            SetFogColor::new_with_raw_value(u64::from_be_bytes([
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
                0x39 => {
                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetBlendColor(
                            SetBlendColor::new_with_raw_value(u64::from_be_bytes([
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
                    loggg.push_str("DP: set env color");

                    if_ready!(8, {
                        s.dp.decoded_commands.push(Command::SetEnvironmentColor(
                            SetEnvironmentColor::new_with_raw_value(u64::from_be_bytes([
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
                0x3C => {
                    loggg.push_str("DP: set combine");

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
                    loggg.push_str("DP: set zimg");

                    if_ready!(8, {});
                }
                0x3F => {
                    loggg.push_str("DP: set cimg");

                    if_ready!(8, {});
                }
                x => panic!("Unknown DP DMA command: {:X}", x),
            }

            // if !loggg.is_empty() {
            //     log::debug!("LOGGG");
            //     log::debug!("{}", loggg);
            // }
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
                Command::SetScissor(_data) => {
                    //log::warn!("DP: set scissor: {:?}", data);
                }

                Command::SetPrimitiveColor(_data) => {
                    //log::warn!("DP: set primitive color: {:?}", data);
                }

                Command::SetEnvironmentColor(_data) => {
                    //log::warn!("DP: set environment color: {:?}", data);
                }

                Command::SetFogColor(_data) => {
                    //log::warn!("DP: set fog color: {:?}", data);
                }

                Command::SetBlendColor(_data) => {
                    //log::warn!("DP: set blend color: {:?}", data);
                }

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

                Command::LoadBlock(data) => {
                    // Load a block from RAM to TMEM

                    let slot = &s.dp.state.tile_slots[data.tile().value() as usize];

                    // TODO probably all wrong!

                    // TODO rename?
                    let left = data.upper_left_x().value();
                    let right = data.lower_right_x().value();
                    let top = data.upper_left_y().value();
                    let dxt = data.dxt().value();

                    //log::warn!("RSP: load block: {}, {}, {}, {}", left, right, top, dxt);

                    // assert_eq!(left, 0);
                    // assert_eq!(top, 0);

                    let texel_count = (right + 1) as usize;
                    let texel_bits = slot.tile.texel_size().bits();
                    let byte_count = (texel_count * texel_bits + 7) & !7; // Round up to byte alignment to not miss any 4-bit values

                    // TODO offset by coords?
                    let ram_address = s.dp.state.texture.ram_address().value();

                    // TODO dxt stuff?

                    let mut tmem_address = slot.tile.tmem_address_byte();

                    s.ram.read_block(
                        RamLocation::from_absolute(ram_address),
                        byte_count,
                        |ram_data| {
                            write_block(ram_data, &mut s.dp.tmem, tmem_address);

                            tmem_address += byte_count;
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

                    let tile_stride = slot.tile.stride_byte() as u32;

                    let mut ram_address = s.dp.state.texture.ram_address().value()
                        + ((top as u32 * image_width) + left as u32) * texel_bits as u32 / 8; // TODO rounding

                    // Copy each row
                    // TODO 4 bits formats: last 4bits copied when they should not sometimes?

                    let mut tmem_address = slot.tile.tmem_address_byte() as u32;

                    //let start_address = tmem_address;
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

                    //log::error!("Loaded tile: {}", tmem_address - start_address);
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
                    // The lower coordinates should be greater than the upper coordinates.
                    // If not, don't render anything.
                    // TODO souce???

                    let rect_left = data.top_left_x();
                    let rect_top = data.top_left_y();
                    let rect_right = data.bottom_right_x();
                    let rect_bottom = data.bottom_right_y();

                    if rect_right <= rect_left || rect_bottom <= rect_top {
                        continue;
                    }

                    if data.flip() {
                        //log::warn!("Rectangle flip");
                    }

                    // Push the texture to the renderer
                    // TODO only if never pushed before?

                    let slot = &s.dp.state.tile_slots[data.tile().value() as usize];

                    let (tile, tile_id) = s.dp.tile_cache.get(&s.dp.tmem, slot.tile, slot.size);

                    // log::error!(
                    //     "Using tile: {} = {}, {}, {}, {}, {}, {}, {} @ {}x{}",
                    //     tile_id,
                    //     tile.width,
                    //     tile.height,
                    //     slot.tile.stride_byte(),
                    //     slot.tile.texel_size().bits(),
                    //     slot.tile.line_size().value(),
                    //     tile.width,
                    //     tile.height,
                    //     rect_left,
                    //     rect_top,
                    // );

                    s.video_renderer.push_command(video::Command::PushTile {
                        tile_id,
                        tile: tile.clone(),
                    });

                    // Push the geometry to the renderer

                    let vertices = [
                        coord(rect_left, rect_top),
                        coord(rect_right, rect_top),
                        coord(rect_right, rect_bottom),
                        coord(rect_left, rect_bottom),
                    ];

                    let s_start_texel = data.top_left_s() as i16 as f32 / 32.0;
                    let t_start_texel = data.top_left_t() as i16 as f32 / 32.0;

                    let dsdx = data.dsdx() as i16 as f32 / 1024.0;
                    let dtdy = data.dtdy() as i16 as f32 / 1024.0;

                    let rect_left = rect_left.value() as f32 / 4.0;
                    let rect_right = rect_right.value() as f32 / 4.0;
                    let rect_top = rect_top.value() as f32 / 4.0;
                    let rect_bottom = rect_bottom.value() as f32 / 4.0;

                    let rect_width = (rect_right - rect_left).abs();
                    let rect_height = (rect_bottom - rect_top).abs();

                    let s_end_texel = s_start_texel + dsdx * rect_width;
                    let t_end_texel = t_start_texel + dtdy * rect_height;

                    let s_start = s_start_texel / tile.width as f32;
                    let t_start = t_start_texel / tile.height as f32;
                    let s_end = s_end_texel / tile.width as f32;
                    let t_end = t_end_texel / tile.height as f32;

                    let uvs = [
                        [s_start, t_start],
                        [s_end, t_start],
                        [s_end, t_end],
                        [s_start, t_end],
                    ];

                    //let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

                    // if uvs[0][0] != 0.0 && uvs[0][0] != 1.0
                    //     || uvs[0][1] != 0.0 && uvs[0][1] != 1.0
                    //     || uvs[1][0] != 0.0 && uvs[1][0] != 1.0
                    //     || uvs[1][1] != 0.0 && uvs[1][1] != 1.0
                    //     || uvs[2][0] != 0.0 && uvs[2][0] != 1.0
                    //     || uvs[2][1] != 0.0 && uvs[2][1] != 1.0
                    //     || uvs[3][0] != 0.0 && uvs[3][0] != 1.0
                    //     || uvs[3][1] != 0.0 && uvs[3][1] != 1.0
                    // {
                    //     log::warn!("Invalid UVs: {:#?}", uvs);
                    //     log::warn!("Rect: {:#?}", data);
                    //     log::warn!("Tile: {:#?}", slot.tile);
                    // }
                    //let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

                    // log::warn!(
                    //     "#{} - {:?} , uv ({:?}, {:?}) ---rect dim {:?} x {:?}, s_start_texel {:?} / {:?} / {:?} / {:?} -> {:?}, dsdx {:?} tile w {} ",
                    //     tile_index,
                    //     vertices,
                    //     uvs[0],
                    //     uvs[2],
                    //     //
                    //     rect_width,
                    //     rect_height,
                    //     data.top_left_s(),
                    //     data.top_left_s() as i16,
                    //     data.top_left_s() as i16 as f32,
                    //     data.top_left_s() as i16 as f32 / 32.0,
                    //     s_start,
                    //     data.dsdx(),
                    //     tile_width
                    // );

                    s.video_renderer.push_command(video::Command::PushQuad {
                        vertices,
                        fill: QuadFill::Texture { tile_id, uvs },
                    });
                }

                _ => {}
            }
        }

        // Render a new frame
        // (we're here because we got a SYNC FULL command)
        // TODO handle SyncFull like the other commands, in the match

        if !s.dp.decoded_commands.is_empty() {
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
    SetScissor(SetScissor),
    SetPrimitiveDepth,
    SetOtherModes,
    LoadTLUT(LoadTile),
    SetTileSize(SetTileSize),
    LoadBlock(LoadBlock),
    LoadTile(LoadTile),
    SetTile(SetTile),
    FillRectangle(FillRectangle),
    SetFillColor(SetFillColor),
    SetFogColor(SetFogColor),
    SetBlendColor(SetBlendColor),
    SetPrimitiveColor(SetPrimitiveColor),
    SetEnvironmentColor(SetEnvironmentColor),
    SetCombineMode,
    SetTextureImage(SetTextureImage),
    SetDepthImage,
    SetColorImage,
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct Color {
    #[bits(24..=31, rw)]
    red: u8,

    #[bits(16..=23, rw)]
    green: u8,

    #[bits(8..=15, rw)]
    blue: u8,

    #[bits(0..=7, rw)]
    alpha: u8,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetEnvironmentColor {
    #[bits(0..=31, rw)]
    color: Color,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetFogColor {
    #[bits(0..=31, rw)]
    color: Color,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetBlendColor {
    #[bits(0..=31, rw)]
    color: Color,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetScissor {
    #[bits(44..=55, rw)]
    upper_left_x: u12,

    #[bits(32..=43, rw)]
    upper_left_y: u12,

    #[bit(25, rw)]
    field: bool,

    #[bit(24, rw)]
    odd: bool,

    #[bits(12..=23, rw)]
    lower_right_x: u12,

    #[bits(0..=11, rw)]
    lower_right_y: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct LoadTLUT {
    #[bits(44..=55, rw)]
    low_index: u12,

    #[bits(24..=26, rw)]
    tile_index: u3,

    #[bits(12..=23, rw)]
    high_index: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetTextureImage {
    #[bits(53..=55, rw)]
    format: ImageFormat,

    #[bits(51..=52, rw)]
    texel_size: TexelSize,

    /// Width in pixels of the texture in RAM, minus one.
    #[bits(32..=41, rw)]
    width: u10,

    #[bits(0..=25, rw)]
    ram_address: u26,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct FillRectangle {
    #[bits(44..=55, rw)]
    lower_right_x: u12,

    #[bits(32..=43, rw)]
    lower_right_y: u12,

    #[bits(24..=26, rw)]
    tile: u3,

    #[bits(12..=23, rw)]
    upper_left_x: u12,

    #[bits(0..=11, rw)]
    upper_left_y: u12,
}

// todo signed/unsigned 10.2, 10.5, 5.10 structs?

#[bitfield(u128, forbid_overlaps, instrospect, default = 0, debug)]
pub struct TextureRectangle {
    /// Flip the texture vertically. TODO vertically?
    ///
    /// Commands:
    /// - TextureRectangle = 0x24
    /// - TextureRectangleFlip = 0x25
    ///
    /// So the flip attribute is the LSB of the command.
    #[bit(120, rw)]
    flip: bool,

    /// Bottom right x screen coordinate, 10.2 format
    ///
    /// NOTE: the official RDP manual seems to be wrong, the bottom coordinates come first!
    #[bits(108..=119, rw)]
    bottom_right_x: u12,

    /// Bottom right y screen coordinate, 10.2 format
    #[bits(96..=107, rw)]
    bottom_right_y: u12,

    /// Tile index
    #[bits(88..=90, rw)]
    tile: u3,

    /// Top left x screen coordinate, 10.2 format
    #[bits(76..=87, rw)]
    top_left_x: u12,

    /// Top left y screen coordinate, 10.2 format
    #[bits(64..=75, rw)]
    top_left_y: u12,

    /// Top left s, 10.5 format
    #[bits(48..=63, rw)]
    top_left_s: u16,

    /// Top left t, 10.5 format
    #[bits(32..=47, rw)]
    top_left_t: u16,

    /// Change in s per x, 5.10 format
    #[bits(16..=31, rw)]
    dsdx: u16,

    /// Change in t per y, 5.10 format
    #[bits(0..=15, rw)]
    dtdy: u16,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct LoadBlock {
    #[bits(44..=55, rw)]
    upper_left_x: u12,

    #[bits(32..=43, rw)]
    upper_left_y: u12,

    #[bits(24..=26, rw)]
    tile: u3,

    #[bits(12..=23, rw)]
    lower_right_x: u12,

    /// TODO ?
    #[bits(0..=11, rw)]
    dxt: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct LoadTile {
    #[bits(44..=55, rw)]
    upper_left_x: u12,

    #[bits(32..=43, rw)]
    upper_left_y: u12,

    #[bits(24..=26, rw)]
    tile: u3,

    #[bits(12..=23, rw)]
    lower_right_x: u12,

    #[bits(0..=11, rw)]
    lower_right_y: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetTile {
    #[bits(53..=55, rw)]
    format: ImageFormat,

    #[bits(51..=52, rw)]
    texel_size: TexelSize,

    /// Tile stride in 64-bit words
    #[bits(41..=49, rw)]
    line_size: u9,

    #[bits(32..=40, rw)]
    tmem_address: u9,

    #[bits(24..=26, rw)]
    tile: u3,

    #[bits(20..=23, rw)]
    palette: u4,

    #[bit(19, rw)]
    clamp_y: bool,

    #[bit(18, rw)]
    mirror_y: bool,

    #[bits(14..=17, rw)]
    mask_y: u4,

    #[bits(10..=13, rw)]
    shift_y: u4,

    #[bit(9, rw)]
    clamp_x: bool,

    #[bit(8, rw)]
    mirror_x: bool,

    #[bits(4..=7, rw)]
    mask_x: u4,

    #[bits(0..=3, rw)]
    shift_x: u4,
}

impl SetTile {
    /// Returns the tile width in bytes, as it's defined in 64-bit words in the command.
    pub fn stride_byte(&self) -> usize {
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
    #[bits(44..=55, rw)]
    upper_left_x: u12,

    #[bits(32..=43, rw)]
    upper_left_y: u12,

    #[bits(24..=26, rw)]
    tile: u3,

    #[bits(12..=23, rw)]
    lower_right_x: u12,

    #[bits(0..=11, rw)]
    lower_right_y: u12,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetPrimitiveColor {
    #[bits(40..=44, rw)]
    min_level: u5,

    #[bits(32..=39, rw)]
    level_fraction: u8,

    #[bits(0..=31, rw)]
    color: Color,
}

#[bitfield(u64, forbid_overlaps, instrospect, default = 0, debug)]
pub struct SetFillColor {
    #[bits(0..=31, rw)]
    color: RGBA,
    // TODO other formats overlapped?
}

#[bitfield(u32, forbid_overlaps, instrospect, default = 0, debug)]
pub struct RGBA {
    #[bits(24..=31, rw)]
    red: u8,

    #[bits(16..=23, rw)]
    green: u8,

    #[bits(8..=15, rw)]
    blue: u8,

    #[bits(0..=7, rw)]
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
        (lo & 1) * 255,
    ]
}
