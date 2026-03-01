#![no_std]

// 启动信息结构
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootInfo {
    pub memory_map: *const MemoryMapEntry,
    pub memory_map_count: u32,
    pub framebuffer: FramebufferInfo,
    pub kernel_base: u64,
    pub kernel_size: u64,
}

// 内存映射条目
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemoryMapEntry {
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub memory_type: u32,
    pub attribute: u64,
}

// 帧缓冲区信息
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
}

// 导出各个模块
pub mod console;
pub mod interrupt;
pub mod mm;
pub mod syscall;
pub mod task;

// 内核初始化函数
pub fn init() {
    // 初始化内存管理
    // 注意：物理内存管理需要从 boot_info 获取内存映射
    // 这里暂时使用空指针，实际使用时需要传递正确的 boot_info
    mm::physical::init(core::ptr::null_mut());
    mm::virt::init();
    
    // 初始化中断处理
    interrupt::init_idt();
    interrupt::register_handlers();
    
    // 初始化进程管理
    task::init();
    
    // 初始化系统调用
    syscall::init();
}
