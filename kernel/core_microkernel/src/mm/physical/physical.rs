// 物理内存管理

use crate::BootInfo;
use crate::console;

// 物理内存页大小
pub const PAGE_SIZE: usize = 4096;

// 物理内存管理器
struct PhysicalMemoryManager {
    // 位图用于跟踪页帧分配
    bitmap: &'static mut [u8],
    // 页帧总数
    total_frames: usize,
    // 已分配的页帧数量
    allocated_frames: usize,
    // 物理内存起始地址
    _physical_memory_start: u64,
}

// 全局物理内存管理器
static mut PHYSICAL_MEMORY_MANAGER: Option<PhysicalMemoryManager> = None;

// 初始化物理内存管理
pub fn init(boot_info: *mut BootInfo) {
    unsafe {
        // 解析内存映射
        let memory_map = (*boot_info).memory_map;
        let memory_map_count = (*boot_info).memory_map_count;
        
        // 计算物理内存总量
        let mut total_memory = 0;
        let mut max_physical_address = 0;
        
        for i in 0..memory_map_count {
            let entry = *memory_map.offset(i as isize);
            let memory_size = entry.number_of_pages * PAGE_SIZE as u64;
            total_memory += memory_size;
            let end_address = entry.physical_start + memory_size;
            if end_address > max_physical_address {
                max_physical_address = end_address;
            }
        }
        
        console::print(core::format_args!("物理内存总量: {} MB\n", total_memory / (1024 * 1024)));
        
        // 计算需要的位图大小
        let total_frames = (max_physical_address as usize + PAGE_SIZE - 1) / PAGE_SIZE;
        let bitmap_size = (total_frames + 7) / 8;
        
        // 寻找可用的内存区域来存储位图
        let mut bitmap_address = 0;
        for i in 0..memory_map_count {
            let entry = *memory_map.offset(i as isize);
            // 寻找可用的内存区域
            if entry.memory_type == 7 { // EFI_CONVENTIONAL_MEMORY
                let start = entry.physical_start;
                let size = entry.number_of_pages * PAGE_SIZE as u64;
                if size >= bitmap_size as u64 {
                    bitmap_address = start;
                    break;
                }
            }
        }
        
        if bitmap_address == 0 {
            console::print(core::format_args!("无法找到足够的内存来存储位图\n"));
            return;
        }
        
        console::print(core::format_args!("位图地址: {:#x}, 大小: {} 字节\n", bitmap_address, bitmap_size));
        
        // 初始化位图
        let bitmap = core::slice::from_raw_parts_mut(bitmap_address as *mut u8, bitmap_size);
        for byte in bitmap.iter_mut() {
            *byte = 0;
        }
        
        // 标记保留内存区域
        for i in 0..memory_map_count {
            let entry = *memory_map.offset(i as isize);
            if entry.memory_type != 7 { // 不是常规内存
                let start_frame = (entry.physical_start / PAGE_SIZE as u64) as usize;
                let end_frame = start_frame + entry.number_of_pages as usize;
                for frame in start_frame..end_frame {
                    if frame < total_frames {
                        let byte_index = frame / 8;
                        let bit_index = frame % 8;
                        if byte_index < bitmap_size {
                            bitmap[byte_index] |= 1 << bit_index;
                        }
                    }
                }
            }
        }
        
        // 标记内核占用的内存
        let kernel_start = (*boot_info).kernel_base;
        let kernel_size = (*boot_info).kernel_size;
        let kernel_end = kernel_start + kernel_size;
        let start_frame = (kernel_start / PAGE_SIZE as u64) as usize;
        let end_frame = ((kernel_end + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64) as usize;
        
        for frame in start_frame..end_frame {
            if frame < total_frames {
                let byte_index = frame / 8;
                let bit_index = frame % 8;
                if byte_index < bitmap_size {
                    bitmap[byte_index] |= 1 << bit_index;
                }
            }
        }
        
        // 标记位图本身占用的内存
        let bitmap_end = bitmap_address + bitmap_size as u64;
        let bitmap_start_frame = (bitmap_address / PAGE_SIZE as u64) as usize;
        let bitmap_end_frame = ((bitmap_end + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64) as usize;
        
        for frame in bitmap_start_frame..bitmap_end_frame {
            if frame < total_frames {
                let byte_index = frame / 8;
                let bit_index = frame % 8;
                if byte_index < bitmap_size {
                    bitmap[byte_index] |= 1 << bit_index;
                }
            }
        }
        
        // 计算可用内存
        let mut free_frames = 0;
        for byte in bitmap.iter() {
            free_frames += byte.count_zeros() as usize;
        }
        let free_memory = free_frames * PAGE_SIZE;
        console::print(core::format_args!("可用物理内存: {} MB\n", free_memory / (1024 * 1024)));
        
        // 创建物理内存管理器
        PHYSICAL_MEMORY_MANAGER = Some(PhysicalMemoryManager {
            bitmap,
            total_frames,
            allocated_frames: 0,
            _physical_memory_start: 0,
        });
    }
}

// 分配物理内存页
pub fn allocate_page() -> Option<usize> {
    unsafe {
        if let Some(ref mut manager) = PHYSICAL_MEMORY_MANAGER {
            // 查找第一个可用的页帧
            for (byte_index, &byte) in manager.bitmap.iter().enumerate() {
                if byte != 0xFF {
                    // 找到一个有可用位的字节
                    for bit_index in 0..8 {
                        if (byte & (1 << bit_index)) == 0 {
                            // 找到可用页帧
                            let frame = byte_index * 8 + bit_index;
                            if frame < manager.total_frames {
                                // 标记为已分配
                                manager.bitmap[byte_index] |= 1 << bit_index;
                                manager.allocated_frames += 1;
                                
                                // 计算物理地址
                                let physical_address = frame * PAGE_SIZE;
                                return Some(physical_address);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

// 释放物理内存页
pub fn free_page(physical_address: usize) {
    unsafe {
        if let Some(ref mut manager) = PHYSICAL_MEMORY_MANAGER {
            // 计算页帧号
            let frame = physical_address / PAGE_SIZE;
            if frame < manager.total_frames {
                // 标记为未分配
                let byte_index = frame / 8;
                let bit_index = frame % 8;
                if byte_index < manager.bitmap.len() {
                    manager.bitmap[byte_index] &= !(1 << bit_index);
                    manager.allocated_frames -= 1;
                }
            }
        }
    }
}

// 获取物理内存总量
#[allow(dead_code)]
pub fn get_total_memory() -> usize {
    unsafe {
        if let Some(ref manager) = PHYSICAL_MEMORY_MANAGER {
            manager.total_frames * PAGE_SIZE
        } else {
            0
        }
    }
}

// 获取已分配内存
#[allow(dead_code)]
pub fn get_allocated_memory() -> usize {
    unsafe {
        if let Some(ref manager) = PHYSICAL_MEMORY_MANAGER {
            manager.allocated_frames * PAGE_SIZE
        } else {
            0
        }
    }
}
