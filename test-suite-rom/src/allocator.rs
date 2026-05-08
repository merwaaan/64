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
    unsafe extern "C" {
        static __bss_end: u32;
    }

    let bss_end = (&raw const __bss_end).addr();

    let heap_start = (bss_end & 0x1FFF_FFFF) | 0x8000_0000;

    // The libdragon IPL3 stores the total memory size at the start of DMEM
    // https://github.com/DragonMinded/libdragon/blob/573bee1c3a1cc4a56e7940bd3912e38fc2ad5f23/boot/README.md?plain=1#L127

    let total_memory =
        unsafe { ptr::with_exposed_provenance::<u32>(0xA400_0000).read_volatile() } as usize;

    let heap_size = total_memory - (bss_end & 0x1FFF_FFFF); // Remaining unused RDRAM

    unsafe {
        const STACK_PADDING: usize = 128 * 1024;

        // The allocator requires a pointer to where in memory the heap should start.
        //
        // Given the above code and the linker script, `heap_start` *should* point to the next
        // available byte in RDRAM. This location *should* be outside any existing heap/stack/static
        // allocation and thus safe to create a pointer to.
        //
        // However, the heap itself may eventually conflict with the program's stack (which grows
        // backwards from the end of RDRAM). To help avoid this, the size is shrunk by an arbitrary
        // amount. But keep in mind it's still possible for the stack to grow far enough that it
        // overlaps with used heap memory.
        //
        // If this happens, try increasing the stack padding.

        ALLOCATOR.lock().init(
            ptr::with_exposed_provenance_mut(heap_start),
            heap_size - STACK_PADDING,
        );
    }
}

pub fn size() -> usize {
    ALLOCATOR.lock().size()
}

pub fn used() -> usize {
    ALLOCATOR.lock().used()
}
