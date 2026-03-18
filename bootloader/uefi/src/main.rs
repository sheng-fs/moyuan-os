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
    let target_address = 0x800000;
    let (kernel_address, kernel_size) = load_kernel(&system_table, image_handle, target_address).expect("Failed to load kernel");
    println!("内核加载成功，地址: {:#x}, 大小: {} 字节", kernel_address, kernel_size);
    
    // 确保内核加载到了预期的地址
    if kernel_address != target_address {
        println!("警告: 内核加载到了非预期地址，可能会导致执行错误");
    }

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
        Ok(fs) => {
            println!("文件系统获取成功");
            fs
        },
        Err(e) => {
            println!("文件系统获取失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    
    // 打开根目录
    let mut root_dir = match file_system.open_volume() {
        Ok(dir) => {
            println!("根目录打开成功");
            dir
        },
        Err(e) => {
            println!("根目录打开失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    
    // 尝试打开kernel目录
    println!("尝试打开kernel目录...");
    let mut kernel_dir = match root_dir.open(cstr16!("kernel"), FileMode::Read, FileAttribute::DIRECTORY) {
        Ok(dir) => {
            println!("kernel目录打开成功");
            dir
        },
        Err(e) => {
            println!("kernel目录打开失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    
    // 打开内核文件
    let kernel_path = cstr16!("kernel.elf");
    let mut kernel_file = match kernel_dir.open(kernel_path, FileMode::Read, FileAttribute::empty()) {
        Ok(file) => {
            println!("内核文件打开成功");
            file
        },
        Err(e) => {
            println!("内核文件打开失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    
    // 获取文件大小
    let mut file_info_buffer = [0u8; 1024];
    let file_info = match kernel_file.get_info::<FileInfo>(&mut file_info_buffer) {
        Ok(info) => info,
        Err(_) => return Err(Status::DEVICE_ERROR),
    };
    let kernel_size = file_info.file_size();
    
    // 尝试分配内存到目标地址，强制成功
    println!("尝试分配到0x800000地址...");
    let kernel_buffer = match boot_services.allocate_pages(
        AllocateType::Address(target_address / 4096),
        MemoryType::LOADER_DATA,
        ((kernel_size + 4095) / 4096) as usize,
    ) {
        Ok(buffer) => {
            println!("成功分配到目标地址: {:#x}", buffer);
            buffer
        },
        Err(e) => {
            println!("无法分配到目标地址 {:#x}，错误: {:?}", target_address, e);
            println!("使用任意地址...");
            match boot_services.allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                ((kernel_size + 4095) / 4096) as usize,
            ) {
                Ok(buffer) => buffer,
                Err(_) => return Err(Status::OUT_OF_RESOURCES),
            }
        }
    };
    
    // 读取内核文件到内存
    let buffer = unsafe { core::slice::from_raw_parts_mut(kernel_buffer as *mut u8, kernel_size as usize) };
    let mut file = kernel_file.into_regular_file().expect("Failed to convert to regular file");
    match file.read(buffer) {
        Ok(size) => if size != kernel_size as usize {
            return Err(Status::DEVICE_ERROR);
        },
        Err(_) => return Err(Status::DEVICE_ERROR),
    }
    
    Ok((kernel_buffer, kernel_size))
}

/// ELF头部结构
#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],      // ELF标识
    e_type: u16,             // 对象文件类型
    e_machine: u16,          // 机器类型
    e_version: u32,          // 对象文件版本
    e_entry: u64,            // 入口点地址
    e_phoff: u64,            // 程序头表偏移
    e_shoff: u64,            // 节头表偏移
    e_flags: u32,             // 处理器特定标志
    e_ehsize: u16,           // ELF头部大小
    e_phentsize: u16,        // 程序头表项大小
    e_phnum: u16,             // 程序头表项数量
    e_shentsize: u16,        // 节头表项大小
    e_shnum: u16,             // 节头表项数量
    e_shstrndx: u16,          // 节头字符串表索引
}

/// 跳转到内核
fn jump_to_kernel(kernel_address: u64, boot_info: &BootInfo) -> ! {
    // 确保引导服务不再被使用
    unsafe {
        // 解析ELF文件的入口点
        let elf_header = kernel_address as *const Elf64Ehdr;
        let entry_point = (*elf_header).e_entry;
        
        let actual_entry = if kernel_address == 0x800000 {
            // 如果内核加载到了预期地址，直接使用ELF入口点
            entry_point
        } else {
            // 否则，计算相对于加载地址的偏移
            kernel_address + (entry_point - 0x800000)
        };
        
        println!("内核加载地址: {:#x}", kernel_address);
        println!("ELF入口点地址: {:#x}", entry_point);
        println!("实际入口点地址: {:#x}", actual_entry);
        
        // 跳转到内核入口点
        let kernel_entry: KernelEntry = core::mem::transmute(actual_entry as *const ());
        kernel_entry(boot_info as *const _ as *mut _);
    }
}
