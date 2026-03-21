//! 飞地核心逻辑模块

use super::error::*;
use super::memory::*;
use super::iommu::*;
use super::hardware::*;
use crate::memory::{PhysicalAddress, VirtualAddress};
use spin::Mutex;
use core::option::Option;

/// 飞地权限位标志
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct EnclavePermissions: u32 {
        /// 读权限
        const READ = 1 << 0;
        /// 写权限
        const WRITE = 1 << 1;
        /// 执行权限
        const EXECUTE = 1 << 2;
        /// 硬件访问权限
        const HARDWARE_ACCESS = 1 << 3;
    }
}

/// 飞地类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum EnclaveType {
    /// 图形飞地
    GraphicEnclave = 0,
    /// 人工智能飞地
    AIEnclave = 1,
    /// 媒体飞地
    MediaEnclave = 2,
    /// 网络飞地
    NetworkEnclave = 3,
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

/// 飞地核心结构体
pub struct Enclave {
    /// 全局唯一ID
    id: usize,
    /// 飞地类型
    enclave_type: EnclaveType,
    /// 权限集合
    permissions: EnclavePermissions,
    /// 专属IOMMU域
    iommu_domain: Option<IommuDomain>,
    /// 内存区域列表
    memory_regions: Vec<EnclaveMemoryRegion>,
    /// 硬件资源列表
    hardware_resources: Vec<HardwareResource>,
    /// 飞地状态
    state: EnclaveState,
}

impl Enclave {
    /// 创建新飞地
    pub fn new(
        id: usize,
        enclave_type: EnclaveType,
        permissions: EnclavePermissions,
    ) -> Result<Self, EnclaveError> {
        Ok(Self {
            id,
            enclave_type,
            permissions,
            iommu_domain: None,
            memory_regions: Vec::new(),
            hardware_resources: Vec::new(),
            state: EnclaveState::Uninitialized,
        })
    }
    
    /// 获取飞地ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// 获取飞地类型
    pub fn enclave_type(&self) -> EnclaveType {
        self.enclave_type
    }
    
    /// 获取权限集合
    pub fn permissions(&self) -> EnclavePermissions {
        self.permissions
    }
    
    /// 获取飞地状态
    pub fn state(&self) -> EnclaveState {
        self.state
    }
    
    /// 设置飞地状态
    pub fn set_state(&mut self, state: EnclaveState) {
        self.state = state;
    }
    
    /// 设置IOMMU域
    pub fn set_iommu_domain(&mut self, domain: IommuDomain) {
        self.iommu_domain = Some(domain);
    }
    
    /// 获取IOMMU域
    pub fn iommu_domain(&self) -> Option<&IommuDomain> {
        self.iommu_domain.as_ref()
    }
    
    /// 获取可变IOMMU域
    pub fn iommu_domain_mut(&mut self) -> Option<&mut IommuDomain> {
        self.iommu_domain.as_mut()
    }
    
    /// 添加内存区域
    pub fn add_memory_region(&mut self, region: EnclaveMemoryRegion) {
        self.memory_regions.push(region);
    }
    
    /// 获取内存区域列表
    pub fn memory_regions(&self) -> &[EnclaveMemoryRegion] {
        &self.memory_regions
    }
    
    /// 添加硬件资源
    pub fn add_hardware_resource(&mut self, resource: HardwareResource) {
        self.hardware_resources.push(resource);
    }
    
    /// 获取硬件资源列表
    pub fn hardware_resources(&self) -> &[HardwareResource] {
        &self.hardware_resources
    }
    
    /// 检查内存访问权限
    pub fn check_memory_access(&self, addr: PhysicalAddress, size: usize) -> bool {
        for region in &self.memory_regions {
            if region.contains(addr, size) {
                return true;
            }
        }
        false
    }
}

/// 全局飞地管理器
pub struct EnclaveManager {
    /// 图形飞地占位实例
    graphic_enclave: Option<Enclave>,
    /// AI飞地占位实例
    ai_enclave: Option<Enclave>,
    /// 媒体飞地占位实例
    media_enclave: Option<Enclave>,
    /// 网络飞地占位实例
    network_enclave: Option<Enclave>,
    /// 下一个飞地ID
    next_id: usize,
}

impl EnclaveManager {
    /// 创建飞地管理器
    pub fn new() -> Self {
        Self {
            graphic_enclave: None,
            ai_enclave: None,
            media_enclave: None,
            network_enclave: None,
            next_id: 0,
        }
    }
    
    /// 获取下一个飞地ID
    fn next_enclave_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// 创建图形飞地
    pub fn create_graphic_enclave(&mut self) -> Result<usize, EnclaveError> {
        if self.graphic_enclave.is_some() {
            return Err(EnclaveError::EnclaveAlreadyExists);
        }
        
        let id = self.next_enclave_id();
        let permissions = EnclavePermissions::READ | EnclavePermissions::WRITE | EnclavePermissions::HARDWARE_ACCESS;
        let enclave = Enclave::new(id, EnclaveType::GraphicEnclave, permissions)?;
        self.graphic_enclave = Some(enclave);
        Ok(id)
    }
    
    /// 创建AI飞地
    pub fn create_ai_enclave(&mut self) -> Result<usize, EnclaveError> {
        if self.ai_enclave.is_some() {
            return Err(EnclaveError::EnclaveAlreadyExists);
        }
        
        let id = self.next_enclave_id();
        let permissions = EnclavePermissions::READ | EnclavePermissions::WRITE | EnclavePermissions::HARDWARE_ACCESS;
        let enclave = Enclave::new(id, EnclaveType::AIEnclave, permissions)?;
        self.ai_enclave = Some(enclave);
        Ok(id)
    }
    
    /// 创建媒体飞地
    pub fn create_media_enclave(&mut self) -> Result<usize, EnclaveError> {
        if self.media_enclave.is_some() {
            return Err(EnclaveError::EnclaveAlreadyExists);
        }
        
        let id = self.next_enclave_id();
        let permissions = EnclavePermissions::READ | EnclavePermissions::WRITE | EnclavePermissions::HARDWARE_ACCESS;
        let enclave = Enclave::new(id, EnclaveType::MediaEnclave, permissions)?;
        self.media_enclave = Some(enclave);
        Ok(id)
    }
    
    /// 创建网络飞地
    pub fn create_network_enclave(&mut self) -> Result<usize, EnclaveError> {
        if self.network_enclave.is_some() {
            return Err(EnclaveError::EnclaveAlreadyExists);
        }
        
        let id = self.next_enclave_id();
        let permissions = EnclavePermissions::READ | EnclavePermissions::WRITE | EnclavePermissions::HARDWARE_ACCESS;
        let enclave = Enclave::new(id, EnclaveType::NetworkEnclave, permissions)?;
        self.network_enclave = Some(enclave);
        Ok(id)
    }
    
    /// 获取飞地
    pub fn get_enclave(&mut self, enclave_type: EnclaveType) -> Option<&mut Enclave> {
        match enclave_type {
            EnclaveType::GraphicEnclave => self.graphic_enclave.as_mut(),
            EnclaveType::AIEnclave => self.ai_enclave.as_mut(),
            EnclaveType::MediaEnclave => self.media_enclave.as_mut(),
            EnclaveType::NetworkEnclave => self.network_enclave.as_mut(),
        }
    }
    
    /// 获取飞地(只读)
    pub fn get_enclave_ref(&self, enclave_type: EnclaveType) -> Option<&Enclave> {
        match enclave_type {
            EnclaveType::GraphicEnclave => self.graphic_enclave.as_ref(),
            EnclaveType::AIEnclave => self.ai_enclave.as_ref(),
            EnclaveType::MediaEnclave => self.media_enclave.as_ref(),
            EnclaveType::NetworkEnclave => self.network_enclave.as_ref(),
        }
    }
    
    /// 销毁飞地
    pub fn destroy_enclave(&mut self, enclave_type: EnclaveType) -> Result<(), EnclaveError> {
        let enclave_opt = match enclave_type {
            EnclaveType::GraphicEnclave => &mut self.graphic_enclave,
            EnclaveType::AIEnclave => &mut self.ai_enclave,
            EnclaveType::MediaEnclave => &mut self.media_enclave,
            EnclaveType::NetworkEnclave => &mut self.network_enclave,
        };
        
        if let Some(mut enclave) = enclave_opt.take() {
            enclave.set_state(EnclaveState::Destroyed);
            Ok(())
        } else {
            Err(EnclaveError::InvalidEnclaveId)
        }
    }
}

/// 全局飞地管理器实例
static mut ENCLAVE_MANAGER: Option<Mutex<EnclaveManager>> = None;

/// 初始化飞地核心
pub fn init_enclave_core() -> Result<(), EnclaveError> {
    unsafe {
        ENCLAVE_MANAGER = Some(Mutex::new(EnclaveManager::new()));
    }
    Ok(())
}

/// 获取飞地管理器
pub fn get_enclave_manager() -> &'static Mutex<EnclaveManager> {
    unsafe {
        ENCLAVE_MANAGER.as_ref().expect("Enclave manager not initialized")
    }
}

/// 创建飞地
pub fn enclave_create(enclave_type: EnclaveType) -> Result<usize, EnclaveError> {
    let mut manager = get_enclave_manager().lock();
    match enclave_type {
        EnclaveType::GraphicEnclave => manager.create_graphic_enclave(),
        EnclaveType::AIEnclave => manager.create_ai_enclave(),
        EnclaveType::MediaEnclave => manager.create_media_enclave(),
        EnclaveType::NetworkEnclave => manager.create_network_enclave(),
    }
}

/// 初始化飞地
pub fn enclave_initialize(enclave_type: EnclaveType) -> Result<(), EnclaveError> {
    let mut manager = get_enclave_manager().lock();
    if let Some(enclave) = manager.get_enclave(enclave_type) {
        if enclave.state() == EnclaveState::Uninitialized {
            enclave.set_state(EnclaveState::Running);
            Ok(())
        } else {
            Err(EnclaveError::InvalidState)
        }
    } else {
        Err(EnclaveError::InvalidEnclaveType)
    }
}

/// 销毁飞地
pub fn enclave_destroy(enclave_type: EnclaveType) -> Result<(), EnclaveError> {
    let mut manager = get_enclave_manager().lock();
    manager.destroy_enclave(enclave_type)
}
