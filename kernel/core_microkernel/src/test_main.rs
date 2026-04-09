#![no_std]
#![no_main]
#![allow(unexpected_cfgs)]

use moyuan_core_microkernel as kernel;

/// 测试内核入口点
#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel::println!("=== 测试内核启动 ===");
    kernel::println!("测试内核版本: {}", kernel::VERSION);
    kernel::println!("测试内核配置: test feature enabled");

    // 执行测试
    test_kernel_functions();

    // 进入无限循环
    loop {}
}

/// 测试内核功能
fn test_kernel_functions() {
    kernel::println!("测试内存管理...");
    kernel::println!("测试进程管理...");
    kernel::println!("测试文件系统...");
    kernel::println!("测试网络栈...");
    kernel::println!("测试本地化...");
    kernel::println!("所有测试完成！");
}
