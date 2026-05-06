use arbitrary_int::prelude::*;
use n64_specs as specs;

use crate::{
    cpu,
    events::{EventType, Events},
    location::Location,
    rendering::video::Frame,
    system::{Address, System},
    value::Value,
};

pub type ViLocation = Location<{ specs::vi::START }, { specs::vi::END }>;

// TODO move to specs
const TOTAL_SCANLINES: usize = 525; // TODO depends????
const FRAME_CPU_CYCLES: usize = (cpu::FREQUENCY / specs::timing::NTSC_FREQUENCY) as usize;
pub const SCANLINE_CPU_CYCLES: usize = FRAME_CPU_CYCLES / TOTAL_SCANLINES; // TODO fractional part?

#[derive(Default, Debug, Clone, Copy)]
pub struct Vi {
    regs: specs::vi::Registers,
}

impl Vi {
    pub fn regs(&self) -> &specs::vi::Registers {
        &self.regs
    }

    pub fn read<T: Value>(s: &System, addr: ViLocation) -> T {
        assert!(T::BYTES == 4, "VI: read with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::vi::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "VI: read from unaligned address {:08X}",
            offset
        );

        let regs_slice = bytemuck::cast_slice(bytemuck::bytes_of(&s.vi.regs));
        T::read_reg(regs_slice, offset)
    }

    pub fn write<T: Value>(s: &mut System, addr: ViLocation, data: T) {
        assert!(T::BYTES == 4, "VI: write with invalid size {}", T::BYTES);

        let offset = addr.relative() & specs::vi::REGISTERS_MASK;

        assert!(
            offset & 3 == 0,
            "VI: write to unaligned address {:08X}",
            offset
        );

        let current_line = s.vi.regs.current_line;

        let offset = addr.relative() & specs::vi::REGISTERS_MASK;

        let regs_slice = bytemuck::cast_slice_mut(bytemuck::bytes_of_mut(&mut s.vi.regs));
        data.write_reg(regs_slice, offset);

        if offset == specs::vi::CurrentLine::OFFSET {
            s.mi.clear_pending_interrupt(specs::interrupt::Interrupt::Vi, &mut s.cop0);

            // CURRENT_LINE is read-only
            // TODO add mask to specs
            s.vi.regs.current_line = current_line;
        }
    }

    pub(crate) fn framebuffer_address(&self) -> u32 {
        self.regs.origin.ram_address().value()
    }

    pub fn framebuffer_width(&self) -> usize {
        self.regs.width.value().value() as usize
    }

    pub fn framebuffer_height(&self) -> usize {
        480 // TODOself.regs[V_SYNC_REG] as usize
    }

    pub fn scanline_completed(s: &mut System) {
        // Increment the current scanline by 2 half scanlines
        // TODO Toggle the field bit?

        s.vi.regs
            .current_line
            .set_line(s.vi.regs.current_line.line().wrapping_add(u9::new(1))); // TODO halfline overlap in struct?
        //s.vi.regs[CURRENT_SCANLINE_REG] = s.vi.regs[CURRENT_SCANLINE_REG].wrapping_add(2) & 0x3FF;

        // Reset the current scanline to 0 if it matches the V_SYNC register

        if s.vi.regs.current_line.line().value() >= s.vi.regs.vertical_total.value().value() {
            s.vi.regs.current_line.set_line(u9::ZERO); // TODO halfline overlap?
        }

        // Raise an interrupt if the current scanline matches the interrupt scanline
        // TODO >= or ==???

        if s.vi.regs.current_line.line().value() == s.vi.regs.interrupt_line.value().value() {
            // TODO halfline overlap?
            s.mi.set_pending_interrupt(specs::interrupt::Interrupt::Vi, &mut s.cop0);
        }

        // Schedule the next scanline
        // probably needd to be computed dynamically based on the current height?

        Events::push(s, EventType::ViScanlineComplete, SCANLINE_CPU_CYCLES);
    }

    pub fn extract_framebuffer(s: &mut System) -> Frame {
        let base_addr = s.vi.framebuffer_address();
        let width = s.vi.framebuffer_width();
        let height = s.vi.framebuffer_height();

        let mut rgba = Vec::with_capacity(width * height * 4);

        let color32 = s.vi.regs.control.color_mode().value() == 3;

        if color32 {
            for y in 0..height {
                for x in 0..width {
                    // TODO optim: directly access RAM with read_block
                    let pixel = s
                        .read::<u32>(Address::p(base_addr + ((y * width + x) * 4) as u32))
                        .expect("Invalid pixel address");

                    // TODO use color specs
                    rgba.push((pixel >> 24) as u8);
                    rgba.push((pixel >> 16) as u8);
                    rgba.push((pixel >> 8) as u8);
                    rgba.push(0xFF); // TODO real val
                }
            }
        } else {
            for y in 0..height {
                for x in 0..width {
                    let pixel = s
                        .read::<u16>(Address::p(base_addr + ((y * width + x) * 2) as u32))
                        .expect("Invalid pixel address");

                    // TODO use color specs
                    rgba.push(Self::b5_to_b8(pixel >> 11));
                    rgba.push(Self::b5_to_b8(pixel >> 6));
                    rgba.push(Self::b5_to_b8(pixel >> 1));
                    rgba.push(0xFF); // TODO real val
                }
            }
        }

        Frame {
            index: 0,
            rgba,
            width,
            height,
        }
    }

    // TODO move out, used elsewhere
    fn b5_to_b8(value: u16) -> u8 {
        (((value & 0x1F) * 255) / 31) as u8
    }
}
