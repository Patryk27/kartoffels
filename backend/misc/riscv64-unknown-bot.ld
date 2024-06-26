MEMORY {
    RAM : ORIGIN = 0x00100000, LENGTH = 128K
}

SECTIONS {
    . = ORIGIN(RAM);
    .init : { *(.init) } > RAM
    .text : { *(.text .text.*) } > RAM
    .rodata : { *(.rodata) } > RAM
    .data : { *(.data) } > RAM
    . = ALIGN(4);
}

/* TODO put stack at the beginning of RAM instead */
PROVIDE(_stack_start = ORIGIN(RAM) + LENGTH(RAM));
