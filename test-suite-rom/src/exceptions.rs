use core::{
    arch::{asm, naked_asm},
    ops::Deref,
    ptr::addr_of_mut,
};

use alloc::boxed::Box;
use n64_specs::exception::Exception;

/// Trait for custom exception handlers.
pub trait ExceptionHandler {
    fn run(&mut self);
}

static mut CUSTOM_EXCEPTION_HANDLER: Option<*mut dyn ExceptionHandler> = None;

/// Custom exception handler scope that keeps the handler installed until dropped.
#[must_use = "keeps the handler installed until dropped"]
pub struct ExceptionHandlerScope<H: ExceptionHandler + 'static> {
    handler: Box<H>,
}

impl<H: ExceptionHandler + 'static> ExceptionHandlerScope<H> {
    fn new(handler: H) -> Self {
        let mut scope = Self {
            handler: Box::new(handler),
        };

        unsafe {
            if (*addr_of_mut!(CUSTOM_EXCEPTION_HANDLER)).is_some() {
                panic!("Custom exception handler already installed");
            }

            CUSTOM_EXCEPTION_HANDLER = Some(scope.handler.as_mut());
        }

        scope
    }
}

impl<H: ExceptionHandler + 'static> Drop for ExceptionHandlerScope<H> {
    fn drop(&mut self) {
        unsafe {
            CUSTOM_EXCEPTION_HANDLER = None;
        }
    }
}

impl<H: ExceptionHandler + 'static> Deref for ExceptionHandlerScope<H> {
    type Target = H;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}

/// Installs a custom exception handler.
///
/// Returns a scope that will uninstall the handler when dropped.
///
/// The handler will be called each time an exception occurs until it gets uninstalled.
pub fn install_exception_handler<H: ExceptionHandler + 'static>(
    handler: H,
) -> ExceptionHandlerScope<H> {
    ExceptionHandlerScope::new(handler)
}

/// A convenience exception handler that tracks the exceptions that occurred.
pub struct ExceptionTracker {
    // TODO more general
    // TODO mult exceptions?
    pub occurred: bool,
    pub syscall: bool,
}

impl ExceptionTracker {
    pub fn new() -> Self {
        Self {
            occurred: false,
            syscall: false,
        }
    }
}

impl ExceptionHandler for ExceptionTracker {
    fn run(&mut self) {
        self.occurred = true;

        let cause: u32;

        unsafe {
            core::arch::asm!(
                "mfc0 {cause}, $13",
                cause = out(reg) cause,
                options(nostack, preserves_flags),
            );
        }

        let exccode = (cause >> 2) & 0x1F;

        // Syscall: manually advance EPC to avoid repeating the syscall instruction

        if exccode == Exception::Syscall.exception_code() {
            self.syscall = true;
        }
    }
}

// The following exception handlers are mapped in linker.ld

#[naked]
#[unsafe(link_section = ".exception.tlb_refill")]
#[unsafe(no_mangle)]
extern "C" fn exception_tlb_refill() -> ! {
    unsafe { naked_asm!("j exception_handler", "nop") }
}

#[naked]
#[unsafe(link_section = ".exception.xtlb_refill")]
#[unsafe(no_mangle)]
extern "C" fn exception_xtlb_refill() -> ! {
    unsafe { naked_asm!("j exception_handler", "nop") }
}

#[naked]
#[unsafe(link_section = ".exception.cache")]
#[unsafe(no_mangle)]
extern "C" fn exception_cache() -> ! {
    unsafe { naked_asm!("j exception_handler", "nop") }
}

#[naked]
#[unsafe(link_section = ".exception.general")]
#[unsafe(no_mangle)]
extern "C" fn exception_general() -> ! {
    unsafe { naked_asm!("j exception_handler", "nop") }
}

#[unsafe(no_mangle)]
extern "C" fn exception_handler() -> ! {
    unsafe {
        asm!(
            ".set noat",
            // Save the registers to the stack
            "addiu $sp, $sp, -256", // (32 regs - Zero - SP + HI + LO) * 8 bytes
            "sd  $1,   0($sp)",
            "sd  $2,   8($sp)",
            "sd  $3,  16($sp)",
            "sd  $4,  24($sp)",
            "sd  $5,  32($sp)",
            "sd  $6,  40($sp)",
            "sd  $7,  48($sp)",
            "sd  $8,  56($sp)",
            "sd  $9,  64($sp)",
            "sd $10,  72($sp)",
            "sd $11,  80($sp)",
            "sd $12,  88($sp)",
            "sd $13,  96($sp)",
            "sd $14, 104($sp)",
            "sd $15, 112($sp)",
            "sd $16, 120($sp)",
            "sd $17, 128($sp)",
            "sd $18, 136($sp)",
            "sd $19, 144($sp)",
            "sd $20, 152($sp)",
            "sd $21, 160($sp)",
            "sd $22, 168($sp)",
            "sd $23, 176($sp)",
            "sd $24, 184($sp)",
            "sd $25, 192($sp)",
            "sd $26, 200($sp)",
            "sd $27, 208($sp)",
            "sd $28, 216($sp)",
            "sd $30, 224($sp)",
            "sd $31, 232($sp)",
            "mfhi $31",
            "sd   $31, 240($sp)",
            "mflo $31",
            "sd   $31, 248($sp)",
            // Jump to the actual handler
            "jal exception_handler_implementation",
            "nop",
            // Restore the registers from the stack
            "ld  $1,   0($sp)",
            "ld  $2,   8($sp)",
            "ld  $3,  16($sp)",
            "ld  $4,  24($sp)",
            "ld  $5,  32($sp)",
            "ld  $6,  40($sp)",
            "ld  $7,  48($sp)",
            "ld  $8,  56($sp)",
            "ld  $9,  64($sp)",
            "ld $10,  72($sp)",
            "ld $11,  80($sp)",
            "ld $12,  88($sp)",
            "ld $13,  96($sp)",
            "ld $14, 104($sp)",
            "ld $15, 112($sp)",
            "ld $16, 120($sp)",
            "ld $17, 128($sp)",
            "ld $18, 136($sp)",
            "ld $19, 144($sp)",
            "ld $20, 152($sp)",
            "ld $21, 160($sp)",
            "ld $22, 168($sp)",
            "ld $23, 176($sp)",
            "ld $24, 184($sp)",
            "ld $25, 192($sp)",
            "ld $26, 200($sp)",
            "ld $27, 208($sp)",
            "ld $28, 216($sp)",
            "ld $30, 224($sp)",
            "ld   $31, 240($sp)",
            "mthi $31",
            "ld   $31, 248($sp)",
            "mtlo $31",
            "ld   $31, 232($sp)",
            "addiu $sp, $sp, 256",
            // Return
            "eret",
            ".set at"
        );
    };

    unreachable!()
}

#[unsafe(no_mangle)]
fn exception_handler_implementation() {
    unsafe {
        if let Some(handler_ptr) = CUSTOM_EXCEPTION_HANDLER {
            (&mut *handler_ptr).run();
        }
    }

    default_exception_handler();
}

fn default_exception_handler() {
    let cause: u32;
    let epc: u32;

    unsafe {
        core::arch::asm!(
            "mfc0 {cause}, $13", // Cause
            "mfc0 {epc}, $14",   // EPC
            cause = out(reg) cause,
            epc = out(reg) epc,
            options(nostack, preserves_flags),
        );
    }

    let exccode = (cause >> 2) & 0x1F;

    // Syscall: manually advance EPC to avoid repeating the syscall instruction

    if exccode == 8 {
        let in_delay_slot = cause & 0x8000_0000 != 0;
        let new_epc = if in_delay_slot {
            epc.wrapping_add(8)
        } else {
            epc.wrapping_add(4)
        };
        unsafe {
            core::arch::asm!(
                "mtc0 {epc}, $14",
                epc = in(reg) new_epc,
                options(nostack, preserves_flags),
            );
        }
    }
}
