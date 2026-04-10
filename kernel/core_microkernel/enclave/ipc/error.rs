//! 飞地IPC错误类型

use core::fmt;

/// 飞地IPC错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnclaveIpcError {
    /// 初始化失败
    InitFailed,
    /// 内存分配失败
    MemoryAllocationFailed,
    /// 内存释放失败
    MemoryFreeFailed,
    /// 密钥生成失败
    KeyGenerationFailed,
    /// 密钥销毁失败
    KeyDestructionFailed,
    /// 加密失败
    EncryptionFailed,
    /// 解密失败
    DecryptionFailed,
    /// 认证标签验证失败
    TagVerificationFailed,
    /// 序列号验证失败
    SequenceNumberVerificationFailed,
    /// 通道创建失败
    ChannelCreationFailed,
    /// 通道不存在
    ChannelNotFound,
    /// 通道已关闭
    ChannelClosed,
    /// 无效的发送方
    InvalidSender,
    /// 无效的接收方
    InvalidReceiver,
    /// 通信权限不足
    PermissionDenied,
    /// IOMMU配置失败
    IommuConfigurationFailed,
    /// 数据块大小超过限制
    BlockSizeTooLarge,
    /// 共享内存已满
    SharedMemoryFull,
    /// 共享内存为空
    SharedMemoryEmpty,
    /// 数据完整性验证失败
    IntegrityCheckFailed,
    /// 重放攻击检测
    ReplayAttackDetected,
    /// 硬件加速不可用
    HardwareAccelerationUnavailable,
}

impl fmt::Display for EnclaveIpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnclaveIpcError::InitFailed => write!(f, "Enclave IPC initialization failed"),
            EnclaveIpcError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            EnclaveIpcError::MemoryFreeFailed => write!(f, "Memory free failed"),
            EnclaveIpcError::KeyGenerationFailed => write!(f, "Key generation failed"),
            EnclaveIpcError::KeyDestructionFailed => write!(f, "Key destruction failed"),
            EnclaveIpcError::EncryptionFailed => write!(f, "Encryption failed"),
            EnclaveIpcError::DecryptionFailed => write!(f, "Decryption failed"),
            EnclaveIpcError::TagVerificationFailed => write!(f, "Authentication tag verification failed"),
            EnclaveIpcError::SequenceNumberVerificationFailed => write!(f, "Sequence number verification failed"),
            EnclaveIpcError::ChannelCreationFailed => write!(f, "Channel creation failed"),
            EnclaveIpcError::ChannelNotFound => write!(f, "Channel not found"),
            EnclaveIpcError::ChannelClosed => write!(f, "Channel closed"),
            EnclaveIpcError::InvalidSender => write!(f, "Invalid sender"),
            EnclaveIpcError::InvalidReceiver => write!(f, "Invalid receiver"),
            EnclaveIpcError::PermissionDenied => write!(f, "Permission denied"),
            EnclaveIpcError::IommuConfigurationFailed => write!(f, "IOMMU configuration failed"),
            EnclaveIpcError::BlockSizeTooLarge => write!(f, "Block size too large"),
            EnclaveIpcError::SharedMemoryFull => write!(f, "Shared memory full"),
            EnclaveIpcError::SharedMemoryEmpty => write!(f, "Shared memory empty"),
            EnclaveIpcError::IntegrityCheckFailed => write!(f, "Integrity check failed"),
            EnclaveIpcError::ReplayAttackDetected => write!(f, "Replay attack detected"),
            EnclaveIpcError::HardwareAccelerationUnavailable => write!(f, "Hardware acceleration unavailable"),
        }
    }
}

/// IPC内存错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcMemoryError {
    /// 分配失败
    AllocationFailed,
    /// 释放失败
    FreeFailed,
    /// 内存块不存在
    BlockNotFound,
    /// 内存块已满
    BlockFull,
    /// 内存块为空
    BlockEmpty,
    /// 无效的内存地址
    InvalidAddress,
}

impl From<IpcMemoryError> for EnclaveIpcError {
    fn from(err: IpcMemoryError) -> Self {
        match err {
            IpcMemoryError::AllocationFailed => EnclaveIpcError::MemoryAllocationFailed,
            IpcMemoryError::FreeFailed => EnclaveIpcError::MemoryFreeFailed,
            IpcMemoryError::BlockNotFound => EnclaveIpcError::ChannelNotFound,
            IpcMemoryError::BlockFull => EnclaveIpcError::SharedMemoryFull,
            IpcMemoryError::BlockEmpty => EnclaveIpcError::SharedMemoryEmpty,
            IpcMemoryError::InvalidAddress => EnclaveIpcError::IntegrityCheckFailed,
        }
    }
}

/// 加密错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionError {
    /// 密钥生成失败
    KeyGenerationFailed,
    /// 密钥销毁失败
    KeyDestructionFailed,
    /// 加密失败
    EncryptionFailed,
    /// 解密失败
    DecryptionFailed,
    /// 认证失败
    AuthenticationFailed,
    /// 无效密钥
    InvalidKey,
}

impl From<EncryptionError> for EnclaveIpcError {
    fn from(err: EncryptionError) -> Self {
        match err {
            EncryptionError::KeyGenerationFailed => EnclaveIpcError::KeyGenerationFailed,
            EncryptionError::KeyDestructionFailed => EnclaveIpcError::KeyDestructionFailed,
            EncryptionError::EncryptionFailed => EnclaveIpcError::EncryptionFailed,
            EncryptionError::DecryptionFailed => EnclaveIpcError::DecryptionFailed,
            EncryptionError::AuthenticationFailed => EnclaveIpcError::TagVerificationFailed,
            EncryptionError::InvalidKey => EnclaveIpcError::IntegrityCheckFailed,
        }
    }
}

/// 通道错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelError {
    /// 创建失败
    CreationFailed,
    /// 不存在
    NotFound,
    /// 已关闭
    Closed,
    /// 无效发送方
    InvalidSender,
    /// 无效接收方
    InvalidReceiver,
}

impl From<ChannelError> for EnclaveIpcError {
    fn from(err: ChannelError) -> Self {
        match err {
            ChannelError::CreationFailed => EnclaveIpcError::ChannelCreationFailed,
            ChannelError::NotFound => EnclaveIpcError::ChannelNotFound,
            ChannelError::Closed => EnclaveIpcError::ChannelClosed,
            ChannelError::InvalidSender => EnclaveIpcError::InvalidSender,
            ChannelError::InvalidReceiver => EnclaveIpcError::InvalidReceiver,
        }
    }
}

/// 安全错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    /// 权限不足
    PermissionDenied,
    /// IOMMU配置失败
    IommuConfigurationFailed,
    /// 验证失败
    VerificationFailed,
}

impl From<SecurityError> for EnclaveIpcError {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::PermissionDenied => EnclaveIpcError::PermissionDenied,
            SecurityError::IommuConfigurationFailed => EnclaveIpcError::IommuConfigurationFailed,
            SecurityError::VerificationFailed => EnclaveIpcError::IntegrityCheckFailed,
        }
    }
}
