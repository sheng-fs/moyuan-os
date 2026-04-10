//! 飞地间IPC（Inter-Process Communication）通道系统
//!
//! 安全高效的飞地间通信系统，基于硬件隔离、AES-256-GCM加密和共享内存机制。

pub mod error;
pub mod memory_pool;
pub mod encryption;
pub mod channel;
pub mod security;

pub use self::error::*;
pub use self::memory_pool::*;
pub use self::encryption::*;
pub use self::channel::*;
pub use self::security::*;

use crate::enclave::core::EnclaveType;
use spin::Mutex;

/// 飞地IPC管理器
pub struct EnclaveIpcManager {
    /// 内存池管理器
    memory_pool: IpcMemoryPool,
    /// 加密管理器
    encryption_manager: EncryptionManager,
    /// 通道管理器
    channel_manager: ChannelManager,
    /// 安全控制器
    security_controller: SecurityController,
    /// 初始化标志
    initialized: bool,
}

impl EnclaveIpcManager {
    /// 创建新的飞地IPC管理器
    pub fn new() -> Self {
        EnclaveIpcManager {
            memory_pool: IpcMemoryPool::new(),
            encryption_manager: EncryptionManager::new(),
            channel_manager: ChannelManager::new(),
            security_controller: SecurityController::new(),
            initialized: false,
        }
    }

    /// 初始化飞地IPC系统
    pub fn init(&mut self) -> Result<(), EnclaveIpcError> {
        crate::println!("Initializing Enclave IPC System...");

        // 初始化内存池
        self.memory_pool.init()?;

        // 初始化加密管理器
        self.encryption_manager.init()?;

        // 初始化通道管理器
        self.channel_manager.init()?;

        // 初始化安全控制器
        self.security_controller.init()?;

        self.initialized = true;
        crate::println!("Enclave IPC System initialized successfully");
        Ok(())
    }

    /// 建立IPC通道
    pub fn establish_channel(
        &mut self,
        sender: EnclaveType,
        receiver: EnclaveType,
        estimated_size: usize,
        priority: ChannelPriority,
    ) -> Result<ChannelId, EnclaveIpcError> {
        // 1. 权限验证
        self.security_controller.verify_communication_permission(sender, receiver)?;

        // 2. 分配共享内存
        let memory_block = self.memory_pool.allocate_memory_block(estimated_size)?;

        // 3. 生成会话密钥
        let session_key = self.encryption_manager.generate_session_key()?;

        // 4. 创建通道
        let channel_id = self.channel_manager.create_channel(
            sender,
            receiver,
            memory_block,
            session_key,
            priority,
        )?;

        // 5. 配置IOMMU权限
        self.security_controller.configure_iommu_permissions(
            channel_id,
            sender,
            receiver,
            &memory_block,
        )?;

        // 6. 分发密钥和内存信息
        self.distribute_channel_info(channel_id, sender, receiver)?;

        Ok(channel_id)
    }

    /// 分发通道信息给通信双方
    fn distribute_channel_info(
        &self,
        channel_id: ChannelId,
        sender: EnclaveType,
        receiver: EnclaveType,
    ) -> Result<(), EnclaveIpcError> {
        crate::println!("Distributing channel info for channel {:?}", channel_id);
        // 在实际实现中，这里应该通过安全通道向双方飞地分发信息
        Ok(())
    }

    /// 发送数据
    pub fn send_data(
        &mut self,
        channel_id: ChannelId,
        sender: EnclaveType,
        data: &[u8],
    ) -> Result<(), EnclaveIpcError> {
        // 获取通道
        let channel = self.channel_manager.get_channel_mut(channel_id)?;

        // 验证发送方
        if channel.sender() != sender {
            return Err(EnclaveIpcError::InvalidSender);
        }

        // 数据分块与加密
        let encrypted_blocks = self.encryption_manager.encrypt_data(
            &channel.session_key,
            data,
            channel.next_sequence_number(),
        )?;

        // 写入共享内存
        self.memory_pool.write_encrypted_blocks(
            channel.memory_block(),
            &encrypted_blocks,
        )?;

        // 更新序列号
        channel.advance_sequence_number(encrypted_blocks.len() as u64);

        // 发送通知
        self.notify_data_ready(channel_id, channel.receiver())?;

        Ok(())
    }

    /// 接收数据
    pub fn receive_data(
        &mut self,
        channel_id: ChannelId,
        receiver: EnclaveType,
        buffer: &mut [u8],
    ) -> Result<usize, EnclaveIpcError> {
        // 获取通道
        let channel = self.channel_manager.get_channel(channel_id)?;

        // 验证接收方
        if channel.receiver() != receiver {
            return Err(EnclaveIpcError::InvalidReceiver);
        }

        // 从共享内存读取加密块
        let encrypted_blocks = self.memory_pool.read_encrypted_blocks(channel.memory_block())?;

        // 解密数据
        let decrypted_data = self.encryption_manager.decrypt_data(
            &channel.session_key,
            &encrypted_blocks,
            channel.expected_sequence_number(),
        )?;

        // 验证序列号
        channel.verify_sequence_numbers(&encrypted_blocks)?;

        // 复制数据到输出缓冲区
        let copy_len = decrypted_data.len().min(buffer.len());
        buffer[..copy_len].copy_from_slice(&decrypted_data[..copy_len]);

        Ok(copy_len)
    }

    /// 关闭通道
    pub fn close_channel(&mut self, channel_id: ChannelId) -> Result<(), EnclaveIpcError> {
        // 获取通道
        let channel = self.channel_manager.remove_channel(channel_id)?;

        // 清理加密密钥
        self.encryption_manager.destroy_session_key(channel.session_key())?;

        // 释放共享内存
        self.memory_pool.free_memory_block(channel.memory_block())?;

        // 清理IOMMU配置
        self.security_controller.cleanup_iommu_permissions(channel_id)?;

        crate::println!("Channel {:?} closed successfully", channel_id);
        Ok(())
    }

    /// 通知数据准备就绪
    fn notify_data_ready(
        &self,
        channel_id: ChannelId,
        receiver: EnclaveType,
    ) -> Result<(), EnclaveIpcError> {
        crate::println!("Notifying {:?} of data ready on channel {:?}", receiver, channel_id);
        // 在实际实现中，这里应该使用事件通知机制
        Ok(())
    }

    /// 获取内存池管理器
    pub fn memory_pool(&self) -> &IpcMemoryPool {
        &self.memory_pool
    }

    /// 获取加密管理器
    pub fn encryption_manager(&self) -> &EncryptionManager {
        &self.encryption_manager
    }

    /// 获取通道管理器
    pub fn channel_manager(&self) -> &ChannelManager {
        &self.channel_manager
    }

    /// 获取安全控制器
    pub fn security_controller(&self) -> &SecurityController {
        &self.security_controller
    }
}

impl Default for EnclaveIpcManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局飞地IPC管理器实例
static mut ENCLAVE_IPC_MANAGER: Option<Mutex<EnclaveIpcManager>> = None;

/// 初始化飞地IPC系统
pub fn init_enclave_ipc() -> Result<(), EnclaveIpcError> {
    unsafe {
        let mut manager = EnclaveIpcManager::new();
        manager.init()?;
        ENCLAVE_IPC_MANAGER = Some(Mutex::new(manager));
    }
    Ok(())
}

/// 获取飞地IPC管理器
pub fn get_enclave_ipc_manager() -> &'static Mutex<EnclaveIpcManager> {
    unsafe {
        ENCLAVE_IPC_MANAGER.as_ref().expect("Enclave IPC manager not initialized")
    }
}
