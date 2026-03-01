use core::arch::asm;
use crate::console;

// IDT 条目类型
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    attributes: u8,
    offset_middle: u16,
    offset_high: u32,
    zero: u32,
}

// IDT 描述符
#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

// IDT 表
static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    attributes: 0,
    offset_middle: 0,
    offset_high: 0,
    zero: 0,
}; 256];

// 中断处理函数类型
type InterruptHandler = extern "C" fn() -> !;

// 初始化IDT
pub fn init_idt() {
    unsafe {
        // 设置IDT描述符
        let idt_descriptor = IdtDescriptor {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: IDT.as_ptr() as u64,
        };
        
        // 加载IDT
        let idt_ptr = &idt_descriptor as *const _ as u64;
        asm!(
            "lidt [rax]",
            in("rax") idt_ptr,
            options(readonly, nostack)
        );
        
        crate::console::print(core::format_args!("IDT初始化成功\n"));
    }
}

// 注册中断处理函数
pub fn register_handlers() {
    // 注册CPU异常处理函数
    register_interrupt_handler(0, divide_by_zero_handler);
    register_interrupt_handler(1, debug_handler);
    register_interrupt_handler(2, non_maskable_interrupt_handler);
    register_interrupt_handler(3, breakpoint_handler);
    register_interrupt_handler(4, overflow_handler);
    register_interrupt_handler(5, bound_range_exceeded_handler);
    register_interrupt_handler(6, invalid_opcode_handler);
    register_interrupt_handler(7, device_not_available_handler);
    register_interrupt_handler(8, double_fault_handler);
    register_interrupt_handler(10, invalid_tss_handler);
    register_interrupt_handler(11, segment_not_present_handler);
    register_interrupt_handler(12, stack_segment_fault_handler);
    register_interrupt_handler(13, general_protection_fault_handler);
    register_interrupt_handler(14, page_fault_handler);
    
    // 注册时钟中断处理函数
    register_interrupt_handler(32, timer_interrupt_handler);
    
    crate::console::print(core::format_args!("中断处理函数注册成功\n"));
}

// 注册中断处理函数
fn register_interrupt_handler(interrupt_number: u8, handler: InterruptHandler) {
    unsafe {
        let handler_address = handler as u64;
        IDT[interrupt_number as usize] = IdtEntry {
            offset_low: (handler_address & 0xFFFF) as u16,
            selector: 0x08, // 代码段选择子
            ist: 0,
            attributes: 0x8E, // 中断门，DPL=0
            offset_middle: ((handler_address >> 16) & 0xFFFF) as u16,
            offset_high: (handler_address >> 32) as u32,
            zero: 0,
        };
    }
}

// 中断处理函数
#[no_mangle]
extern "C" fn divide_by_zero_handler() -> ! {
    crate::console::print(core::format_args!("除零异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn debug_handler() -> ! {
    crate::console::print(core::format_args!("调试异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn non_maskable_interrupt_handler() -> ! {
    crate::console::print(core::format_args!("不可屏蔽中断\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn breakpoint_handler() -> ! {
    crate::console::print(core::format_args!("断点异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn overflow_handler() -> ! {
    crate::console::print(core::format_args!("溢出异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn bound_range_exceeded_handler() -> ! {
    crate::console::print(core::format_args!("边界范围超出异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn invalid_opcode_handler() -> ! {
    crate::console::print(core::format_args!("无效操作码异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn device_not_available_handler() -> ! {
    crate::console::print(core::format_args!("设备不可用异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn double_fault_handler() -> ! {
    crate::console::print(core::format_args!("双重故障异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn invalid_tss_handler() -> ! {
    crate::console::print(core::format_args!("无效TSS异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn segment_not_present_handler() -> ! {
    crate::console::print(core::format_args!("段不存在异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn stack_segment_fault_handler() -> ! {
    crate::console::print(core::format_args!("栈段故障异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn general_protection_fault_handler() -> ! {
    crate::console::print(core::format_args!("通用保护故障异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

#[no_mangle]
extern "C" fn page_fault_handler() -> ! {
    crate::console::print(core::format_args!("页故障异常\n"));
    loop { unsafe { asm!("hlt"); } }
}

// 时钟中断处理函数
static mut TIMER_COUNT: u32 = 0;

#[no_mangle]
extern "C" fn timer_interrupt_handler() -> ! {
    unsafe {
        TIMER_COUNT += 1;
        if TIMER_COUNT % 100 == 0 { // 每100个时钟中断（约1秒）打印一次
            crate::console::print(core::format_args!("时钟中断: {}\n", TIMER_COUNT));
        }
        
        // 触发调度
        if TIMER_COUNT % 10 == 0 { // 每10个时钟中断触发一次调度
            crate::task::scheduler::schedule();
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

// 初始化PIC
pub fn init_pic() {
    unsafe {
        // 初始化主PIC
        asm!(
            "mov al, 0x11",
            "out 0x20, al", // 主PIC命令端口
            "out 0xA0, al", // 从PIC命令端口
            
            "mov al, 0x20",
            "out 0x21, al", // 主PIC数据端口，设置中断向量号从32开始
            "mov al, 0x28",
            "out 0xA1, al", // 从PIC数据端口，设置中断向量号从40开始
            
            "mov al, 0x04",
            "out 0x21, al", // 主PIC设置从PIC连接到IRQ2
            "mov al, 0x02",
            "out 0xA1, al", // 从PIC设置连接到主PIC的IRQ2
            
            "mov al, 0x01",
            "out 0x21, al", // 主PIC设置为8086模式
            "out 0xA1, al", // 从PIC设置为8086模式
            
            "mov al, 0xFF",
            "out 0x21, al", // 主PIC屏蔽所有中断
            "out 0xA1, al", // 从PIC屏蔽所有中断
            
            "mov al, 0xFE",
            "out 0x21, al", // 主PIC允许时钟中断
            
            options(nomem, nostack)
        );
        
        crate::console::print(core::format_args!("PIC初始化成功\n"));
    }
}

// 启用中断
pub fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
    crate::console::print(core::format_args!("中断已启用\n"));
}
