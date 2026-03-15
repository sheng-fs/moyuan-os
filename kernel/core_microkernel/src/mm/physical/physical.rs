// 物理内存管理

use crate::BootInfo;
use crate::console;

// 物理内存页大小
pub const PAGE_SIZE: usize = 4096;

// 空闲块结构
struct FreeBlock {
    start_frame: usize,
    size: usize, // 以页为单位
    next: Option<*mut FreeBlock>,
}

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
    // 空闲块链表
    free_list: Option<*mut FreeBlock>,
}

// 全局物理内存管理器
static mut PHYSICAL_MEMORY_MANAGER: Option<PhysicalMemoryManager> = None;

// 初始化物理内存管理
pub unsafe fn init(boot_info: *mut BootInfo) {
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
        let total_frames = (max_physical_address as usize).div_ceil(PAGE_SIZE);
        let bitmap_size = total_frames.div_ceil(8);
        
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
        let end_frame = kernel_end.div_ceil(PAGE_SIZE as u64) as usize;
        
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
        let bitmap_end_frame = bitmap_end.div_ceil(PAGE_SIZE as u64) as usize;
        
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
            free_list: None,
        });
        
        // 初始化空闲块链表
        build_free_list();
}

// 构建空闲块链表
fn build_free_list() {
    unsafe {
        if let Some(ref mut manager) = PHYSICAL_MEMORY_MANAGER {
            let mut current_start = None;
            let mut current_size = 0;
            
            // 遍历所有页帧
            for frame in 0..manager.total_frames {
                let byte_index = frame / 8;
                let bit_index = frame % 8;
                
                if byte_index < manager.bitmap.len() {
                    let is_free = (manager.bitmap[byte_index] & (1 << bit_index)) == 0;
                    
                    if is_free {
                        // 当前页帧是空闲的
                        if current_start.is_none() {
                            current_start = Some(frame);
                        }
                        current_size += 1;
                    } else {
                        // 当前页帧已分配，结束当前空闲块
                        if current_start.is_some() {
                            if let Err(_) = add_free_block(current_start.unwrap(), current_size) {
                                // 记录错误但继续执行
                                crate::console::print(core::format_args!("警告: 添加空闲块失败\n"));
                            }
                            current_start = None;
                            current_size = 0;
                        }
                    }
                }
            }
            
            // 处理最后一个空闲块
            if current_start.is_some() {
                if let Err(_) = add_free_block(current_start.unwrap(), current_size) {
                    // 记录错误但继续执行
                    crate::console::print(core::format_args!("警告: 添加最后一个空闲块失败\n"));
                }
            }
        }
    }
}

// 添加空闲块到链表
fn add_free_block(start_frame: usize, size: usize) -> Result<(), AllocError> {
    unsafe {
        if let Some(ref mut manager) = PHYSICAL_MEMORY_MANAGER {
            // 分配空闲块结构
            match allocate_page() {
                Ok(block_addr) => {
                    let block = block_addr as *mut FreeBlock;
                    (*block).start_frame = start_frame;
                    (*block).size = size;
                    (*block).next = manager.free_list;
                    
                    // 添加到链表头部
                    manager.free_list = Some(block);
                    Ok(())
                },
                Err(err) => Err(err),
            }
        } else {
            Err(AllocError::InternalError)
        }
    }
}

// 内存分配错误类型
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AllocError {
    OutOfMemory,
    InvalidAddress,
    InternalError,
}

// 分配物理内存页
pub fn allocate_page() -> Result<usize, AllocError> {
    unsafe {
        if let Some(ref mut manager) = PHYSICAL_MEMORY_MANAGER {
            // 使用空闲块链表查找可用页帧
            let mut prev: Option<*mut FreeBlock> = None;
            let mut current = manager.free_list;
            
            while let Some(block) = current {
                if (*block).size >= 1 {
                    // 找到足够大的空闲块
                    let frame = (*block).start_frame;
                    
                    // 验证内存区域大小
                    if frame >= manager.total_frames {
                        return Err(AllocError::InvalidAddress);
                    }
                    
                    // 标记为已分配
                    let byte_index = frame / 8;
                    let bit_index = frame % 8;
                    if byte_index < manager.bitmap.len() {
                        manager.bitmap[byte_index] |= 1 << bit_index;
                        manager.allocated_frames += 1;
                    } else {
                        return Err(AllocError::InternalError);
                    }
                    
                    // 更新空闲块
                    (*block).start_frame += 1;
                    (*block).size -= 1;
                    
                    // 如果块大小为0，从链表中移除
                    if (*block).size == 0 {
                        if let Some(prev_block) = prev {
                            (*prev_block).next = (*block).next;
                        } else {
                            manager.free_list = (*block).next;
                        }
                        // 释放空闲块结构
                        if let Err(_) = free_page(block as usize) {
                            // 记录错误但继续执行
                            crate::console::print(core::format_args!("警告: 释放空闲块结构失败\n"));
                        }
                    }
                    
                    // 计算物理地址
                    let physical_address = frame * PAGE_SIZE;
                    return Ok(physical_address);
                }
                
                prev = current;
                current = (*block).next;
            }
            return Err(AllocError::OutOfMemory);
        }
        Err(AllocError::InternalError)
    }
}

// 释放物理内存页
pub fn free_page(physical_address: usize) -> Result<(), AllocError> {
    unsafe {
        if let Some(ref mut manager) = PHYSICAL_MEMORY_MANAGER {
            // 检查物理地址是否页对齐
            if !physical_address.is_multiple_of(PAGE_SIZE) {
                return Err(AllocError::InvalidAddress); // 物理地址不是页对齐的
            }
            
            // 计算页帧号
            let frame = physical_address / PAGE_SIZE;
            if frame >= manager.total_frames {
                return Err(AllocError::InvalidAddress);
            }
            
            // 标记为未分配
            let byte_index = frame / 8;
            let bit_index = frame % 8;
            if byte_index < manager.bitmap.len() {
                manager.bitmap[byte_index] &= !(1 << bit_index);
                manager.allocated_frames -= 1;
                
                // 添加到空闲块链表
                let _ = add_free_block(frame, 1);
                return Ok(());
            } else {
                return Err(AllocError::InternalError);
            }
        }
        Err(AllocError::InternalError)
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
