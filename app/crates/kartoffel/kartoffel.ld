MEMORY {
    ram : ORIGIN = 0x00100000, LENGTH = 128K
}

SECTIONS {
    . = ORIGIN(ram);

    .stack : {
        . += 4096;
        _stack_end = .;
    } > ram

    .text : {
        KEEP(*(.init))
        *(.text .text.*)
        *(.data .data.*)
        *(.rodata .rodata.*)
        *(.sdata .sdata.*)
        *(.srodata .srodata.*)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(.eh_frame)
    } > ram

    _heap_start = .;
}

_heap_end = ORIGIN(ram) + LENGTH(ram) - 1;
