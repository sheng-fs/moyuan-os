/* Linker script for x86_64 kernel */

OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(x86_64)
ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5); /* R_E */
    data PT_LOAD FLAGS(6); /* R_W */
}

SECTIONS {
    . = 0x800000;

    .text : {
        *(.text)
        *(.rodata)
    } :text

    .data : {
        *(.data)
        *(.bss)
    } :data

    /* PVH ELF Note */
    .note.pvh : {
        . = ALIGN(4);
        LONG(0x00000004) /* namesz */
        LONG(0x00000000) /* descsz */
        LONG(0x00000001) /* type */
        BYTE(0x50) BYTE(0x56) BYTE(0x48) BYTE(0x00) /* "PVH" */
        . = ALIGN(8);
    } :text
}
