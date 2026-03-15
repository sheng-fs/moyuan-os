#![no_std]
#![no_main]

extern crate alloc;

// 导入libc函数
extern "C" {
    fn exit(status: i32) -> !;
    fn printf(format: *const u8, ...) -> i32;
    fn unlink(path: *const u8) -> i32;
}



// 主函数
#[no_mangle]
pub extern "C" fn main() -> ! {
    // 这里应该解析命令行参数
    // 暂时硬编码测试
    let files = ["/test.txt"];
    
    for file in &files {
        unsafe {
            let result = unlink(file.as_ptr());
            if result < 0 {
                printf(concat!("rm: {}: No such file or directory", "\n").as_ptr() as *const u8, file.as_ptr());
            } else {
                printf(concat!("rm: removed '{}'", "\n").as_ptr() as *const u8, file.as_ptr());
            }
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


