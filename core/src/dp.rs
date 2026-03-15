use crate::{
    location::Location,
    mi::Interrupt,
    ram::{Ram, RamLocation},
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

#[derive(Default, Clone)]
pub struct Dp {
    pub regs: [u32; 8],
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
                    log::error!("DP FREEZE");
                }

                // FLUSH

                if trigger_bits[0] & STATUS_FLUSH_CLEAR != 0 {
                    status &= !STATUS_FLUSH;
                }
                if trigger_bits[0] & STATUS_FLUSH_SET != 0 {
                    status |= STATUS_FLUSH;
                    log::error!("DP FLUSH");
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
        //     "DP DMA (XBus={}): {:08X} -> {:08X} -> {:08X}",
        //     from_sp,
        //     s.dp.regs[START_REG as usize],
        //     current,
        //     end
        // );

        let data: Vec<u8> = if from_sp {
            (current..end)
                .map(|i| s.sp.read_mem(SpMemLocation::from_relative(i)))
                .collect()
        } else {
            (current..end)
                .map(|i| Ram::read(s, RamLocation::from_relative(i)))
                .collect()
        };

        let mut index = 0;

        loop {
            if index >= data.len() {
                break;
            }

            let b0 = data[index];
            let b1 = data[index + 1];
            let b2 = data[index + 2];
            let b3 = data[index + 3];

            // let data = u32::from_be_bytes([
            //     data[current],
            //     data[current + 1],
            //     data[current + 2],
            //     data[current + 3],
            // ]);

            // TODO fullsync: END_PENDING 0, DP int

            let mut loggg = String::new();

            match b0 & 0x3F {
                0..=7 | 0x10..=0x23 | 0x31 => {
                    //log::debug!("DP: NOP");
                    index += 8;
                }
                0x08..=0x0F => {
                    let shade = b0 & 4 != 0;
                    let texture = b0 & 2 != 0;
                    let zbuffer = b0 & 1 != 0;

                    loggg.push_str(&format!(
                        "DP: Fill triangle (S={}, T={}, Z={})",
                        zbuffer, texture, shade
                    ));

                    index += 32;

                    if shade {
                        index += 64;
                    }

                    if texture {
                        index += 64;
                    }

                    if zbuffer {
                        index += 16;
                    }
                }
                0x24 | 0x25 => {
                    loggg.push_str(&"DP: Texture Rectangle");
                    index += 16;
                }
                0x26 => {
                    loggg.push_str(&"DP: Sync Load");
                    index += 8;
                }
                0x27 => {
                    loggg.push_str(&"DP: Sync Pipe");
                    index += 8;
                }
                0x28 => {
                    loggg.push_str(&"DP: Sync Tile");
                    index += 8;
                }
                0x29 => {
                    loggg.push_str(&"DP: Sync Full");

                    s.mi.set_pending_interrupt(Interrupt::Dp, &mut s.cop0); // TOD temp
                    index += 8;
                }
                0x2A => {
                    loggg.push_str(&"DP: Set key GB");
                    index += 8;
                }
                0x2B => {
                    loggg.push_str(&"DP: Set key R");
                    index += 8;
                }
                0x2C => {
                    loggg.push_str(&"DP: Set convert");
                    index += 8;
                }
                0x2D => {
                    loggg.push_str(&"DP: Set scissor");
                    index += 8;
                }
                0x2E => {
                    loggg.push_str(&"DP: Set prim depth");
                    index += 8;
                }
                0x2F => {
                    loggg.push_str(&"DP: Set other mode");
                    index += 8;
                }
                0x30 => {
                    loggg.push_str(&"DP: Load TLUT");
                    index += 8;
                }
                0x32 => {
                    loggg.push_str(&"DP: Set tile size");
                    index += 8;
                }
                0x33 => {
                    loggg.push_str(&"DP: Load block");
                    index += 8;
                }
                0x34 => {
                    loggg.push_str(&"DP: Load tile");
                    index += 8;
                }
                0x35 => {
                    loggg.push_str(&"DP: Set tile");
                    index += 8;
                }
                0x36 => {
                    loggg.push_str(&"DP: fill rect");
                    index += 8;
                }
                0x37 => {
                    loggg.push_str(&"DP: fill color");
                    index += 8;
                }
                0x38 => {
                    loggg.push_str(&"DP: set fog color");
                    index += 8;
                }
                0x39 => {
                    loggg.push_str(&"DP: set blend color");
                    index += 8;
                }
                0x3A => {
                    loggg.push_str(&"DP: set prim color");
                    index += 8;
                }
                0x3B => {
                    loggg.push_str(&"DP: set env color");
                    index += 8;
                }
                0x3C => {
                    loggg.push_str(&"DP: set combine");
                    index += 8;
                }
                0x3D => {
                    loggg.push_str(&"DP: set timg");
                    index += 8;
                }
                0x3E => {
                    loggg.push_str(&"DP: set zimg");
                    index += 8;
                }
                0x3F => {
                    loggg.push_str(&"DP: set cimg");
                    index += 8;
                }
                _ => panic!("Unknown DP DMA command: {:X}", b0 & 0x3F),
            }

            if false {
                log::debug!("{}", loggg);
            }
        }

        s.dp.regs[CURRENT_REG as usize] = s.dp.regs[END_REG as usize]; // TODO latest addr? what if not "aligned"?

        s.dp.regs[STATUS_REG as usize] &= !STATUS_END_PENDING;
    }
}
