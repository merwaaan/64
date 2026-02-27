use strum::{Display, EnumIter};

use crate::{exception::Exception, system::System};

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[repr(u32)]
pub enum Interrupt {
    Sp = 1,
    Si = 1 << 1,
    Ai = 1 << 2,
    Vi = 1 << 3,
    Pi = 1 << 4,
    Dp = 1 << 5,
}

impl Interrupt {
    /// Raises pending interrupts ready to be serviced
    pub fn check(s: &mut System) {
        // We can only raise interrupts if:
        // - Interrupts are globally enabled
        // - We are not currently handling an exception
        // - We are not currently handling an error exception

        let enabled = s.cop0.ie() && !s.cop0.exl() && !s.cop0.erl();

        if enabled {
            // Then, combine the interrupt mask and pending bits

            let mask = s.cop0.interrupt_mask();
            let pending = s.cop0.interrupt_pending();

            let interrupts = mask & pending;

            if interrupts != 0 {
                Exception::Interrupt.raise(s);
            }
        }
    }
}
