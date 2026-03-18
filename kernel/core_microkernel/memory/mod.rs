//! 内存模块
//! 
//! 定义内存地址类型和内存管理功能

use core::fmt;

/// 物理地址
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress {
    addr: usize,
}

impl PhysicalAddress {
    /// 创建物理地址
    pub const fn new(addr: usize) -> Self {
        Self { addr }
    }
    
    /// 获取地址值
    pub const fn value(&self) -> usize {
        self.addr
    }
    
    /// 偏移地址
    pub fn offset(&self, offset: usize) -> Self {
        Self::new(self.addr + offset)
    }
}

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.addr)
    }
}

/// 虚拟地址
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress {
    addr: usize,
}

impl VirtualAddress {
    /// 创建虚拟地址
    pub const fn new(addr: usize) -> Self {
        Self { addr }
    }
    
    /// 获取地址值
    pub const fn value(&self) -> usize {
        self.addr
    }
    
    /// 偏移地址
    pub fn offset(&self, offset: usize) -> Self {
        Self::new(self.addr + offset)
    }
}

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.addr)
    }
}

/// 内存页大小
pub const PAGE_SIZE: usize = 4096;

/// 内存区域
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRegion {
    /// 基地址
    pub base: PhysicalAddress,
    /// 大小
    pub size: usize,
    /// 是否可写
    pub writable: bool,
    /// 是否可执行
    pub executable: bool,
}

impl MemoryRegion {
    /// 创建内存区域
    pub fn new(base: PhysicalAddress, size: usize, writable: bool, executable: bool) -> Self {
        Self {
            base,
            size,
            writable,
            executable,
        }
    }
    
    /// 检查地址是否在区域内
    pub fn contains(&self, addr: PhysicalAddress) -> bool {
        addr >= self.base && addr < self.base.offset(self.size)
    }
    
    /// 获取结束地址
    pub fn end(&self) -> PhysicalAddress {
        self.base.offset(self.size)
    }
}