#![no_std]

// Linux兼容层

pub mod syscall;
pub mod elf;
pub mod libc;

// 初始化Linux兼容层
pub fn init() {
    // 初始化系统调用表
    syscall::init();
    
    // 初始化ELF加载器
    elf::init();
    
    // 初始化C库兼容层
    libc::init();
}
