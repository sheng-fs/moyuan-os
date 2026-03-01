#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi_services::println;

// 内核入口点类型
type KernelEntry = extern "C" fn(*mut BootInfo) -> !;

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

// 默认实现
impl Default for MemoryMapEntry {
    fn default() -> Self {
        Self {
            physical_start: 0,
            virtual_start: 0,
            number_of_pages: 0,
            memory_type: 0,
            attribute: 0,
        }
    }
}

impl Default for FramebufferInfo {
    fn default() -> Self {
        Self {
            address: 0,
            width: 0,
            height: 0,
            pitch: 0,
            bpp: 0,
        }
    }
}

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize UEFI services");

    println!("墨渊操作系统 UEFI 引导加载器");
    println!("正在初始化...");

    // 获取内存映射
    let mut memory_map_buffer = [0u8; 4096];
    let memory_map = system_table.boot_services().memory_map(&mut memory_map_buffer).expect("Failed to get memory map");
    println!("内存映射获取成功: {} 个条目", memory_map.entries().count());

    // 转换内存映射为我们的格式
    let mut memory_map_entries = [MemoryMapEntry::default(); 256];
    let mut entry_count = 0;
    for (i, entry) in memory_map.entries().enumerate() {
        if i >= 256 {
            break;
        }
        memory_map_entries[i] = MemoryMapEntry {
            physical_start: entry.phys_start,
            virtual_start: entry.virt_start,
            number_of_pages: entry.page_count,
            memory_type: 0, // 暂时设置为0
            attribute: entry.att.bits(),
        };
        entry_count += 1;
    }

    // 帧缓冲区信息（暂时使用默认值）
    let framebuffer_info = FramebufferInfo::default();
    println!("帧缓冲区信息: 宽={}, 高={}, 地址={:#x}", framebuffer_info.width, framebuffer_info.height, framebuffer_info.address);

    // 加载内核到固定地址 0x800000
    let (kernel_address, kernel_size) = load_kernel(&system_table, image_handle, 0x800000).expect("Failed to load kernel");
    println!("内核加载成功，地址: {:#x}, 大小: {} 字节", kernel_address, kernel_size);

    // 准备启动信息
    let boot_info = BootInfo {
        memory_map: memory_map_entries.as_ptr(),
        memory_map_count: entry_count,
        framebuffer: framebuffer_info,
        kernel_base: kernel_address,
        kernel_size: kernel_size,
    };

    // 跳转到内核
    println!("准备跳转到内核...");
    jump_to_kernel(kernel_address, &boot_info);

    Status::SUCCESS
}

/// 加载内核文件到指定地址
fn load_kernel(system_table: &SystemTable<Boot>, image_handle: Handle, target_address: u64) -> Result<(u64, u64), Status> {
    // 暂时返回一个模拟的内核地址和大小
    // 实际实现需要读取文件系统和加载内核文件
    Ok((target_address, 1024 * 1024)) // 假设内核大小为1MB
}

/// 跳转到内核
fn jump_to_kernel(kernel_address: u64, boot_info: &BootInfo) -> ! {
    // 确保引导服务不再被使用
    unsafe {
        // 获取内核入口点（假设内核入口点在文件开头）
        let kernel_entry = kernel_address as *const KernelEntry;
        let kernel_entry = core::ptr::read(kernel_entry);
        
        // 跳转到内核
        kernel_entry(boot_info as *const _ as *mut _);
    }
    
    // 无限循环，以防跳转失败
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
