// 键盘驱动模块

use core::arch::asm;
use crate::interrupt;

// 键盘端口
const KEYBOARD_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;

// 扫描码集1映射表
static SCANCODE_MAP: [Option<char>; 128] = [
    None,           // 0
    Some('`'),      // 1
    Some('1'),      // 2
    Some('2'),      // 3
    Some('3'),      // 4
    Some('4'),      // 5
    Some('5'),      // 6
    Some('6'),      // 7
    Some('7'),      // 8
    Some('8'),      // 9
    Some('9'),      // 10
    Some('0'),      // 11
    Some('-'),      // 12
    Some('='),      // 13
    Some('\x08'),   // 14 (Backspace)
    Some('\t'),     // 15 (Tab)
    Some('q'),      // 16
    Some('w'),      // 17
    Some('e'),      // 18
    Some('r'),      // 19
    Some('t'),      // 20
    Some('y'),      // 21
    Some('u'),      // 22
    Some('i'),      // 23
    Some('o'),      // 24
    Some('p'),      // 25
    Some('['),      // 26
    Some(']'),      // 27
    Some('\n'),     // 28 (Enter)
    None,           // 29 (Left Ctrl)
    Some('a'),      // 30
    Some('s'),      // 31
    Some('d'),      // 32
    Some('f'),      // 33
    Some('g'),      // 34
    Some('h'),      // 35
    Some('j'),      // 36
    Some('k'),      // 37
    Some('l'),      // 38
    Some(';'),      // 39
    Some('\''),     // 40
    Some('`'),      // 41
    None,           // 42 (Left Shift)
    Some('\\'),     // 43
    Some('z'),      // 44
    Some('x'),      // 45
    Some('c'),      // 46
    Some('v'),      // 47
    Some('b'),      // 48
    Some('n'),      // 49
    Some('m'),      // 50
    Some(','),      // 51
    Some('.'),      // 52
    Some('/'),      // 53
    None,           // 54 (Right Shift)
    Some('*'),      // 55
    None,           // 56 (Left Alt)
    Some(' '),      // 57 (Space)
    None,           // 58 (Caps Lock)
    None,           // 59 (F1)
    None,           // 60 (F2)
    None,           // 61 (F3)
    None,           // 62 (F4)
    None,           // 63 (F5)
    None,           // 64 (F6)
    None,           // 65 (F7)
    None,           // 66 (F8)
    None,           // 67 (F9)
    None,           // 68 (F10)
    None,           // 69 (Num Lock)
    None,           // 70 (Scroll Lock)
    None,           // 71 (Home)
    None,           // 72 (Up Arrow)
    None,           // 73 (Page Up)
    Some('-'),      // 74
    None,           // 75 (Left Arrow)
    None,           // 76
    None,           // 77 (Right Arrow)
    Some('+'),      // 78
    None,           // 79 (End)
    None,           // 80 (Down Arrow)
    None,           // 81 (Page Down)
    None,           // 82 (Insert)
    None,           // 83 (Delete)
    None,           // 84
    None,           // 85
    None,           // 86
    None,           // 87 (F11)
    None,           // 88 (F12)
    None,           // 89
    None,           // 90
    None,           // 91
    None,           // 92
    None,           // 93
    None,           // 94
    None,           // 95
    None,           // 96
    None,           // 97
    None,           // 98
    None,           // 99
    None,           // 100
    None,           // 101
    None,           // 102
    None,           // 103
    None,           // 104
    None,           // 105
    None,           // 106
    None,           // 107
    None,           // 108
    None,           // 109
    None,           // 110
    None,           // 111
    None,           // 112
    None,           // 113
    None,           // 114
    None,           // 115
    None,           // 116
    None,           // 117
    None,           // 118
    None,           // 119
    None,           // 120
    None,           // 121
    None,           // 122
    None,           // 123
    None,           // 124
    None,           // 125
    None,           // 126
    None,           // 127
];

// 大写扫描码映射表
static SCANCODE_MAP_SHIFT: [Option<char>; 128] = [
    None,           // 0
    Some('~'),      // 1
    Some('!'),      // 2
    Some('@'),      // 3
    Some('#'),      // 4
    Some('$'),      // 5
    Some('%'),      // 6
    Some('^'),      // 7
    Some('&'),      // 8
    Some('*'),      // 9
    Some('('),      // 10
    Some(')'),      // 11
    Some('_'),      // 12
    Some('+'),      // 13
    Some('\x08'),   // 14 (Backspace)
    Some('\t'),     // 15 (Tab)
    Some('Q'),      // 16
    Some('W'),      // 17
    Some('E'),      // 18
    Some('R'),      // 19
    Some('T'),      // 20
    Some('Y'),      // 21
    Some('U'),      // 22
    Some('I'),      // 23
    Some('O'),      // 24
    Some('P'),      // 25
    Some('{'),      // 26
    Some('}'),      // 27
    Some('\n'),     // 28 (Enter)
    None,           // 29 (Left Ctrl)
    Some('A'),      // 30
    Some('S'),      // 31
    Some('D'),      // 32
    Some('F'),      // 33
    Some('G'),      // 34
    Some('H'),      // 35
    Some('J'),      // 36
    Some('K'),      // 37
    Some('L'),      // 38
    Some(':'),      // 39
    Some('"'),      // 40
    Some('~'),      // 41
    None,           // 42 (Left Shift)
    Some('|'),      // 43
    Some('Z'),      // 44
    Some('X'),      // 45
    Some('C'),      // 46
    Some('V'),      // 47
    Some('B'),      // 48
    Some('N'),      // 49
    Some('M'),      // 50
    Some('<'),      // 51
    Some('>'),      // 52
    Some('?'),      // 53
    None,           // 54 (Right Shift)
    Some('*'),      // 55
    None,           // 56 (Left Alt)
    Some(' '),      // 57 (Space)
    None,           // 58 (Caps Lock)
    None,           // 59 (F1)
    None,           // 60 (F2)
    None,           // 61 (F3)
    None,           // 62 (F4)
    None,           // 63 (F5)
    None,           // 64 (F6)
    None,           // 65 (F7)
    None,           // 66 (F8)
    None,           // 67 (F9)
    None,           // 68 (F10)
    None,           // 69 (Num Lock)
    None,           // 70 (Scroll Lock)
    None,           // 71 (Home)
    None,           // 72 (Up Arrow)
    None,           // 73 (Page Up)
    Some('-'),      // 74
    None,           // 75 (Left Arrow)
    None,           // 76
    None,           // 77 (Right Arrow)
    Some('+'),      // 78
    None,           // 79 (End)
    None,           // 80 (Down Arrow)
    None,           // 81 (Page Down)
    None,           // 82 (Insert)
    None,           // 83 (Delete)
    None,           // 84
    None,           // 85
    None,           // 86
    None,           // 87 (F11)
    None,           // 88 (F12)
    None,           // 89
    None,           // 90
    None,           // 91
    None,           // 92
    None,           // 93
    None,           // 94
    None,           // 95
    None,           // 96
    None,           // 97
    None,           // 98
    None,           // 99
    None,           // 100
    None,           // 101
    None,           // 102
    None,           // 103
    None,           // 104
    None,           // 105
    None,           // 106
    None,           // 107
    None,           // 108
    None,           // 109
    None,           // 110
    None,           // 111
    None,           // 112
    None,           // 113
    None,           // 114
    None,           // 115
    None,           // 116
    None,           // 117
    None,           // 118
    None,           // 119
    None,           // 120
    None,           // 121
    None,           // 122
    None,           // 123
    None,           // 124
    None,           // 125
    None,           // 126
    None,           // 127
];

// 键盘状态
static mut SHIFT_PRESSED: bool = false;
static mut CTRL_PRESSED: bool = false;
static mut ALT_PRESSED: bool = false;
static mut CAPS_LOCK: bool = false;

// 环形缓冲区
const BUFFER_SIZE: usize = 256;
static mut KEYBOARD_BUFFER: [Option<char>; BUFFER_SIZE] = [None; BUFFER_SIZE];
static mut BUFFER_HEAD: usize = 0;
static mut BUFFER_TAIL: usize = 0;

// 检查键盘控制器是否准备好
fn is_keyboard_ready() -> bool {
    unsafe {
        inb(KEYBOARD_STATUS_PORT) & 1 != 0
    }
}

// 读取键盘扫描码
fn read_scancode() -> u8 {
    unsafe {
        while !is_keyboard_ready() {
            // 等待键盘输入
        }
        inb(KEYBOARD_PORT)
    }
}

// 键盘中断处理函数
#[no_mangle]
extern "C" fn keyboard_irq_handler() -> ! {
    unsafe {
        let scancode = read_scancode();
        
        // 处理按键释放事件（扫描码最高位为1）
        if scancode & 0x80 != 0 {
            let key_code = scancode & 0x7F;
            match key_code {
                42 | 54 => SHIFT_PRESSED = false, // 左右Shift键释放
                29 => CTRL_PRESSED = false,       // Left Ctrl键释放
                56 => ALT_PRESSED = false,        // Left Alt键释放
                58 => CAPS_LOCK = !CAPS_LOCK,     // Caps Lock键切换
                _ => {}
            }
        } else {
            // 处理按键按下事件
            let key_code = scancode;
            match key_code {
                42 | 54 => SHIFT_PRESSED = true,  // 左右Shift键按下
                29 => CTRL_PRESSED = true,        // Left Ctrl键按下
                56 => ALT_PRESSED = true,         // Left Alt键按下
                _ => {
                    // 查找对应的字符
                    let char_option = if SHIFT_PRESSED {
                        SCANCODE_MAP_SHIFT[key_code as usize]
                    } else if CAPS_LOCK && key_code >= 16 && key_code <= 35 {
                        // 大写字母
                        SCANCODE_MAP_SHIFT[key_code as usize]
                    } else {
                        SCANCODE_MAP[key_code as usize]
                    };
                    
                    // 将字符添加到缓冲区
                    if let Some(c) = char_option {
                        let next_tail = (BUFFER_TAIL + 1) % BUFFER_SIZE;
                        if next_tail != BUFFER_HEAD {
                            KEYBOARD_BUFFER[BUFFER_TAIL] = Some(c);
                            BUFFER_TAIL = next_tail;
                        }
                    }
                }
            }
        }
        
        // 发送EOI
        asm!(
            "mov al, 0x20",
            "out 0x20, al",
            options(nomem, nostack)
        );
    }
    
    // 从中断返回
    unsafe {
        asm!(
            "iretq",
            options(noreturn)
        );
    }
}

// 初始化键盘
pub fn init() {
    unsafe {
        // 注册键盘中断处理函数
        interrupt::register_interrupt_handler(33, keyboard_irq_handler);
        
        // 启用键盘中断
        let pic_mask = inb(0x21);
        outb(0x21, pic_mask & 0xFD); // 清除IRQ1的屏蔽位
        
        crate::console::print(core::format_args!("键盘初始化成功\n"));
    }
}

// 导出给设备服务的函数
#[no_mangle]
extern "C" fn keyboard_init() {
    init();
}

#[no_mangle]
extern "C" fn keyboard_read_key() -> Option<char> {
    read_key()
}

#[no_mangle]
extern "C" fn keyboard_has_key() -> bool {
    has_key()
}

// 检查是否有可用输入
#[allow(dead_code)]
pub fn has_key() -> bool {
    unsafe {
        BUFFER_HEAD != BUFFER_TAIL
    }
}

// 非阻塞读取一个字符
#[allow(dead_code)]
pub fn read_key() -> Option<char> {
    unsafe {
        if BUFFER_HEAD != BUFFER_TAIL {
            let c = KEYBOARD_BUFFER[BUFFER_HEAD];
            BUFFER_HEAD = (BUFFER_HEAD + 1) % BUFFER_SIZE;
            c
        } else {
            None
        }
    }
}

// 清空输入缓冲区
#[allow(dead_code)]
pub fn clear_buffer() {
    unsafe {
        BUFFER_HEAD = 0;
        BUFFER_TAIL = 0;
        for i in 0..BUFFER_SIZE {
            KEYBOARD_BUFFER[i] = None;
        }
    }
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
