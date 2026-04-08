use core::arch::asm;

// IDT条目类型
#[repr(packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    attributes: u8,
    offset_middle: u16,
    offset_high: u32,
    zero: u32,
}

// IDT指针
#[repr(packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

// IDT表
static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    attributes: 0,
    offset_middle: 0,
    offset_high: 0,
    zero: 0,
}; 256];

// IDT指针
static mut IDT_POINTER: IdtPointer = IdtPointer {
    limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
    base: 0,
};

// 注册中断处理函数
pub fn register_interrupt_handler(vector: u8, handler: fn(), attributes: u8) {
    let handler_addr = handler as u64;
    
    unsafe {
        IDT[vector as usize] = IdtEntry {
            offset_low: (handler_addr & 0xFFFF) as u16,
            selector: 0x08, // 代码段选择子
            ist: 0,
            attributes: attributes,
            offset_middle: ((handler_addr >> 16) & 0xFFFF) as u16,
            offset_high: (handler_addr >> 32) as u32,
            zero: 0,
        };
    }
}

// 初始化IDT
pub fn init_idt() {
    unsafe {
        IDT_POINTER.base = IDT.as_ptr() as u64;
        asm!("lidt [{0}]", in(reg) &IDT_POINTER);
    }
}
