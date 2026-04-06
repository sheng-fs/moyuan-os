#![no_main]
#![no_std]

use uefi::prelude::*;
use uefi::table::boot::AllocateType;
use uefi::table::boot::MemoryType;
use uefi::proto::media::file::*;
use uefi::cstr16;
use uefi_services::println;

type KernelEntry = extern "C" fn(*mut BootInfo) -> !;

#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const MemoryMapEntry,
    pub memory_map_count: u32,
    pub framebuffer: FramebufferInfo,
    pub kernel_base: u64,
    pub kernel_size: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct MemoryMapEntry {
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub memory_type: u32,
    pub attribute: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
}

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize UEFI services");

    println!("墨渊操作系统 UEFI 引导加载器");
    println!("正在初始化...");

    let mut memory_map_buffer = [0u8; 16384];
    let memory_map = system_table.boot_services().memory_map(&mut memory_map_buffer).expect("Failed to get memory map");
    println!("内存映射获取成功: {} 个条目", memory_map.entries().count());

    let framebuffer_info = FramebufferInfo::default();
    println!("帧缓冲区信息: 宽={}, 高={}, 地址={:#x}", framebuffer_info.width, framebuffer_info.height, framebuffer_info.address);

    let (kernel_base, kernel_size, entry_point) = load_kernel(&system_table, image_handle).expect("Failed to load kernel");
    println!("内核加载成功，基地址: {:#x}, 大小: {} 字节, 入口: {:#x}", kernel_base, kernel_size, entry_point);

    // 现在执行所有内存分配
    let boot_services = system_table.boot_services();
    
    // 分配持久内存来存储内存映射
    let memory_map_ptr = boot_services.allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1, // 1页足够存储256个条目
    ).expect("Failed to allocate memory for memory map") as *mut MemoryMapEntry;
    
    let memory_map_entries = unsafe { core::slice::from_raw_parts_mut(memory_map_ptr, 256) };
    let mut entry_count = 0;
    for (i, entry) in memory_map.entries().enumerate() {
        if i >= 256 {
            break;
        }
        memory_map_entries[i] = MemoryMapEntry {
            physical_start: entry.phys_start,
            virtual_start: entry.virt_start,
            number_of_pages: entry.page_count,
            memory_type: 0,
            attribute: entry.att.bits(),
        };
        entry_count += 1;
    }

    // 分配持久内存来存储BootInfo
    let boot_info_ptr = boot_services.allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
    ).expect("Failed to allocate memory for BootInfo") as *mut BootInfo;
    
    unsafe {
        *boot_info_ptr = BootInfo {
            memory_map: memory_map_ptr,
            memory_map_count: entry_count,
            framebuffer: framebuffer_info,
            kernel_base,
            kernel_size,
        };
    }

    // 现在直接跳转到内核，不再调用任何UEFI函数
    jump_to_kernel(entry_point, boot_info_ptr);
}

fn load_kernel(system_table: &SystemTable<Boot>, image_handle: Handle) -> Result<(u64, u64, u64), Status> {
    let boot_services = system_table.boot_services();
    let mut file_system = match boot_services.get_image_file_system(image_handle) {
        Ok(fs) => fs,
        Err(e) => {
            println!("文件系统获取失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    let mut root_dir = match file_system.open_volume() {
        Ok(dir) => dir,
        Err(e) => {
            println!("根目录打开失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    println!("尝试打开kernel.elf文件...");
    let kernel_path = cstr16!("kernel.elf");
    let mut kernel_file = match root_dir.open(kernel_path, FileMode::Read, FileAttribute::empty()) {
        Ok(file) => file,
        Err(e) => {
            println!("内核文件打开失败: {:?}", e);
            return Err(Status::NOT_FOUND);
        },
    };
    let mut file_info_buffer = [0u8; 1024];
    let file_info = match kernel_file.get_info::<FileInfo>(&mut file_info_buffer) {
        Ok(info) => info,
        Err(_) => return Err(Status::DEVICE_ERROR),
    };
    let file_size = file_info.file_size();
    let temp_buffer = match boot_services.allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        ((file_size + 4095) / 4096) as usize,
    ) {
        Ok(buffer) => buffer,
        Err(_) => return Err(Status::OUT_OF_RESOURCES),
    };
    let buffer = unsafe { core::slice::from_raw_parts_mut(temp_buffer as *mut u8, file_size as usize) };
    let mut file = kernel_file.into_regular_file().expect("Failed to convert to regular file");
    match file.read(buffer) {
        Ok(size) => if size != file_size as usize {
            return Err(Status::DEVICE_ERROR);
        },
        Err(_) => return Err(Status::DEVICE_ERROR),
    };
    let elf_header = temp_buffer as *const Elf64Ehdr;
    let entry_point = unsafe { (*elf_header).e_entry };
    let phoff = unsafe { (*elf_header).e_phoff };
    let phnum = unsafe { (*elf_header).e_phnum } as usize;
    let phentsize = unsafe { (*elf_header).e_phentsize } as usize;
    let mut kernel_base = 0;
    let mut kernel_size = 0;
    for i in 0..phnum {
        let phdr_ptr = (temp_buffer + phoff) as *const u8;
        let phdr = unsafe { &*((phdr_ptr.add(i * phentsize)) as *const Elf64Phdr) };
        if phdr.p_type == 1 {
            let vaddr = phdr.p_vaddr;
            let memsz = phdr.p_memsz;
            let filesz = phdr.p_filesz;
            let offset = phdr.p_offset;
            if kernel_base == 0 {
                kernel_base = vaddr;
            }
            let end_addr = vaddr + memsz;
            if end_addr > kernel_base + kernel_size {
                kernel_size = end_addr - kernel_base;
            }
            println!("加载段: vaddr={:#x}, filesz={:#x}, memsz={:#x}", vaddr, filesz, memsz);
            let pages_needed = ((vaddr + memsz + 4095) / 4096) - (vaddr / 4096);
            let target_page = vaddr / 4096;
            let _ = boot_services.allocate_pages(
                AllocateType::Address(target_page),
                MemoryType::LOADER_DATA,
                pages_needed as usize,
            );
            let dest_ptr = vaddr as *mut u8;
            let src_ptr = (temp_buffer + offset) as *const u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, filesz as usize);
                if memsz > filesz {
                    let zero_start = dest_ptr.add(filesz as usize);
                    let zero_count = (memsz - filesz) as usize;
                    core::ptr::write_bytes(zero_start, 0, zero_count);
                }
            }
        }
    }
    Ok((kernel_base, kernel_size, entry_point))
}

fn jump_to_kernel(entry_point: u64, boot_info: *mut BootInfo) -> ! {
    unsafe {
        println!("跳转到内核入口点: {:#x}", entry_point);
        let kernel_entry: KernelEntry = core::mem::transmute(entry_point as *const ());
        kernel_entry(boot_info);
    }
}