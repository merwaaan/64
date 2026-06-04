ENTRY(_entrypoint)

SECTIONS {
    . = 0x80000000;
    .exception.tlb_refill : {
        KEEP(*(.exception.tlb_refill))
    }

    . = 0x80000080;
    .exception.xtlb_refill : {
        KEEP(*(.exception.xtlb_refill))
    }

    . = 0x80000100;
    .exception.cache : {
        KEEP(*(.exception.cache))
    }

    . = 0x80000180;
    .exception.general : {
        KEEP(*(.exception.general))
    }

    . = 0x80000400;

    .boot : {
        *(.boot)
        . = ALIGN(16);
    }

    .text : {
        *(.text .text.*)
        . = ALIGN(16);
    }

    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(16);
    }

    .data : {
        *(.data .data.*)
        . = ALIGN(16);
    }

    .bss : {
        *(.bss .bss.*)
        . = ALIGN(16);
        __bss_end = .;
    }

    /* We'll configure the allocator to start the heap from here */
    __heap_start = .; /* TODO use it istead of bss end */

    /DISCARD/ : {
        *(.MIPS.*)
        *(.comment)
        *(.mdebug.*)
        *(.pdr)
        *(.reginfo)
        *(.got)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
