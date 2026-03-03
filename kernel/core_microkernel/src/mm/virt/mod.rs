pub mod address_space;
use crate::mm::virt::address_space::{PAGE_FLAG_PRESENT, PAGE_FLAG_WRITABLE, PAGE_FLAG_GLOBAL};

// 全局地址空间
#[allow(static_mut_refs)]
static mut KERNEL_ADDRESS_SPACE: Option<address_space::AddressSpace> = None;

// 初始化虚拟内存管理
pub fn init() {
    unsafe {
        // 创建内核地址空间
        let mut address_space = address_space::AddressSpace::new();
        address_space.init();
        
        // 映射内核代码和数据
        // 这里需要根据实际的内核布局进行映射
        // 暂时映射物理内存的前1GB
        for i in 0..(1024 * 1024 * 1024 / 4096) {
            let physical_addr = i * 4096;
            let virtual_addr = physical_addr as u64;
            address_space.map(virtual_addr, physical_addr as u64, PAGE_FLAG_PRESENT | PAGE_FLAG_WRITABLE | PAGE_FLAG_GLOBAL);
        }
        
        // 激活地址空间
        address_space.activate();
        
        KERNEL_ADDRESS_SPACE = Some(address_space);
        crate::console::print(core::format_args!("虚拟内存管理初始化成功\n"));
    }
}

// 获取内核地址空间
#[allow(dead_code)]
#[allow(static_mut_refs)]
pub fn get_kernel_address_space() -> Option<&'static mut address_space::AddressSpace> {
    unsafe {
        KERNEL_ADDRESS_SPACE.as_mut()
    }
}

pub use address_space::AddressSpace;
