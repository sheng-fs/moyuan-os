//! 飞地硬件绑定模块

use super::error::*;
use super::core::*;
use super::iommu::*;

/// 硬件资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareResourceType {
    /// PCI设备
    PciDevice,
    /// 中断控制器
    InterruptController,
    /// 图形卡
    GraphicsCard,
    /// 网络卡
    NetworkCard,
    /// 音频设备
    AudioDevice,
    /// AI加速器
    AiAccelerator,
    /// 媒体处理器
    MediaProcessor,
}

/// 硬件资源
#[derive(Debug, Clone, Copy)]
pub struct HardwareResource {
    /// 资源类型
    pub resource_type: HardwareResourceType,
    /// PCI设备ID（如果是PCI设备）
    pub pci_device_id: Option<u16>,
    /// 基地址
    pub base_address: usize,
    /// 大小
    pub size: usize,
    /// 中断向量
    pub interrupt_vector: Option<u8>,
}

impl HardwareResource {
    /// 创建硬件资源
    pub fn new(
        resource_type: HardwareResourceType,
        pci_device_id: Option<u16>,
        base_address: usize,
        size: usize,
        interrupt_vector: Option<u8>,
    ) -> Self {
        Self {
            resource_type,
            pci_device_id,
            base_address,
            size,
            interrupt_vector,
        }
    }
}

/// 硬件绑定管理器
pub struct HardwareBindManager;

impl HardwareBindManager {
    /// 绑定硬件到飞地
    pub fn bind_hardware(
        enclave: &mut Enclave,
        resource: HardwareResource,
    ) -> Result<(), EnclaveError> {
        crate::println!(
            "Binding hardware resource {:?} to enclave {}",
            resource,
            enclave.id()
        );
        
        // 检查飞地是否有硬件访问权限
        if !enclave.permissions().contains(EnclavePermissions::HARDWARE_ACCESS) {
            return Err(EnclaveError::PermissionDenied);
        }
        
        // 获取飞地的IOMMU域
        let domain = match enclave.iommu_domain_mut() {
            Some(d) => d,
            None => return Err(EnclaveError::EnclaveNotInitialized),
        };
        
        // 如果是PCI设备，附加到IOMMU域
        if let Some(device_id) = resource.pci_device_id {
            domain.attach_device(device_id).map_err(|e| EnclaveError::IommuError(e))?;
        }
        
        // 添加硬件资源到飞地
        enclave.add_hardware_resource(resource);
        
        Ok(())
    }
    
    /// 解绑硬件从飞地
    pub fn unbind_hardware(
        enclave: &mut Enclave,
        resource: HardwareResource,
    ) -> Result<(), EnclaveError> {
        crate::println!(
            "Unbinding hardware resource {:?} from enclave {}",
            resource,
            enclave.id()
        );
        
        // 获取飞地的IOMMU域
        let domain = match enclave.iommu_domain_mut() {
            Some(d) => d,
            None => return Err(EnclaveError::EnclaveNotInitialized),
        };
        
        // 如果是PCI设备，从IOMMU域分离
        if let Some(device_id) = resource.pci_device_id {
            domain.detach_device(device_id).map_err(|e| EnclaveError::IommuError(e))?;
        }
        
        Ok(())
    }
    
    /// 为图形飞地分配默认硬件
    pub fn assign_graphic_hardware(enclave: &mut Enclave) -> Result<(), EnclaveError> {
        crate::println!("Assigning graphic hardware to enclave {}", enclave.id());
        
        // 模拟显卡资源
        let gpu_resource = HardwareResource::new(
            HardwareResourceType::GraphicsCard,
            Some(0x0100), // 虚拟PCI设备ID
            0xD0000000,   // 虚拟基地址
            0x10000000,   // 256MB
            Some(16),     // 中断向量
        );
        
        Self::bind_hardware(enclave, gpu_resource)?;
        
        Ok(())
    }
    
    /// 为AI飞地分配默认硬件
    pub fn assign_ai_hardware(enclave: &mut Enclave) -> Result<(), EnclaveError> {
        crate::println!("Assigning AI hardware to enclave {}", enclave.id());
        
        // 模拟AI加速器资源
        let ai_resource = HardwareResource::new(
            HardwareResourceType::AiAccelerator,
            Some(0x0200), // 虚拟PCI设备ID
            0xE0000000,   // 虚拟基地址
            0x20000000,   // 512MB
            Some(17),     // 中断向量
        );
        
        Self::bind_hardware(enclave, ai_resource)?;
        
        Ok(())
    }
    
    /// 为媒体飞地分配默认硬件
    pub fn assign_media_hardware(enclave: &mut Enclave) -> Result<(), EnclaveError> {
        crate::println!("Assigning media hardware to enclave {}", enclave.id());
        
        // 模拟媒体处理器资源
        let media_resource = HardwareResource::new(
            HardwareResourceType::MediaProcessor,
            Some(0x0300), // 虚拟PCI设备ID
            0xC0000000,   // 虚拟基地址
            0x08000000,   // 128MB
            Some(18),     // 中断向量
        );
        
        Self::bind_hardware(enclave, media_resource)?;
        
        Ok(())
    }
    
    /// 为网络飞地分配默认硬件
    pub fn assign_network_hardware(enclave: &mut Enclave) -> Result<(), EnclaveError> {
        crate::println!("Assigning network hardware to enclave {}", enclave.id());
        
        // 模拟网卡资源
        let nic_resource = HardwareResource::new(
            HardwareResourceType::NetworkCard,
            Some(0x0400), // 虚拟PCI设备ID
            0xB0000000,   // 虚拟基地址
            0x01000000,   // 16MB
            Some(19),     // 中断向量
        );
        
        Self::bind_hardware(enclave, nic_resource)?;
        
        Ok(())
    }
    
    /// 根据飞地类型分配硬件
    pub fn assign_hardware_by_type(enclave: &mut Enclave) -> Result<(), EnclaveError> {
        match enclave.enclave_type() {
            EnclaveType::GraphicEnclave => Self::assign_graphic_hardware(enclave),
            EnclaveType::AIEnclave => Self::assign_ai_hardware(enclave),
            EnclaveType::MediaEnclave => Self::assign_media_hardware(enclave),
            EnclaveType::NetworkEnclave => Self::assign_network_hardware(enclave),
        }
    }
}
