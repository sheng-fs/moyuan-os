//! 飞地错误处理模块

use core::fmt;

/// 飞地错误类型
#[derive(Debug)]
pub enum EnclaveError {
    /// IOMMU错误
    IommuError(IommuError),
    /// 内存分配失败
    MemoryAllocationFailed,
    /// 内存绑定失败
    MemoryBindFailed,
    /// 硬件资源不足
    InsufficientHardwareResources,
    /// ACPI DMAR表未找到
    DmarTableNotFound,
    /// IOMMU硬件未找到
    IommuHardwareNotFound,
    /// 无效的飞地类型
    InvalidEnclaveType,
    /// 无效的飞地ID
    InvalidEnclaveId,
    /// 飞地已存在
    EnclaveAlreadyExists,
    /// 飞地未初始化
    EnclaveNotInitialized,
    /// 权限错误
    PermissionDenied,
    /// 无效状态
    InvalidState,
    /// 硬件绑定失败
    HardwareBindFailed,
    /// 设备附加失败
    DeviceAttachFailed,
}

impl fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnclaveError::IommuError(err) => write!(f, "IOMMU error: {}", err),
            EnclaveError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            EnclaveError::MemoryBindFailed => write!(f, "Memory bind failed"),
            EnclaveError::InsufficientHardwareResources => write!(f, "Insufficient hardware resources"),
            EnclaveError::DmarTableNotFound => write!(f, "DMAR table not found"),
            EnclaveError::IommuHardwareNotFound => write!(f, "IOMMU hardware not found"),
            EnclaveError::InvalidEnclaveType => write!(f, "Invalid enclave type"),
            EnclaveError::InvalidEnclaveId => write!(f, "Invalid enclave ID"),
            EnclaveError::EnclaveAlreadyExists => write!(f, "Enclave already exists"),
            EnclaveError::EnclaveNotInitialized => write!(f, "Enclave not initialized"),
            EnclaveError::PermissionDenied => write!(f, "Permission denied"),
            EnclaveError::InvalidState => write!(f, "Invalid state"),
            EnclaveError::HardwareBindFailed => write!(f, "Hardware bind failed"),
            EnclaveError::DeviceAttachFailed => write!(f, "Device attach failed"),
        }
    }
}

/// IOMMU错误类型
#[derive(Debug)]
pub enum IommuError {
    /// 初始化失败
    InitFailed,
    /// 设备附加失败
    DeviceAttachFailed,
    /// 设备分离失败
    DeviceDetachFailed,
    /// 内存映射失败
    MemoryMapFailed,
    /// 根表地址无效
    InvalidRootTableAddress,
    /// 上下文表地址无效
    InvalidContextTableAddress,
    /// 权限错误
    PermissionDenied,
    /// 硬件错误
    HardwareError,
}

impl fmt::Display for IommuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IommuError::InitFailed => write!(f, "IOMMU initialization failed"),
            IommuError::DeviceAttachFailed => write!(f, "Device attach failed"),
            IommuError::DeviceDetachFailed => write!(f, "Device detach failed"),
            IommuError::MemoryMapFailed => write!(f, "Memory map failed"),
            IommuError::InvalidRootTableAddress => write!(f, "Invalid root table address"),
            IommuError::InvalidContextTableAddress => write!(f, "Invalid context table address"),
            IommuError::PermissionDenied => write!(f, "Permission denied"),
            IommuError::HardwareError => write!(f, "Hardware error"),
        }
    }
}
