// VGA输出模块

use core::fmt;

const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

static mut ROW: usize = 0;
static mut COL: usize = 0;
const WHITE_ON_BLACK: u8 = 0x0F;

/// 初始化VGA
pub fn init() {
    unsafe {
        clear_screen();
        ROW = 0;
        COL = 0;
    }
}

/// 清空屏幕
fn clear_screen() {
    unsafe {
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let index = y * VGA_WIDTH + x;
                *VGA_BUFFER.offset(index as isize) = 0x20 | (WHITE_ON_BLACK as u16) << 8;
            }
        }
    }
}

/// 滚动屏幕
fn scroll() {
    unsafe {
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let src_index = y * VGA_WIDTH + x;
                let dst_index = (y - 1) * VGA_WIDTH + x;
                *VGA_BUFFER.offset(dst_index as isize) = *VGA_BUFFER.offset(src_index as isize);
            }
        }
        
        // 清空最后一行
        for x in 0..VGA_WIDTH {
            let index = (VGA_HEIGHT - 1) * VGA_WIDTH + x;
            *VGA_BUFFER.offset(index as isize) = 0x20 | (WHITE_ON_BLACK as u16) << 8;
        }
        
        ROW = VGA_HEIGHT - 1;
        COL = 0;
    }
}

/// 打印一个字符
fn put_char(c: char) {
    unsafe {
        match c {
            '\n' => {
                ROW += 1;
                COL = 0;
            }
            '\t' => {
                COL = (COL + 4) & !3;
            }
            _ => {
                if COL >= VGA_WIDTH {
                    ROW += 1;
                    COL = 0;
                }
                
                if ROW >= VGA_HEIGHT {
                    scroll();
                }
                
                let index = ROW * VGA_WIDTH + COL;
                *VGA_BUFFER.offset(index as isize) = c as u16 | (WHITE_ON_BLACK as u16) << 8;
                COL += 1;
            }
        }
    }
}

/// 打印格式化字符串到VGA
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    struct VgaWriter;
    impl fmt::Write for VgaWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for c in s.chars() {
                put_char(c);
            }
            Ok(())
        }
    }
    VgaWriter.write_fmt(args).unwrap();
}