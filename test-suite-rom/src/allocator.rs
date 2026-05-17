use core::ptr;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!(
        "allocation error for {} bytes (align: {})",
        layout.size(),
        layout.align()
    )
}

pub fn configure() {
    // The libdragon IPL3 stores the total memory size at the start of DMEM
    // https://github.com/DragonMinded/libdragon/blob/573bee1c3a1cc4a56e7940bd3912e38fc2ad5f23/boot/README.md?plain=1#L127

    let memory_size =
        unsafe { ptr::with_exposed_provenance::<u32>(0xA400_0000).read_volatile() } as usize;

    // We can start the heap afer the last ELF section, which is .bss

    unsafe extern "C" {
        static __bss_end: u32;
    }

    let bss_end = ptr::addr_of!(__bss_end) as usize;

    let heap_start = (bss_end & 0x1FFF_FFFF) | 0x8000_0000; // TODO masking not needed?

    // The heap itself may eventually conflict with the program's stack (which grows backwards from the end of RDRAM).
    // To avoid this, the size is shrunk by a fixed amount.
    // That amount might need to be tuned for each program.

    const STACK_SIZE: usize = 128 * 1024;

    let heap_size = memory_size - (bss_end & 0x1FFF_FFFF) - STACK_SIZE;

    // Initialize the global allocator

    unsafe {
        ALLOCATOR.lock().init(
            ptr::with_exposed_provenance_mut(heap_start),
            heap_size - STACK_SIZE,
        );
    }
}

pub fn size() -> usize {
    ALLOCATOR.lock().size()
}

pub fn used() -> usize {
    ALLOCATOR.lock().used()
}
