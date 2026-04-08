#![no_std]
#![no_main]

// 导入主内核模块
use moyuan_core_microkernel::BootInfo;

// 测试入口点
#[no_mangle]
extern "C" fn test_main(boot_info: *mut BootInfo) -> ! {
    // 调用主内核的测试入口点
    moyuan_core_microkernel::test_main(boot_info);
}


