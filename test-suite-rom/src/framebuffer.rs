use core::ptr::{addr_of, addr_of_mut};

use n64_specs::color::RGBA8888;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
pub const PIXELS: usize = WIDTH as usize * HEIGHT as usize;

pub const WHITE: RGBA8888 = RGBA8888::from_rgba(0xFF, 0xFF, 0xFF, 0xFF);
pub const RED: RGBA8888 = RGBA8888::from_rgba(0xFF, 0x00, 0x00, 0xFF);
pub const GREEN: RGBA8888 = RGBA8888::from_rgba(0x00, 0xFF, 0x00, 0xFF);

// TODO
// - vi regs to specs + use them here
// - add index() func to regs?
const VI_BASE_REG: *mut u32 = 0xA440_0000 as *mut u32;

static mut BUFFER: [u32; PIXELS] = [0; PIXELS];

pub struct Framebuffer;

impl Framebuffer {
    pub fn configure() {
        // TODO be explicit
        unsafe {
            VI_BASE_REG.write_volatile(12879);
            VI_BASE_REG.add(1).write_volatile(addr_of!(BUFFER) as u32);
            VI_BASE_REG.add(2).write_volatile(WIDTH);
            VI_BASE_REG.add(3).write_volatile(2);
            VI_BASE_REG.add(5).write_volatile(0x03E5_2239);
            VI_BASE_REG.add(6).write_volatile(0x0000_020D);
            VI_BASE_REG.add(7).write_volatile(0x0000_0C15);
            VI_BASE_REG.add(8).write_volatile(0x0C15_0C15);
            VI_BASE_REG.add(9).write_volatile(0x006C_02EC);
            VI_BASE_REG.add(10).write_volatile(0x0025_01FF);
            VI_BASE_REG.add(11).write_volatile(0x000E_0204);
            VI_BASE_REG.add(12).write_volatile((0x100 * WIDTH) / 160);
            VI_BASE_REG.add(13).write_volatile((0x100 * HEIGHT) / 60);
        }

        Self::fill(WHITE);
    }

    pub fn fill(color: RGBA8888) {
        let p = addr_of_mut!(BUFFER).cast::<u32>();

        for i in 0..PIXELS {
            unsafe {
                p.add(i).write_volatile(color.raw_value());
            }
        }
    }
}
