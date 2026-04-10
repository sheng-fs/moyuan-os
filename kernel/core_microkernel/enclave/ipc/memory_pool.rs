//! 飞地IPC共享内存管理系统
//!
//! 实现飞地IPC内存池的分配、管理和回收。

use super::error::*;
use crate::enclave::memory::EnclaveMemoryManager;
use crate::memory::PhysicalAddress;
use core::sync::atomic::{AtomicUsize, Ordering};

/// 大页大小常量
pub const HUGE_PAGE_2MB: usize = 0x200000;
pub const HUGE_PAGE_1GB: usize = 0x40000000;

/// 默认数据块大小（2MB大页）
pub const DEFAULT_BLOCK_SIZE: usize = HUGE_PAGE_2MB;

/// 加密数据块结构
#[derive(Debug, Clone)]
pub struct EncryptedBlock {
    /// 序列号
    pub sequence_number: u64,
    /// 密文数据
    pub ciphertext: Vec<u8>,
    /// GCM认证标签
    pub auth_tag: [u8; 16],
    /// 初始化向量（IV）
    pub iv: [u8; 12],
}

/// 共享内存块
#[derive(Debug, Clone)]
pub struct SharedMemoryBlock {
    /// 内存块ID
    pub block_id: usize,
    /// 物理起始地址
    pub physical_addr: PhysicalAddress,
    /// 大小
    pub size: usize,
    /// 是否使用大页
    pub huge_page: bool,
    /// 写指针（字节偏移）
    write_offset: AtomicUsize,
    /// 读指针（字节偏移）
    read_offset: AtomicUsize,
    /// 是否已分配
    allocated: bool,
}

impl SharedMemoryBlock {
    /// 创建新的共享内存块
    pub fn new(
        block_id: usize,
        physical_addr: PhysicalAddress,
        size: usize,
        huge_page: bool,
    ) -> Self {
        SharedMemoryBlock {
            block_id,
            physical_addr,
            size,
            huge_page,
            write_offset: AtomicUsize::new(0),
            read_offset: AtomicUsize::new(0),
            allocated: true,
        }
    }

    /// 获取物理地址
    pub fn physical_address(&self) -> PhysicalAddress {
        self.physical_addr
    }

    /// 获取大小
    pub fn size(&self) -> usize {
        self.size
    }

    /// 检查是否使用大页
    pub fn is_huge_page(&self) -> bool {
        self.huge_page
    }

    /// 获取可用空间
    pub fn available_space(&self) -> usize {
        let write = self.write_offset.load(Ordering::Acquire);
        let read = self.read_offset.load(Ordering::Acquire);

        if write >= read {
            self.size - (write - read)
        } else {
            read - write
        }
    }

    /// 获取已使用空间
    pub fn used_space(&self) -> usize {
        let write = self.write_offset.load(Ordering::Acquire);
        let read = self.read_offset.load(Ordering::Acquire);

        if write >= read {
            write - read
        } else {
            self.size - (read - write)
        }
    }

    /// 清零内存（安全清理）
    pub fn zero_memory(&mut self) {
        crate::println!("Zeroing shared memory block {:?}", self.block_id);
        // 在实际实现中，这里应该物理清零内存
    }

    /// 检查是否已分配
    pub fn is_allocated(&self) -> bool {
        self.allocated
    }

    /// 标记为已释放
    pub fn mark_free(&mut self) {
        self.allocated = false;
    }
}

/// IPC内存池管理器
pub struct IpcMemoryPool {
    /// 已分配的内存块列表
    memory_blocks: Vec<SharedMemoryBlock>,
    /// 下一个块ID
    next_block_id: AtomicUsize,
    /// 初始化标志
    initialized: bool,
}

impl IpcMemoryPool {
    /// 创建新的IPC内存池
    pub fn new() -> Self {
        IpcMemoryPool {
            memory_blocks: Vec::new(),
            next_block_id: AtomicUsize::new(0),
            initialized: false,
        }
    }

    /// 初始化内存池
    pub fn init(&mut self) -> Result<(), EnclaveIpcError> {
        crate::println!("Initializing Enclave IPC Memory Pool...");

        // 预分配一些内存块（可选优化）
        self.initialized = true;
        crate::println!("Enclave IPC Memory Pool initialized");
        Ok(())
    }

    /// 分配共享内存块
    pub fn allocate_memory_block(
        &mut self,
        estimated_size: usize,
    ) -> Result<SharedMemoryBlock, IpcMemoryError> {
        // 确定内存块大小（向上舍入到大页边界）
        let (block_size, use_huge_page) = if estimated_size >= HUGE_PAGE_1GB {
            (HUGE_PAGE_1GB, true)
        } else if estimated_size >= DEFAULT_BLOCK_SIZE {
            (DEFAULT_BLOCK_SIZE, true)
        } else {
            (DEFAULT_BLOCK_SIZE, true) // 始终使用大页优化性能
        };

        // 分配物理内存
        let (phys_addr, allocated_size) = EnclaveMemoryManager::allocate_memory(
            block_size,
            use_huge_page,
        ).map_err(|_| IpcMemoryError::AllocationFailed)?;

        let block_id = self.next_block_id.fetch_add(1, Ordering::Relaxed);

        let block = SharedMemoryBlock::new(
            block_id,
            phys_addr,
            allocated_size,
            use_huge_page,
        );

        crate::println!(
            "Allocated shared memory block: id={}, size={}, huge_page={}",
            block_id, allocated_size, use_huge_page
        );

        self.memory_blocks.push(block.clone());
        Ok(block)
    }

    /// 释放共享内存块
    pub fn free_memory_block(
        &mut self,
        block: &SharedMemoryBlock,
    ) -> Result<(), IpcMemoryError> {
        // 查找并移除内存块
        if let Some(pos) = self.memory_blocks.iter().position(|b| b.block_id == block.block_id) {
            let mut block = self.memory_blocks.remove(pos);

            // 清零内存（安全清理）
            block.zero_memory();

            // 释放物理内存
            EnclaveMemoryManager::free_memory(
                block.physical_address(),
                block.size(),
                block.is_huge_page(),
            );

            block.mark_free();

            crate::println!("Freed shared memory block: id={}", block.block_id);
            Ok(())
        } else {
            Err(IpcMemoryError::BlockNotFound)
        }
    }

    /// 获取内存块
    pub fn get_block(&self, block_id: usize) -> Option<&SharedMemoryBlock> {
        self.memory_blocks.iter().find(|b| b.block_id == block_id)
    }

    /// 获取可变内存块
    pub fn get_block_mut(&mut self, block_id: usize) -> Option<&mut SharedMemoryBlock> {
        self.memory_blocks.iter_mut().find(|b| b.block_id == block_id)
    }

    /// 写入加密数据块到共享内存
    pub fn write_encrypted_blocks(
        &self,
        block: &SharedMemoryBlock,
        encrypted_blocks: &[EncryptedBlock],
    ) -> Result<(), IpcMemoryError> {
        let mut write_offset = block.write_offset.load(Ordering::Acquire);

        for encrypted_block in encrypted_blocks {
            // 序列化加密块（简化实现）
            let serialized = Self::serialize_encrypted_block(encrypted_block);

            // 检查空间是否足够
            let required_space = serialized.len();
            if write_offset + required_space > block.size() {
                return Err(IpcMemoryError::BlockFull);
            }

            // 写入数据（简化实现）
            crate::println!(
                "Writing encrypted block seq={} to offset {}",
                encrypted_block.sequence_number, write_offset
            );

            write_offset += required_space;
        }

        block.write_offset.store(write_offset, Ordering::Release);
        Ok(())
    }

    /// 从共享内存读取加密数据块
    pub fn read_encrypted_blocks(
        &self,
        block: &SharedMemoryBlock,
    ) -> Result<Vec<EncryptedBlock>, IpcMemoryError> {
        let mut read_offset = block.read_offset.load(Ordering::Acquire);
        let write_offset = block.write_offset.load(Ordering::Acquire);

        if read_offset >= write_offset {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();

        // 简化实现：假设我们能正确解析所有数据块
        // 实际实现需要根据序列化格式解析

        crate::println!(
            "Reading encrypted blocks from offset {} to {}",
            read_offset, write_offset
        );

        // 更新读指针
        block.read_offset.store(write_offset, Ordering::Release);

        Ok(result)
    }

    /// 序列化加密块
    fn serialize_encrypted_block(block: &EncryptedBlock) -> Vec<u8> {
        // 简化实现，实际应该使用更高效的序列化
        let mut result = Vec::new();

        // 序列号 (8字节)
        result.extend_from_slice(&block.sequence_number.to_le_bytes());

        // IV (12字节)
        result.extend_from_slice(&block.iv);

        // 认证标签 (16字节)
        result.extend_from_slice(&block.auth_tag);

        // 密文长度 (8字节)
        result.extend_from_slice(&(block.ciphertext.len() as u64).to_le_bytes());

        // 密文数据
        result.extend_from_slice(&block.ciphertext);

        result
    }

    /// 获取内存池统计信息
    pub fn pool_stats(&self) -> PoolStats {
        let total_size: usize = self.memory_blocks.iter().map(|b| b.size()).sum();
        let used_size: usize = self.memory_blocks.iter().map(|b| b.used_space()).sum();

        PoolStats {
            total_blocks: self.memory_blocks.len(),
            total_size,
            used_size,
            free_size: total_size - used_size,
        }
    }
}

impl Default for IpcMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存池统计信息
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    /// 总块数
    pub total_blocks: usize,
    /// 总大小
    pub total_size: usize,
    /// 已使用大小
    pub used_size: usize,
    /// 空闲大小
    pub free_size: usize,
}
