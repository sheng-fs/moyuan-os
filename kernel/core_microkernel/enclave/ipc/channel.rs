//! 通道管理模块
//!
//! 实现IPC通道的建立、管理和数据传输流程。

use super::error::*;
use super::memory_pool::SharedMemoryBlock;
use super::encryption::SessionKey;
use super::memory_pool::EncryptedBlock;
use crate::enclave::core::EnclaveType;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// 通道ID类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(pub u64);

/// 通道优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ChannelPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 实时优先级
    Realtime = 3,
}

/// IPC通道状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelState {
    /// 未初始化
    Uninitialized,
    /// 建立中
    Establishing,
    /// 活跃
    Active,
    /// 关闭中
    Closing,
    /// 已关闭
    Closed,
}

/// IPC通道
pub struct IpcChannel {
    /// 通道ID
    channel_id: ChannelId,
    /// 发送方
    sender: EnclaveType,
    /// 接收方
    receiver: EnclaveType,
    /// 共享内存块
    memory_block: SharedMemoryBlock,
    /// 会话密钥
    session_key: SessionKey,
    /// 优先级
    priority: ChannelPriority,
    /// 状态
    state: ChannelState,
    /// 下一个序列号
    next_sequence_number: AtomicU64,
    /// 期望的序列号
    expected_sequence_number: AtomicU64,
    /// 创建时间戳
    created_at: u64,
    /// 总传输字节数
    total_bytes_transferred: AtomicUsize,
}

impl IpcChannel {
    /// 创建新的IPC通道
    pub fn new(
        channel_id: ChannelId,
        sender: EnclaveType,
        receiver: EnclaveType,
        memory_block: SharedMemoryBlock,
        session_key: SessionKey,
        priority: ChannelPriority,
    ) -> Self {
        IpcChannel {
            channel_id,
            sender,
            receiver,
            memory_block,
            session_key,
            priority,
            state: ChannelState::Establishing,
            next_sequence_number: AtomicU64::new(0),
            expected_sequence_number: AtomicU64::new(0),
            created_at: 0,
            total_bytes_transferred: AtomicUsize::new(0),
        }
    }

    /// 获取通道ID
    pub fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    /// 获取发送方
    pub fn sender(&self) -> EnclaveType {
        self.sender
    }

    /// 获取接收方
    pub fn receiver(&self) -> EnclaveType {
        self.receiver
    }

    /// 获取共享内存块
    pub fn memory_block(&self) -> &SharedMemoryBlock {
        &self.memory_block
    }

    /// 获取可变的共享内存块
    pub fn memory_block_mut(&mut self) -> &mut SharedMemoryBlock {
        &mut self.memory_block
    }

    /// 获取会话密钥
    pub fn session_key(&self) -> &SessionKey {
        &self.session_key
    }

    /// 获取优先级
    pub fn priority(&self) -> ChannelPriority {
        self.priority
    }

    /// 获取状态
    pub fn state(&self) -> ChannelState {
        self.state
    }

    /// 设置状态
    pub fn set_state(&mut self, state: ChannelState) {
        self.state = state;
    }

    /// 获取下一个序列号
    pub fn next_sequence_number(&self) -> u64 {
        self.next_sequence_number.load(Ordering::Acquire)
    }

    /// 推进序列号
    pub fn advance_sequence_number(&self, count: u64) {
        self.next_sequence_number.fetch_add(count, Ordering::Release);
    }

    /// 获取期望的序列号
    pub fn expected_sequence_number(&self) -> u64 {
        self.expected_sequence_number.load(Ordering::Acquire)
    }

    /// 验证序列号
    pub fn verify_sequence_numbers(
        &self,
        encrypted_blocks: &[EncryptedBlock],
    ) -> Result<(), EnclaveIpcError> {
        let expected = self.expected_sequence_number.load(Ordering::Acquire);

        for (i, block) in encrypted_blocks.iter().enumerate() {
            let expected_seq = expected + i as u64;
            if block.sequence_number != expected_seq {
                return Err(EnclaveIpcError::SequenceNumberVerificationFailed);
            }
        }

        // 更新期望的序列号
        self.expected_sequence_number.store(
            expected + encrypted_blocks.len() as u64,
            Ordering::Release,
        );

        Ok(())
    }

    /// 记录传输的字节数
    pub fn record_transfer(&self, bytes: usize) {
        self.total_bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 获取总传输字节数
    pub fn total_bytes_transferred(&self) -> usize {
        self.total_bytes_transferred.load(Ordering::Relaxed)
    }

    /// 检查通道是否活跃
    pub fn is_active(&self) -> bool {
        self.state == ChannelState::Active
    }
}

/// 通道管理器
pub struct ChannelManager {
    /// 活跃通道列表
    channels: Vec<IpcChannel>,
    /// 下一个通道ID
    next_channel_id: AtomicU64,
    /// 初始化标志
    initialized: bool,
}

impl ChannelManager {
    /// 创建新的通道管理器
    pub fn new() -> Self {
        ChannelManager {
            channels: Vec::new(),
            next_channel_id: AtomicU64::new(0),
            initialized: false,
        }
    }

    /// 初始化通道管理器
    pub fn init(&mut self) -> Result<(), EnclaveIpcError> {
        crate::println!("Initializing Channel Manager...");
        self.initialized = true;
        crate::println!("Channel Manager initialized");
        Ok(())
    }

    /// 创建通道
    pub fn create_channel(
        &mut self,
        sender: EnclaveType,
        receiver: EnclaveType,
        memory_block: SharedMemoryBlock,
        session_key: SessionKey,
        priority: ChannelPriority,
    ) -> Result<ChannelId, ChannelError> {
        let channel_id = ChannelId(self.next_channel_id.fetch_add(1, Ordering::Relaxed));

        let mut channel = IpcChannel::new(
            channel_id,
            sender,
            receiver,
            memory_block,
            session_key,
            priority,
        );

        channel.set_state(ChannelState::Active);

        crate::println!(
            "Created IPC channel: id={:?}, sender={:?}, receiver={:?}, priority={:?}",
            channel_id, sender, receiver, priority
        );

        self.channels.push(channel);
        Ok(channel_id)
    }

    /// 获取通道
    pub fn get_channel(&self, channel_id: ChannelId) -> Result<&IpcChannel, ChannelError> {
        self.channels
            .iter()
            .find(|c| c.channel_id() == channel_id)
            .ok_or(ChannelError::NotFound)
    }

    /// 获取可变通道
    pub fn get_channel_mut(&mut self, channel_id: ChannelId) -> Result<&mut IpcChannel, ChannelError> {
        self.channels
            .iter_mut()
            .find(|c| c.channel_id() == channel_id)
            .ok_or(ChannelError::NotFound)
    }

    /// 移除通道
    pub fn remove_channel(&mut self, channel_id: ChannelId) -> Result<IpcChannel, ChannelError> {
        let pos = self.channels
            .iter()
            .position(|c| c.channel_id() == channel_id)
            .ok_or(ChannelError::NotFound)?;

        let mut channel = self.channels.remove(pos);
        channel.set_state(ChannelState::Closed);

        crate::println!("Removed IPC channel: id={:?}", channel_id);
        Ok(channel)
    }

    /// 获取所有活跃通道
    pub fn active_channels(&self) -> Vec<&IpcChannel> {
        self.channels
            .iter()
            .filter(|c| c.is_active())
            .collect()
    }

    /// 获取通道统计信息
    pub fn channel_stats(&self) -> ChannelManagerStats {
        let active_count = self.active_channels().len();
        let total_bytes: usize = self.channels
            .iter()
            .map(|c| c.total_bytes_transferred())
            .sum();

        ChannelManagerStats {
            total_channels: self.channels.len(),
            active_channels: active_count,
            total_bytes_transferred: total_bytes,
        }
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 通道管理器统计信息
#[derive(Debug, Clone, Copy)]
pub struct ChannelManagerStats {
    /// 总通道数
    pub total_channels: usize,
    /// 活跃通道数
    pub active_channels: usize,
    /// 总传输字节数
    pub total_bytes_transferred: usize,
}
