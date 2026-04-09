//! 飞地专属调度队列
//!
//! 为每类飞地单独创建一个专属调度队列，采用无锁环形缓冲区结构。

use super::lockfree_ringbuffer::LockFreeRingBuffer;
use crate::enclave::core::EnclaveType;
use core::sync::atomic::{AtomicUsize, Ordering};

/// 队列状态信息
#[derive(Debug, Clone, Copy)]
pub struct QueueStatus {
    /// 当前队列长度
    pub length: usize,
    /// 队列是否为空
    pub is_empty: bool,
    /// 队列是否已满
    pub is_full: bool,
    /// 总入队次数
    pub total_enqueues: usize,
    /// 总出队次数
    pub total_dequeues: usize,
}

/// 飞地调度队列
pub struct EnclaveQueue {
    /// 飞地类型
    enclave_type: EnclaveType,
    /// 无锁环形缓冲区
    ring_buffer: LockFreeRingBuffer,
    /// 总入队次数
    total_enqueues: AtomicUsize,
    /// 总出队次数
    total_dequeues: AtomicUsize,
}

impl EnclaveQueue {
    /// 创建新的飞地队列
    pub fn new(enclave_type: EnclaveType) -> Self {
        EnclaveQueue {
            enclave_type,
            ring_buffer: LockFreeRingBuffer::new(),
            total_enqueues: AtomicUsize::new(0),
            total_dequeues: AtomicUsize::new(0),
        }
    }

    /// 获取队列类型
    pub fn enclave_type(&self) -> EnclaveType {
        self.enclave_type
    }

    /// 入队操作
    pub fn enqueue(&self, enclave_type: EnclaveType) -> Result<(), &'static str> {
        if enclave_type != self.enclave_type {
            return Err("Enclave type mismatch");
        }

        let result = self.ring_buffer.enqueue(enclave_type);
        if result.is_ok() {
            self.total_enqueues.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// 出队操作
    pub fn dequeue(&self) -> Option<EnclaveType> {
        let result = self.ring_buffer.dequeue();
        if result.is_some() {
            self.total_dequeues.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.ring_buffer.is_empty()
    }

    /// 检查队列是否已满
    pub fn is_full(&self) -> bool {
        self.ring_buffer.is_full()
    }

    /// 获取当前队列长度
    pub fn len(&self) -> usize {
        self.ring_buffer.len()
    }

    /// 获取队列状态
    pub fn status(&self) -> QueueStatus {
        QueueStatus {
            length: self.len(),
            is_empty: self.is_empty(),
            is_full: self.is_full(),
            total_enqueues: self.total_enqueues.load(Ordering::Relaxed),
            total_dequeues: self.total_dequeues.load(Ordering::Relaxed),
        }
    }
}

/// 飞地队列优先级（数值越大优先级越高）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EnclaveQueuePriority {
    /// NetworkEnclave 队列（最低优先级）
    Network = 0,
    /// MediaEnclave 队列
    Media = 1,
    /// GraphicEnclave 队列
    Graphic = 2,
    /// AIEnclave 队列（最高优先级）
    AI = 3,
}

impl From<EnclaveType> for EnclaveQueuePriority {
    fn from(enclave_type: EnclaveType) -> Self {
        match enclave_type {
            EnclaveType::NetworkEnclave => EnclaveQueuePriority::Network,
            EnclaveType::MediaEnclave => EnclaveQueuePriority::Media,
            EnclaveType::GraphicEnclave => EnclaveQueuePriority::Graphic,
            EnclaveType::AIEnclave => EnclaveQueuePriority::AI,
        }
    }
}

/// 飞地时间片配置
#[derive(Debug, Clone, Copy)]
pub struct TimeSliceConfig {
    /// AIEnclave 时间片（微秒）
    pub ai_time_slice: u64,
    /// GraphicEnclave 时间片（微秒）
    pub graphic_time_slice: u64,
    /// MediaEnclave 时间片（微秒）
    pub media_time_slice: u64,
    /// NetworkEnclave 时间片（微秒）
    pub network_time_slice: u64,
}

impl Default for TimeSliceConfig {
    fn default() -> Self {
        TimeSliceConfig {
            ai_time_slice: 100_000,      // 100ms
            graphic_time_slice: 80_000,   // 80ms
            media_time_slice: 60_000,     // 60ms
            network_time_slice: 40_000,   // 40ms
        }
    }
}

impl TimeSliceConfig {
    /// 获取指定飞地类型的时间片
    pub fn get_time_slice(&self, enclave_type: EnclaveType) -> u64 {
        match enclave_type {
            EnclaveType::AIEnclave => self.ai_time_slice,
            EnclaveType::GraphicEnclave => self.graphic_time_slice,
            EnclaveType::MediaEnclave => self.media_time_slice,
            EnclaveType::NetworkEnclave => self.network_time_slice,
        }
    }
}
