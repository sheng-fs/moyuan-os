#![no_std]
#![no_main]

extern crate alloc;

// 导入libc函数
extern "C" {
    fn exit(status: i32) -> !;
    fn printf(format: *const u8, ...) -> i32;
    fn open(path: *const u8, flags: i32, mode: u32) -> i32;
    fn close(fd: i32) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

// 定义标准文件描述符
const STDOUT_FILENO: i32 = 1;

// 文件操作标志
const O_RDONLY: u32 = 0;





// 主函数
#[no_mangle]
pub extern "C" fn main() -> ! {
    // 这里应该解析命令行参数
    // 暂时硬编码测试
    let files = ["/test.txt"];
    
    for file in &files {
        unsafe {
            let fd = open(file.as_ptr(), O_RDONLY as i32, 0);
            if fd < 0 {
                printf(concat!("cat: {}: No such file or directory", "\n").as_ptr() as *const u8, file.as_ptr());
                continue;
            }
            
            let mut buf = [0u8; 1024];
            loop {
                let n = read(fd, buf.as_mut_ptr(), buf.len());
                if n <= 0 {
                    break;
                }
                write(STDOUT_FILENO, buf.as_ptr(), n as usize);
            }
            
            close(fd);
        }
    }
    
    unsafe {
        exit(0);
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


