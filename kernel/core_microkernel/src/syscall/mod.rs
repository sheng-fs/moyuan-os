pub mod syscall;
pub use syscall::*;

// 系统调用入口点
#[no_mangle]
extern "C" fn syscall_entry(rax: usize, rdi: usize, rsi: usize, rdx: usize, rcx: usize, r8: usize, r9: usize) -> isize {
    // 处理系统调用
    handle_syscall(rax, rdi, rsi, rdx, rcx, r8, r9)
}

// 初始化系统调用
pub fn init() {
    // 初始化文件系统
    syscall::init_fs();
    
    // 配置系统调用寄存器
    unsafe {
        // 设置LSTAR寄存器，指向系统调用入口点
        core::arch::asm!(
            "lea rax, [rip+syscall_entry]",
            "mov rcx, 0xC0000082", // LSTAR
            "wrmsr",
            options(nostack)
        );
        
        // 设置STAR寄存器
        core::arch::asm!(
            "mov rax, 0x001b000800000000", // 内核代码段选择子: 0x001b, 用户代码段选择子: 0x0008
            "mov rdx, 0x0013001000000000", // 内核数据段选择子: 0x0013, 用户数据段选择子: 0x0010
            "mov rcx, 0xC0000081", // STAR
            "wrmsr",
            options(nostack)
        );
        
        // 设置SFMASK寄存器，清除IF标志
        core::arch::asm!(
            "mov rax, 0",
            "mov rcx, 0xC0000084", // SFMASK
            "wrmsr",
            options(nostack)
        );
    }
}
