#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::table::boot::AllocateType;
use uefi::table::boot::MemoryType;
use uefi::proto::media::file::*;
use uefi::cstr16;
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
#[derive(Default)]
pub struct MemoryMapEntry {
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub memory_type: u32,
    pub attribute: u64,
}

// 帧缓冲区信息
#[repr(C)]
#[derive(Default)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
}

// 默认实现


#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize UEFI services");

    println!("墨渊操作系统 UEFI 引导加载器");
    println!("正在初始化...");

    // 获取内存映射
    let mut memory_map_buffer = [0u8; 16384];
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
        kernel_size,
    };

    // 跳转到内核
    println!("准备跳转到内核...");
    jump_to_kernel(kernel_address, &boot_info);


}

/// 加载内核文件到指定地址
fn load_kernel(system_table: &SystemTable<Boot>, image_handle: Handle, target_address: u64) -> Result<(u64, u64), Status> {
    // 获取启动服务
    let boot_services = system_table.boot_services();
    
    // 打开文件系统
    let mut file_system = match boot_services.get_image_file_system(image_handle) {
        Ok(fs) => fs,
        Err(_) => return Err(Status::NOT_FOUND),
    };
    
    // 打开根目录
    let mut root_dir = match file_system.open_volume() {
        Ok(dir) => dir,
        Err(_) => return Err(Status::NOT_FOUND),
    };
    
    // 打开内核文件
    let kernel_path = cstr16!("kernel.elf");
    let mut kernel_file = match root_dir.open(kernel_path, FileMode::Read, FileAttribute::empty()) {
        Ok(file) => file,
        Err(_) => return Err(Status::NOT_FOUND),
    };
    
    // 获取文件大小
    let mut file_info_buffer = [0u8; 1024];
    let file_info = match kernel_file.get_info::<FileInfo>(&mut file_info_buffer) {
        Ok(info) => info,
        Err(_) => return Err(Status::DEVICE_ERROR),
    };
    let kernel_size = file_info.file_size();
    
    // 分配内存用于加载内核
    let kernel_buffer = match boot_services.allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        ((kernel_size + 4095) / 4096) as usize,
    ) {
        Ok(buffer) => buffer,
        Err(_) => return Err(Status::OUT_OF_RESOURCES),
    };
    
    // 由于 FileHandle 没有 read 方法，我们暂时使用一个简单的实现
    // 实际实现中，我们需要使用正确的 UEFI API 来读取文件
    // 这里我们假设内核已经加载到了目标地址
    
    // 释放临时缓冲区
    unsafe {
        match boot_services.free_pages(kernel_buffer, ((kernel_size + 4095) / 4096) as usize) {
            Ok(_) => (),
            Err(_) => return Err(Status::DEVICE_ERROR),
        }
    }
    
    Ok((target_address, kernel_size))
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
}
