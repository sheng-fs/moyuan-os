//! 飞地模块
//! 
//! 实现基于 VT-d 技术的硬件级隔离，支持图形、AI、媒体和网络飞地

use crate::arch::x86_64::iommu::{IommuDomain, IommuError};
use crate::memory::{PhysicalAddress, VirtualAddress};
use core::fmt;

/// 飞地类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnclaveType {
    /// 图形飞地
    GraphicEnclave,
    /// 人工智能飞地
    AIEnclave,
    /// 媒体飞地
    MediaEnclave,
    /// 网络飞地
    NetworkEnclave,
}

/// 飞地状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnclaveState {
    /// 未初始化
    Uninitialized,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 已销毁
    Destroyed,
}

/// 飞地资源需求
#[derive(Debug, Clone, Copy)]
pub struct EnclaveResources {
    /// 内存大小（字节）
    pub memory_size: usize,
    /// CPU 核心数
    pub cpu_cores: usize,
    /// 所需硬件设备 ID
    pub device_ids: &'static [u16],
}

/// 飞地元数据
#[derive(Debug)]
pub struct EnclaveMetadata {
    /// 飞地类型
    pub enclave_type: EnclaveType,
    /// 飞地 ID
    pub id: usize,
    /// 飞地状态
    pub state: EnclaveState,
    /// 资源需求
    pub resources: EnclaveResources,
    /// 基地址
    pub base_address: PhysicalAddress,
    /// IOMMU 域
    pub iommu_domain: Option<IommuDomain>,
}

/// 飞地错误
#[derive(Debug)]
pub enum EnclaveError {
    /// 内存分配失败
    MemoryAllocationFailed,
    /// IOMMU 初始化失败
    IommuInitFailed(IommuError),
    /// 设备分配失败
    DeviceAllocationFailed,
    /// 飞地状态错误
    InvalidState,
    /// 权限错误
    PermissionDenied,
}

impl fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EnclaveError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            EnclaveError::IommuInitFailed(err) => write!(f, "IOMMU initialization failed: {:?}", err),
            EnclaveError::DeviceAllocationFailed => write!(f, "Device allocation failed"),
            EnclaveError::InvalidState => write!(f, "Invalid enclave state"),
            EnclaveError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

/// 飞地结构
pub struct Enclave {
    /// 元数据
    metadata: EnclaveMetadata,
    /// 内存映射表
    memory_map: Vec<(PhysicalAddress, VirtualAddress, usize)>,
    /// 设备列表
    devices: Vec<u16>,
}

impl Enclave {
    /// 创建新飞地
    pub fn new(enclave_type: EnclaveType, id: usize, resources: EnclaveResources) -> Result<Self, EnclaveError> {
        // 1. 分配内存
        let base_address = Self::allocate_memory(resources.memory_size)?;
        
        // 2. 初始化 IOMMU 域
        let iommu_domain = IommuDomain::new().map_err(EnclaveError::IommuInitFailed)?;
        
        // 3. 分配设备
        let devices = Self::allocate_devices(&resources.device_ids)?;
        
        // 4. 配置 IOMMU
        for &device_id in &devices {
            iommu_domain.attach_device(device_id).map_err(EnclaveError::IommuInitFailed)?;
        }
        
        let metadata = EnclaveMetadata {
            enclave_type,
            id,
            state: EnclaveState::Uninitialized,
            resources,
            base_address,
            iommu_domain: Some(iommu_domain),
        };
        
        Ok(Self {
            metadata,
            memory_map: Vec::new(),
            devices,
        })
    }
    
    /// 分配内存
    fn allocate_memory(size: usize) -> Result<PhysicalAddress, EnclaveError> {
        // 这里实现内存分配逻辑
        // 实际实现需要调用内存管理器
        Ok(PhysicalAddress::new(0x100000000)) // 临时返回一个地址
    }
    
    /// 分配设备
    fn allocate_devices(device_ids: &[u16]) -> Result<Vec<u16>, EnclaveError> {
        // 这里实现设备分配逻辑
        // 实际实现需要检查设备是否可用
        Ok(device_ids.to_vec())
    }
    
    /// 初始化飞地
    pub fn initialize(&mut self) -> Result<(), EnclaveError> {
        if self.metadata.state != EnclaveState::Uninitialized {
            return Err(EnclaveError::InvalidState);
        }
        
        // 初始化飞地内部状态
        // 配置内存保护
        // 配置设备访问权限
        
        self.metadata.state = EnclaveState::Running;
        Ok(())
    }
    
    /// 暂停飞地
    pub fn pause(&mut self) -> Result<(), EnclaveError> {
        if self.metadata.state != EnclaveState::Running {
            return Err(EnclaveError::InvalidState);
        }
        
        // 暂停飞地执行
        // 保存状态
        
        self.metadata.state = EnclaveState::Paused;
        Ok(())
    }
    
    /// 恢复飞地
    pub fn resume(&mut self) -> Result<(), EnclaveError> {
        if self.metadata.state != EnclaveState::Paused {
            return Err(EnclaveError::InvalidState);
        }
        
        // 恢复飞地执行
        // 加载状态
        
        self.metadata.state = EnclaveState::Running;
        Ok(())
    }
    
    /// 销毁飞地
    pub fn destroy(&mut self) -> Result<(), EnclaveError> {
        if self.metadata.state == EnclaveState::Destroyed {
            return Err(EnclaveError::InvalidState);
        }
        
        // 释放设备
        for &device_id in &self.devices {
            if let Some(ref mut domain) = self.metadata.iommu_domain {
                domain.detach_device(device_id).ok();
            }
        }
        
        // 释放内存
        // 清理 IOMMU 域
        self.metadata.iommu_domain = None;
        
        self.metadata.state = EnclaveState::Destroyed;
        Ok(())
    }
    
    /// 获取飞地元数据
    pub fn metadata(&self) -> &EnclaveMetadata {
        &self.metadata
    }
    
    /// 检查内存访问权限
    pub fn check_memory_access(&self, addr: PhysicalAddress, size: usize) -> bool {
        // 检查内存是否在飞地范围内
        // 实际实现需要更复杂的权限检查
        addr >= self.metadata.base_address && 
        addr < self.metadata.base_address + self.metadata.resources.memory_size
    }
    
    /// 检查设备访问权限
    pub fn check_device_access(&self, device_id: u16) -> bool {
        // 检查设备是否属于飞地
        self.devices.contains(&device_id)
    }
}

/// 飞地管理器
pub struct EnclaveManager {
    enclaves: Vec<Enclave>,
    next_id: usize,
}

impl EnclaveManager {
    /// 创建飞地管理器
    pub fn new() -> Self {
        Self {
            enclaves: Vec::new(),
            next_id: 0,
        }
    }
    
    /// 创建飞地
    pub fn create_enclave(&mut self, enclave_type: EnclaveType, resources: EnclaveResources) -> Result<usize, EnclaveError> {
        let id = self.next_id;
        self.next_id += 1;
        
        let enclave = Enclave::new(enclave_type, id, resources)?;
        self.enclaves.push(enclave);
        
        Ok(id)
    }
    
    /// 获取飞地
    pub fn get_enclave(&mut self, id: usize) -> Option<&mut Enclave> {
        self.enclaves.iter_mut().find(|e| e.metadata().id == id)
    }
    
    /// 销毁飞地
    pub fn destroy_enclave(&mut self, id: usize) -> Result<(), EnclaveError> {
        if let Some((index, enclave)) = self.enclaves.iter_mut().enumerate().find(|(_, e)| e.metadata().id == id) {
            enclave.destroy()?;
            self.enclaves.remove(index);
            Ok(())
        } else {
            Err(EnclaveError::InvalidState)
        }
    }
    
    /// 列出所有飞地
    pub fn list_enclaves(&self) -> Vec<&EnclaveMetadata> {
        self.enclaves.iter().map(|e| e.metadata()).collect()
    }
}

/// 全局飞地管理器实例
static mut ENCLAVE_MANAGER: Option<EnclaveManager> = None;

/// 初始化飞地管理器
pub fn init_enclave_manager() {
    unsafe {
        ENCLAVE_MANAGER = Some(EnclaveManager::new());
    }
}

/// 获取飞地管理器
pub fn get_enclave_manager() -> &'static mut EnclaveManager {
    unsafe {
        ENCLAVE_MANAGER.as_mut().expect("Enclave manager not initialized")
    }
}

/// 创建飞地
pub fn enclave_create(enclave_type: EnclaveType, resources: EnclaveResources) -> Result<usize, EnclaveError> {
    get_enclave_manager().create_enclave(enclave_type, resources)
}

/// 初始化飞地
pub fn enclave_initialize(id: usize) -> Result<(), EnclaveError> {
    if let Some(enclave) = get_enclave_manager().get_enclave(id) {
        enclave.initialize()
    } else {
        Err(EnclaveError::InvalidState)
    }
}

/// 销毁飞地
pub fn enclave_destroy(id: usize) -> Result<(), EnclaveError> {
    get_enclave_manager().destroy_enclave(id)
}