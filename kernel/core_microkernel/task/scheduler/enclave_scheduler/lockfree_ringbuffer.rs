//! 无锁环形缓冲区实现
//!
//! 使用 CAS (Compare-And-Swap) 操作实现无锁的环形缓冲区，
//! 减少调度器在入队、出队操作时的锁竞争开销。

use core::sync::atomic::{AtomicUsize, Ordering};
use crate::enclave::core::EnclaveType;

/// 环形缓冲区大小 - 必须是 2 的幂
const RING_BUFFER_SIZE: usize = 64;
const RING_BUFFER_MASK: usize = RING_BUFFER_SIZE - 1;

/// 无锁环形缓冲区
pub struct LockFreeRingBuffer {
    /// 缓冲区数据
    buffer: [Option<EnclaveType>; RING_BUFFER_SIZE],
    /// 读指针
    read_idx: AtomicUsize,
    /// 写指针
    write_idx: AtomicUsize,
}

impl LockFreeRingBuffer {
    /// 创建新的无锁环形缓冲区
    pub fn new() -> Self {
        LockFreeRingBuffer {
            buffer: [None; RING_BUFFER_SIZE],
            read_idx: AtomicUsize::new(0),
            write_idx: AtomicUsize::new(0),
        }
    }

    /// 入队操作
    pub fn enqueue(&self, item: EnclaveType) -> Result<(), &'static str> {
        let mut write_idx;
        let mut next_write_idx;

        loop {
            write_idx = self.write_idx.load(Ordering::Acquire);
            let read_idx = self.read_idx.load(Ordering::Acquire);

            next_write_idx = (write_idx + 1) & RING_BUFFER_MASK;

            // 检查缓冲区是否已满
            if next_write_idx == read_idx {
                return Err("Ring buffer is full");
            }

            // 尝试 CAS 更新写指针
            if self.write_idx.compare_exchange_weak(
                write_idx,
                next_write_idx,
                Ordering::AcqRel, // 使用 AcqRel 确保内存屏障
                Ordering::Relaxed
            ).is_ok() {
                break;
            }
        }

        // 写入数据 - 使用适当的内存屏障
        unsafe {
            let buffer_ptr = &self.buffer as *const [Option<EnclaveType>; RING_BUFFER_SIZE] as *mut [Option<EnclaveType>; RING_BUFFER_SIZE];
            // 确保写操作不会被重排序
            core::sync::atomic::fence(Ordering::Release);
            (*buffer_ptr)[write_idx] = Some(item);
        }

        Ok(())
    }

    /// 出队操作
    pub fn dequeue(&self) -> Option<EnclaveType> {
        let mut read_idx;

        loop {
            read_idx = self.read_idx.load(Ordering::Acquire);
            let write_idx = self.write_idx.load(Ordering::Acquire);

            // 检查缓冲区是否为空
            if read_idx == write_idx {
                return None;
            }

            let next_read_idx = (read_idx + 1) & RING_BUFFER_MASK;

            // 尝试 CAS 更新读指针
            if self.read_idx.compare_exchange_weak(
                read_idx,
                next_read_idx,
                Ordering::AcqRel, // 使用 AcqRel 确保内存屏障
                Ordering::Relaxed
            ).is_ok() {
                break;
            }
        }

        // 读取数据 - 使用适当的内存屏障
        unsafe {
            let buffer_ptr = &self.buffer as *const [Option<EnclaveType>; RING_BUFFER_SIZE] as *mut [Option<EnclaveType>; RING_BUFFER_SIZE];
            // 确保读操作不会被重排序
            core::sync::atomic::fence(Ordering::Acquire);
            let item = (*buffer_ptr)[read_idx].take();
            item
        }
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.read_idx.load(Ordering::Acquire) == self.write_idx.load(Ordering::Acquire)
    }

    /// 检查缓冲区是否已满
    pub fn is_full(&self) -> bool {
        let read_idx = self.read_idx.load(Ordering::Acquire);
        let write_idx = self.write_idx.load(Ordering::Acquire);
        ((write_idx + 1) & RING_BUFFER_MASK) == read_idx
    }

    /// 获取当前元素数量
    pub fn len(&self) -> usize {
        let read_idx = self.read_idx.load(Ordering::Acquire);
        let write_idx = self.write_idx.load(Ordering::Acquire);

        if write_idx >= read_idx {
            write_idx - read_idx
        } else {
            RING_BUFFER_SIZE - read_idx + write_idx
        }
    }
}

impl Default for LockFreeRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}
