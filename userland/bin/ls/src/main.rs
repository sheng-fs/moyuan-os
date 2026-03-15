#![no_std]
#![no_main]

extern crate alloc;

// 导入libc函数
extern "C" {
    fn exit(status: i32) -> !;
    fn printf(format: *const u8, ...) -> i32;
}

// 打印格式化字符串
macro_rules! println {
    ($($arg:tt)*) => {
        unsafe {
            printf(concat!($($arg)*, "\n").as_ptr() as *const u8);
        }
    };
}

// 主函数
#[no_mangle]
pub extern "C" fn main() -> ! {
    println!("Directory contents:");
    println!("  .");
    println!("  ..");
    println!("  bin");
    println!("  dev");
    println!("  etc");
    println!("  home");
    println!("  lib");
    println!("  proc");
    println!("  sys");
    println!("  tmp");
    
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


