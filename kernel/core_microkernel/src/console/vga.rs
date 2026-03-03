// VGA输出模块

use core::fmt;

// VGA文本模式常量
const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

// 显示模式枚举
pub enum DisplayMode {
    Text,
    Graphics,
}

// 当前显示模式
static mut CURRENT_MODE: DisplayMode = DisplayMode::Text;

// 文本模式状态
static mut ROW: usize = 0;
static mut COL: usize = 0;
const WHITE_ON_BLACK: u8 = 0x0F;

// 图形模式状态
static mut FRAMEBUFFER_ADDR: u64 = 0;
static mut FRAMEBUFFER_WIDTH: usize = 1024;
static mut FRAMEBUFFER_HEIGHT: usize = 768;
static mut FRAMEBUFFER_PITCH: usize = 0;
static mut FRAMEBUFFER_BPP: usize = 32;

/// 初始化VGA
pub fn init() {
    unsafe {
        clear_screen();
        ROW = 0;
        COL = 0;
        CURRENT_MODE = DisplayMode::Text;
    }
}

/// 设置帧缓冲区信息（用于图形模式）
pub fn set_framebuffer(addr: u64, width: usize, height: usize, pitch: usize, bpp: usize) {
    unsafe {
        FRAMEBUFFER_ADDR = addr;
        FRAMEBUFFER_WIDTH = width;
        FRAMEBUFFER_HEIGHT = height;
        FRAMEBUFFER_PITCH = pitch;
        FRAMEBUFFER_BPP = bpp;
    }
}

/// 设置显示模式
pub fn set_mode(mode: DisplayMode) {
    unsafe {
        CURRENT_MODE = mode;
    }
}

/// 清空屏幕
pub fn clear_screen() {
    unsafe {
        match CURRENT_MODE {
            DisplayMode::Text => {
                for y in 0..VGA_HEIGHT {
                    for x in 0..VGA_WIDTH {
                        let index = y * VGA_WIDTH + x;
                        *VGA_BUFFER.offset(index as isize) = 0x20 | (WHITE_ON_BLACK as u16) << 8;
                    }
                }
                ROW = 0;
                COL = 0;
            }
            DisplayMode::Graphics => {
                if FRAMEBUFFER_ADDR != 0 {
                    let buffer = core::slice::from_raw_parts_mut(FRAMEBUFFER_ADDR as *mut u8, FRAMEBUFFER_PITCH * FRAMEBUFFER_HEIGHT);
                    for byte in buffer.iter_mut() {
                        *byte = 0;
                    }
                }
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

/// 设置光标位置
pub fn set_cursor_pos(x: usize, y: usize) {
    unsafe {
        if x < VGA_WIDTH && y < VGA_HEIGHT {
            COL = x;
            ROW = y;
        }
    }
}

/// 打印一个字符
pub fn put_char(c: char) {
    unsafe {
        match CURRENT_MODE {
            DisplayMode::Text => {
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
            DisplayMode::Graphics => {
                // 图形模式下的字符显示需要实现字体渲染
                // 这里暂时不实现
            }
        }
    }
}

/// 在指定位置显示字符
pub fn print_char(c: char, x: usize, y: usize, fg_color: u8, bg_color: u8) {
    unsafe {
        if x < VGA_WIDTH && y < VGA_HEIGHT {
            let index = y * VGA_WIDTH + x;
            let color = (bg_color << 4) | fg_color;
            *VGA_BUFFER.offset(index as isize) = c as u16 | (color as u16) << 8;
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

/// 绘制像素（图形模式）
pub fn draw_pixel(x: usize, y: usize, color: u32) {
    unsafe {
        if let DisplayMode::Graphics = CURRENT_MODE {
            if FRAMEBUFFER_ADDR != 0 && x < FRAMEBUFFER_WIDTH && y < FRAMEBUFFER_HEIGHT {
                let byte_offset = y * FRAMEBUFFER_PITCH + x * (FRAMEBUFFER_BPP / 8);
                let buffer = core::slice::from_raw_parts_mut(FRAMEBUFFER_ADDR as *mut u8, FRAMEBUFFER_PITCH * FRAMEBUFFER_HEIGHT);
                if byte_offset + 3 < buffer.len() {
                    buffer[byte_offset] = (color & 0xFF) as u8;
                    buffer[byte_offset + 1] = ((color >> 8) & 0xFF) as u8;
                    buffer[byte_offset + 2] = ((color >> 16) & 0xFF) as u8;
                    buffer[byte_offset + 3] = ((color >> 24) & 0xFF) as u8;
                }
            }
        }
    }
}

/// 绘制直线（图形模式）
pub fn draw_line(x1: usize, y1: usize, x2: usize, y2: usize, color: u32) {
    // 使用Bresenham算法绘制直线
    let mut x = x1 as isize;
    let mut y = y1 as isize;
    let dx = (x2 as isize - x).abs();
    let dy = (y2 as isize - y).abs();
    let sx = if x < x2 as isize { 1 } else { -1 };
    let sy = if y < y2 as isize { 1 } else { -1 };
    let mut err = dx - dy;
    
    loop {
        draw_pixel(x as usize, y as usize, color);
        if x == x2 as isize && y == y2 as isize {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// 绘制矩形（图形模式）
pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: u32, filled: bool) {
    if filled {
        // 绘制填充矩形
        for dy in 0..height {
            for dx in 0..width {
                draw_pixel(x + dx, y + dy, color);
            }
        }
    } else {
        // 绘制矩形边框
        draw_line(x, y, x + width - 1, y, color);
        draw_line(x, y + height - 1, x + width - 1, y + height - 1, color);
        draw_line(x, y, x, y + height - 1, color);
        draw_line(x + width - 1, y, x + width - 1, y + height - 1, color);
    }
}