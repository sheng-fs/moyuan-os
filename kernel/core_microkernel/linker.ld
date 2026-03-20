/* Linker script for x86_64 kernel */

OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(x86_64)
ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5); /* R_E */
    data PT_LOAD FLAGS(6); /* R_W */
}

SECTIONS {
    . = 0x200000;

    /* PVH ELF Note */
    .note.pvh : {
        . = ALIGN(8);
        BYTE(0x04) BYTE(0x00) BYTE(0x00) BYTE(0x00) /* namesz */
        BYTE(0x00) BYTE(0x00) BYTE(0x00) BYTE(0x00) /* descsz */
        BYTE(0x01) BYTE(0x00) BYTE(0x00) BYTE(0x00) /* type */
        BYTE(0x50) BYTE(0x56) BYTE(0x48) BYTE(0x00) /* "PVH" */
        . = ALIGN(8);
    } :text

    /* Text section */
    .text : {
        *(.text)
        *(.rodata)
    } :text

    /* Data section */
    .data : {
        *(.data)
        *(.bss)
    } :data
}
