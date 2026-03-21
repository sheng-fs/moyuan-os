//! 飞地内存管理模块

use super::error::*;
use crate::memory::{PhysicalAddress, VirtualAddress};
use crate::mm::physical::allocate_page;
use crate::mm::physical::free_page;

/// 飞地内存区域
#[derive(Debug, Clone, Copy)]
pub struct EnclaveMemoryRegion {
    /// 虚拟地址起始
    pub virtual_start: VirtualAddress,
    /// 物理地址起始
    pub physical_start: PhysicalAddress,
    /// 大小
    pub size: usize,
    /// 有效状态
    pub valid: bool,
    /// 是否使用大页
    pub huge_page: bool,
}

impl EnclaveMemoryRegion {
    /// 创建内存区域
    pub fn new(
        virtual_start: VirtualAddress,
        physical_start: PhysicalAddress,
        size: usize,
        huge_page: bool,
    ) -> Self {
        Self {
            virtual_start,
            physical_start,
            size,
            valid: true,
            huge_page,
        }
    }
    
    /// 检查地址是否在区域内
    pub fn contains(&self, addr: PhysicalAddress, size: usize) -> bool {
        addr >= self.physical_start &&
        addr + size <= self.physical_start + self.size
    }
    
    /// 获取结束地址
    pub fn physical_end(&self) -> PhysicalAddress {
        self.physical_start + self.size
    }
    
    /// 获取虚拟结束地址
    pub fn virtual_end(&self) -> VirtualAddress {
        self.virtual_start + self.size
    }
}

/// 飞地内存管理器
pub struct EnclaveMemoryManager;

impl EnclaveMemoryManager {
    /// 分配飞地内存
    pub fn allocate_memory(
        size: usize,
        huge_page: bool,
    ) -> Result<(PhysicalAddress, usize), EnclaveError> {
        crate::println!("Allocating enclave memory: {} bytes, huge_page: {}", size, huge_page);
        
        let mut allocated_size = 0;
        let mut current_addr = PhysicalAddress::new(0);
        
        // 分配内存
        let page_size = if huge_page { 0x200000 } else { 0x1000 }; // 2MB 大页或 4KB 标准页
        let num_pages = (size + page_size - 1) / page_size;
        
        for i in 0..num_pages {
            if let Ok(page) = allocate_page() {
                if i == 0 {
                    current_addr = PhysicalAddress::new(page);
                }
                allocated_size += page_size;
            } else {
                // 释放已分配的内存
                for j in 0..i {
                    let _ = free_page(current_addr.value() + j * page_size);
                }
                return Err(EnclaveError::MemoryAllocationFailed);
            }
        }
        
        Ok((current_addr, allocated_size))
    }
    
    /// 释放飞地内存
    pub fn free_memory(addr: PhysicalAddress, size: usize, huge_page: bool) {
        crate::println!("Freeing enclave memory: {:?}, {} bytes", addr, size);
        
        let page_size = if huge_page { 0x200000 } else { 0x1000 };
        let num_pages = (size + page_size - 1) / page_size;
        
        for i in 0..num_pages {
            let _ = free_page(addr.value() + i * page_size);
        }
    }
    
    /// 创建内存区域
    pub fn create_region(
        virtual_start: VirtualAddress,
        physical_start: PhysicalAddress,
        size: usize,
        huge_page: bool,
    ) -> EnclaveMemoryRegion {
        EnclaveMemoryRegion::new(virtual_start, physical_start, size, huge_page)
    }
}
