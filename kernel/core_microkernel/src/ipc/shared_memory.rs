// 共享内存模块

extern crate alloc;

use alloc::{vec::Vec, sync::Arc, string::String};
use spin::Mutex;

// 导入IPC错误类型和结果类型
use super::IpcResult;

const MAX_SHARED_MEMORY_SIZE: usize = 1024 * 1024; // 1MB最大共享内存

// 共享内存段结构
#[allow(dead_code)]
struct SharedMemorySegment {
    physical_address: usize,  // 物理地址
    size: usize,              // 大小
    ref_count: usize,         // 引用计数
    name: Option<String>,     // 名称（用于命名共享内存）
    is_allocated: bool,       // 是否已分配
}

impl SharedMemorySegment {
    fn new(physical_address: usize, size: usize, name: Option<String>) -> Self {
        Self {
            physical_address,
            size,
            ref_count: 1,
            name,
            is_allocated: true,
        }
    }

    fn increment_ref_count(&mut self) {
        self.ref_count += 1;
    }

    fn decrement_ref_count(&mut self) -> usize {
        self.ref_count = self.ref_count.saturating_sub(1);
        self.ref_count
    }

    #[allow(dead_code)]
    fn is_referenced(&self) -> bool {
        self.ref_count > 0
    }
}

// 共享内存表
static mut SHARED_MEMORY_SEGMENTS: Vec<Option<Arc<Mutex<SharedMemorySegment>>>> = Vec::new();
static mut NEXT_SHM_ID: usize = 0;

// 初始化共享内存模块
pub fn init() {
    unsafe {
        let segments = &raw mut SHARED_MEMORY_SEGMENTS;
        (*segments).reserve(64);
    }
}

// 创建共享内存
pub fn create_shared_memory(size: usize, name: Option<&str>) -> IpcResult<usize> {
    if size == 0 || size > MAX_SHARED_MEMORY_SIZE {
        return Err(super::IpcError::InvalidArgument);
    }

    // 分配物理内存
    let pages = size.div_ceil(4096);
    let mut physical_address = 0;
    
    // 分配多个页面
    for i in 0..pages {
        match crate::mm::physical::allocate_page() {
            Ok(addr) => {
                if i == 0 {
                    physical_address = addr;
                }
            },
            Err(_) => {
                // 释放已分配的页面
                if physical_address != 0 {
                    for j in 0..i {
                        let _ = crate::mm::physical::free_page(physical_address + j * 4096);
                    }
                }
                return Err(super::IpcError::ResourceBusy);
            }
        }
    }

    unsafe {
        let next_shm_id = &raw mut NEXT_SHM_ID;
        let shm_id = *next_shm_id;
        if shm_id >= 64 {
            // 释放已分配的页面
            for i in 0..pages {
                let _ = crate::mm::physical::free_page(physical_address + i * 4096);
            }
            return Err(super::IpcError::ResourceBusy);
        }
        *next_shm_id += 1;

        // 检查命名共享内存是否已存在
        if let Some(name_str) = name {
            let segments = &raw const SHARED_MEMORY_SEGMENTS;
            for segment in &(*segments) {
                if let Some(segment) = segment {
                    let segment = segment.lock();
                    if let Some(segment_name) = &segment.name {
                        if segment_name == name_str {
                            // 释放已分配的页面
                            for i in 0..pages {
                                let _ = crate::mm::physical::free_page(physical_address + i * 4096);
                            }
                            return Err(super::IpcError::ResourceBusy);
                        }
                    }
                }
            }
        }

        let segment = Arc::new(Mutex::new(SharedMemorySegment::new(
            physical_address,
            size,
            name.map(|s| String::from(s)),
        )));

        let segments = &raw mut SHARED_MEMORY_SEGMENTS;
        (*segments).push(Some(segment));
        Ok(shm_id)
    }
}

// 打开共享内存
pub fn open_shared_memory(name: &str) -> IpcResult<usize> {
    unsafe {
        let segments = &raw const SHARED_MEMORY_SEGMENTS;
        for (i, segment) in (*segments).iter().enumerate() {
            if let Some(segment) = segment {
                let mut segment = segment.lock();
                if let Some(segment_name) = &segment.name {
                    if segment_name == name {
                        segment.increment_ref_count();
                        return Ok(i);
                    }
                }
            }
        }
        Err(super::IpcError::NotFound)
    }
}

// 映射共享内存到进程地址空间
pub fn map_shared_memory(shm_id: usize, process_pid: usize) -> IpcResult<usize> {
    unsafe {
        let segments = &raw const SHARED_MEMORY_SEGMENTS;
        if shm_id >= (*segments).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let segment = (&(*segments))[shm_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let segment = segment.lock();

        if !segment.is_allocated {
            return Err(super::IpcError::InvalidArgument);
        }

        // 获取进程
        let process = crate::task::process::get_process(process_pid).ok_or(super::IpcError::InvalidArgument)?;

        // 分配虚拟地址（使用固定范围 0x700000000000 开始）
        let virtual_address = 0x700000000000 + (shm_id * 0x1000000) as u64; // 每个共享内存段1MB

        // 映射物理内存到虚拟地址
        let pages = segment.size.div_ceil(4096);
        for i in 0..pages {
            let phys_page = segment.physical_address + i * 4096;
            let virt_page = virtual_address as usize + i * 4096;
            // 使用 map 方法，设置可读写标志
            process.address_space.map(virt_page as u64, phys_page as u64, 0x3); // 0x3 = PRESENT | WRITABLE
        }

        Ok(virtual_address as usize)
    }
}

// 解除共享内存映射
pub fn unmap_shared_memory(shm_id: usize, process_pid: usize, virtual_address: usize) -> IpcResult<()> {
    unsafe {
        let segments = &raw const SHARED_MEMORY_SEGMENTS;
        if shm_id >= (*segments).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let segment = (&(*segments))[shm_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let segment = segment.lock();

        // 获取进程
        let process = crate::task::process::get_process(process_pid).ok_or(super::IpcError::InvalidArgument)?;

        // 解除映射
        let pages = segment.size.div_ceil(4096);
        for i in 0..pages {
            let virt_page = virtual_address + i * 4096;
            process.address_space.unmap(virt_page as u64);
        }

        Ok(())
    }
}

// 关闭共享内存
pub fn close_shared_memory(shm_id: usize) -> IpcResult<()> {
    unsafe {
        let segments = &raw mut SHARED_MEMORY_SEGMENTS;
        if shm_id >= (*segments).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        // 先获取共享内存段的引用
        let segment = (&(*segments))[shm_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let mut segment = segment.lock();

        let ref_count = segment.decrement_ref_count();
        if ref_count == 0 {
            // 释放物理内存
            let pages = segment.size.div_ceil(4096);
            let physical_address = segment.physical_address;
            for i in 0..pages {
                let _ = crate::mm::physical::free_page(physical_address + i * 4096);
            }
            segment.is_allocated = false;
            // 现在可以安全地修改segments
            (&mut (*segments))[shm_id] = None;
        }

        Ok(())
    }
}

// 获取共享内存信息
#[allow(dead_code)]
pub fn get_shared_memory_info(shm_id: usize) -> IpcResult<(usize, usize)> {
    unsafe {
        let segments = &raw const SHARED_MEMORY_SEGMENTS;
        if shm_id >= (*segments).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let segment = (&(*segments))[shm_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let segment = segment.lock();

        Ok((segment.physical_address, segment.size))
    }
}