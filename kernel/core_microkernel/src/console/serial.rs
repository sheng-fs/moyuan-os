// 串口输出模块

use core::fmt;

const SERIAL_PORT: u16 = 0x3F8; // COM1 端口

/// 初始化串口
#[allow(dead_code)]
pub fn init() {
    unsafe {
        // 禁用中断
        outb(SERIAL_PORT + 1, 0x00);
        
        // 设置波特率为 9600
        outb(SERIAL_PORT + 3, 0x80); // 启用波特率设置
        outb(SERIAL_PORT + 0, 0x0C); // 波特率低字节 (9600 / 16 = 600, 600 的十六进制是 0x0258, 低字节是 0x58? 不对，应该是 9600 的除数是 12，所以低字节是 0x0C)
        outb(SERIAL_PORT + 1, 0x00); // 波特率高字节
        
        // 设置数据格式: 8 位数据, 1 位停止位, 无校验
        outb(SERIAL_PORT + 3, 0x03);
        
        // 启用 FIFO, 清除缓冲区, 设置触发级别为 14 字节
        outb(SERIAL_PORT + 2, 0xC7);
        
        // 启用中断
        outb(SERIAL_PORT + 1, 0x01);
    }
}

/// 检查串口是否可写
fn is_transmit_empty() -> bool {
    unsafe {
        inb(SERIAL_PORT + 5) & 0x20 != 0
    }
}

/// 发送一个字节到串口
fn send_byte(byte: u8) {
    while !is_transmit_empty() {
        // 等待发送缓冲区为空
    }
    unsafe {
        outb(SERIAL_PORT, byte);
    }
}

/// 打印格式化字符串到串口
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    struct SerialWriter;
    impl fmt::Write for SerialWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for byte in s.bytes() {
                send_byte(byte);
            }
            Ok(())
        }
    }
    SerialWriter.write_fmt(args).unwrap();
}

// 内联汇编函数
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