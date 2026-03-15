#![no_std]
#![no_main]

extern crate alloc;

// 导入libc函数
extern "C" {
    fn exit(status: i32) -> !;
    fn printf(format: *const u8, ...) -> i32;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

// 定义标准文件描述符
const STDOUT_FILENO: i32 = 1;

// 打印字符串
fn print_str(s: &str) {
    unsafe {
        write(STDOUT_FILENO, s.as_ptr(), s.len());
    }
}

// 打印格式化字符串
macro_rules! println {
    ($fmt:expr) => {
        unsafe {
            printf(concat!($fmt, "\n").as_ptr() as *const u8);
        }
    };
    ($fmt:expr, $($arg:expr),*) => {
        unsafe {
            printf(concat!($fmt, "\n").as_ptr() as *const u8, $($arg),*);
        }
    };
}

// 启动服务
fn start_service(name: &str) -> bool {
    println!("Starting {}", name);
    // 这里应该实现服务启动逻辑
    // 暂时返回成功
    true
}

// 创建文件系统层次结构
fn setup_filesystem() -> bool {
    println!("Setting up filesystem...");
    
    // 创建基本目录结构
    let directories = [
        "/bin",
        "/dev",
        "/etc",
        "/home",
        "/lib",
        "/proc",
        "/sys",
        "/tmp"
    ];
    
    for dir in &directories {
        println!("Creating directory: {}", dir);
        // 这里应该调用mkdir系统调用
    }
    
    true
}

// 启动Shell进程
fn start_shell() -> bool {
    println!("Starting shell...");
    // 这里应该实现Shell进程启动逻辑
    // 暂时返回成功
    true
}

// 主函数
#[no_mangle]
pub extern "C" fn main() -> ! {
    print_str("\n=== MOYUAN OS INIT PROCESS ===\n\n");
    
    // 1. 初始化文件系统
    if !setup_filesystem() {
        println!("Error: Failed to setup filesystem");
        unsafe { exit(1); }
    }
    
    // 2. 启动核心服务
    let services = [
        "device service",
        "filesystem service"
    ];
    
    for service in &services {
        if !start_service(service) {
            println!("Error: Failed to start {}", service);
            unsafe { exit(1); }
        }
    }
    
    // 3. 启动Shell进程
    if !start_shell() {
        println!("Error: Failed to start shell");
        unsafe { exit(1); }
    }
    
    // 4. 监控Shell进程
    println!("Init process entering idle state");
    
    // 进入无限循环，等待Shell进程结束
    loop {
        // 这里应该实现进程监控逻辑
    }
}



// 分配器
use alloc::alloc::GlobalAlloc;
use core::ptr::null_mut;

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
    }
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

// panic 处理函数
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
