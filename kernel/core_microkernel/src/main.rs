#![no_std]
#![no_main]

#[allow(unused_imports)]
use core::panic::PanicInfo;

mod console;
mod interrupt;

#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const MemoryMapEntry,
    pub memory_map_count: u32,
    pub framebuffer: FramebufferInfo,
    pub kernel_base: u64,
    pub kernel_size: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemoryMapEntry {
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub memory_type: u32,
    pub attribute: u64,
}

#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
}

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack)
    );
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack)
    );
    value
}

fn is_transmit_empty(port: u16) -> bool {
    unsafe { inb(port + 5) & 0x20 != 0 }
}

fn serial_putc(port: u16, c: u8) {
    while !is_transmit_empty(port) {}
    unsafe { outb(port, c); }
}

fn serial_puts(port: u16, s: &[u8]) {
    for &c in s {
        serial_putc(port, c);
    }
}

fn print_hex(port: u16, value: u64) {
    let hex_chars = b"0123456789ABCDEF";
    serial_puts(port, b"0x");
    for i in 0..16 {
        let nibble = ((value >> (60 - 4 * i)) & 0xF) as usize;
        serial_putc(port, hex_chars[nibble]);
    }
}

#[no_mangle]
extern "C" fn _start(boot_info: *mut BootInfo) -> ! {
    const SERIAL_PORT: u16 = 0x3F8;
    
    use core::fmt;
    use core::fmt::Write;
    
    unsafe {
        outb(SERIAL_PORT + 1, 0x00);
        outb(SERIAL_PORT + 3, 0x80);
        outb(SERIAL_PORT + 0, 0x03);
        outb(SERIAL_PORT + 1, 0x00);
        outb(SERIAL_PORT + 3, 0x03);
        outb(SERIAL_PORT + 2, 0xC7);
        outb(SERIAL_PORT + 4, 0x0B);

        serial_puts(SERIAL_PORT, b"[KERNEL] MOYUAN OS Kernel Started!\n");
        serial_puts(SERIAL_PORT, b"[KERNEL] Boot Info Addr: ");
        print_hex(SERIAL_PORT, boot_info as u64);
        serial_puts(SERIAL_PORT, b"\n");
        serial_puts(SERIAL_PORT, b"[KERNEL] MOYUAN OS Kernel Booting...\n");
        serial_puts(SERIAL_PORT, b"[KERNEL] Kernel Base: ");
        print_hex(SERIAL_PORT, (*boot_info).kernel_base);
        serial_puts(SERIAL_PORT, b", Size: ");
        let kernel_size = (*boot_info).kernel_size;

        struct SerialWriter;
        impl fmt::Write for SerialWriter {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                for byte in s.bytes() {
                    serial_putc(SERIAL_PORT, byte);
                }
                Ok(())
            }
        }
        let _ = write!(SerialWriter, "{}", kernel_size);

        serial_puts(SERIAL_PORT, b" bytes\n");
        serial_puts(SERIAL_PORT, b"[KERNEL] Initialization Complete!\n");
        serial_puts(SERIAL_PORT, b"[KERNEL] Welcome to MOYUAN OS!\n");
    }
    
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
