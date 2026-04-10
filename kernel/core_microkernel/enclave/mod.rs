//! 飞地模块 - 核心入口
//!
//! 实现基于 VT-d 技术的硬件级隔离，支持图形、AI、媒体和网络飞地

pub mod error;
pub mod iommu;
pub mod memory;
pub mod hardware;
pub mod core;
pub mod ipc;

pub use self::core::*;
pub use self::error::*;
pub use self::iommu::*;
pub use self::memory::*;
pub use self::hardware::*;
pub use self::ipc::*;

/// 初始化飞地子系统
pub fn init() -> Result<(), EnclaveError> {
    crate::println!("Initializing enclave subsystem...");

    // 初始化 IOMMU
    iommu::init_iommu()?;

    // 初始化飞地核心
    core::init_enclave_core()?;

    // 初始化飞地IPC系统
    ipc::init_enclave_ipc().map_err(|_| EnclaveError::InitFailed)?;

    crate::println!("Enclave subsystem initialized successfully");
    Ok(())
}
