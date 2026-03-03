#![no_std]
#![no_main]

#[cfg(all(not(feature = "test"), not(test)))]
use core::panic::PanicInfo;


// 导入各个模块
mod mm;
mod task;
mod syscall;
mod interrupt;
mod console;

// 导入设备服务
use services::device_service;

// 启动信息结构
#[repr(C)]
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
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
}

// 内核主函数
#[no_mangle]
extern "C" fn _start() -> ! {
    // 临时的启动函数，实际的启动逻辑会由UEFI引导加载器调用
    // 这里只是为了避免链接错误
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

// 内核主函数
#[no_mangle]
extern "C" fn kernel_main(boot_info: *mut BootInfo) -> ! {
    // 初始化控制台
    console::init();
    
    // 打印启动信息
    print!("墨渊操作系统内核启动中...\n");
    print!("内核基地址: {:#x}, 大小: {} 字节\n", unsafe { (*boot_info).kernel_base }, unsafe { (*boot_info).kernel_size });
    
    // 初始化内核
    init_kernel(boot_info);
    
    // 打印启动完成信息
    print!("墨渊操作系统启动完成！\n");
    
    // 进入主循环
    loop {
        // 这里将来会运行调度器
        unsafe { core::arch::asm!("hlt"); }
    }
}

// 初始化内核
fn init_kernel(boot_info: *mut BootInfo) {
    // 初始化内存管理
    init_memory(boot_info);
    
    // 初始化中断处理
    init_interrupts();
    
    // 初始化进程管理
    init_processes();
    
    // 初始化系统调用
    init_syscalls();
    
    // 初始化设备服务
    print!("初始化设备服务...\n");
    device_service::init();
}

// 初始化内存管理
fn init_memory(boot_info: *mut BootInfo) {
    // 初始化物理内存管理
    print!("初始化物理内存管理...\n");
    mm::physical::init(boot_info);
    
    // 初始化虚拟内存管理
    print!("初始化虚拟内存管理...\n");
    mm::virt::init();
}

// 初始化中断处理
fn init_interrupts() {
    print!("初始化中断处理...\n");
    // 初始化IDT
    interrupt::init_idt();
    
    // 注册中断处理函数
    interrupt::register_handlers();
    
    // 初始化PIC
    interrupt::init_pic();
    
    // 启用中断
    interrupt::enable_interrupts();
}

// 初始化进程管理
fn init_processes() {
    print!("初始化进程管理...\n");
    // 初始化进程管理系统
    task::init();
    
    // 测试多进程创建和调度
    test_multiprocess();
}

// 测试多进程创建和调度
fn test_multiprocess() {
    print!("测试多进程创建和调度...\n");
    
    // 创建两个子进程
    for _i in 0..2 {
        // 这里简化处理，实际应该通过系统调用创建进程
        if let Some(child_pid) = task::process::process_create(test_process_entry as *const () as u64, 4096) {
            task::scheduler::add_to_ready_queue(child_pid);
            print!("创建子进程: {}\n", child_pid);
        }
    }
}

// 测试进程入口点
fn test_process_entry() {
    if let Some(process) = task::process::get_current_process() {
        let pid = process.pid;
        loop {
            print!("进程 {} 运行中...\n", pid);
            // 简单的延迟
            for _ in 0..1000000 {
                unsafe { core::arch::asm!("nop"); }
            }
        }
    }
}

// 初始化系统调用
fn init_syscalls() {
    print!("初始化系统调用...\n");
    // 初始化系统调用表
    syscall::init();
}

// 恐慌处理
#[cfg(all(not(feature = "test"), not(test)))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 打印恐慌信息
    print!("内核恐慌: {}\n", info);
    loop {
        // 无限循环
        unsafe { core::arch::asm!("hlt"); }
    }
}

// 测试入口点
#[cfg(feature = "test")]
extern "C" fn test_main(boot_info: *mut BootInfo) -> ! {
    // 初始化控制台
    console::init();
    
    // 打印测试启动信息
    print!("墨渊操作系统测试模式启动...\n");
    
    // 初始化内核
    init_kernel(boot_info);
    
    // 测试内存管理
    test_memory_management();
    
    // 测试进程管理
    test_process_management();
    
    // 测试系统调用
    test_syscalls();
    
    // 测试中断处理
    test_interrupts();
    
    // 测试完成
    print!("所有测试完成！\n");
    loop {
        // 无限循环
        unsafe { core::arch::asm!("hlt"); }
    }
}

// 测试内存管理
#[cfg(feature = "test")]
fn test_memory_management() {
    print!("测试内存管理...\n");
    // 测试物理内存分配
    let page = mm::physical::allocate_page();
    assert!(page.is_some(), "物理内存分配失败");
    print!("物理内存分配成功: {:#x}\n", page.unwrap());
    
    // 测试物理内存释放
    if let Some(addr) = page {
        mm::physical::free_page(addr);
        print!("物理内存释放成功: {:#x}\n", addr);
    }
}

// 测试进程管理
#[cfg(feature = "test")]
fn test_process_management() {
    print!("测试进程管理...\n");
    // 测试进程创建
    // 这里需要实现进程创建逻辑
}

// 测试系统调用
#[cfg(feature = "test")]
fn test_syscalls() {
    print!("测试系统调用...\n");
    // 测试系统调用处理
    let result = syscall::syscall::handle_syscall(0, 0, 0, 0, 0, 0, 0); // exit系统调用
    assert_eq!(result, 0, "系统调用处理失败");
    print!("系统调用测试成功\n");
}

// 测试中断处理
#[cfg(feature = "test")]
fn test_interrupts() {
    print!("测试中断处理...\n");
    // 测试中断处理函数注册
    // 这里需要实现中断处理函数注册测试
}
