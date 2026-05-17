ENTRY(_entrypoint)

SECTIONS {
    /* Start in KSEG0, skip the first 0x400 bytes as legacy data can lie there according to Libdragon */
    . = 0x80000400;

    /* TODO can we use this for the soft reboot? */
    __boot_start = .;

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
