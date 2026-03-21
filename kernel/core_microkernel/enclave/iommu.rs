//! IOMMU硬件操作模块

use super::error::*;
use crate::memory::PhysicalAddress;
use crate::power::acpi::*;
use core::ptr;

/// IOMMU寄存器偏移
const IOMMU_CAP_OFFSET: usize = 0x08;
const IOMMU_GCMD_OFFSET: usize = 0x18;
const IOMMU_GSTS_OFFSET: usize = 0x1c;
const IOMMU_RTADDR_OFFSET: usize = 0x20;
const IOMMU_CCMD_OFFSET: usize = 0x28;
const IOMMU_FSTS_OFFSET: usize = 0x34;
const IOMMU_FECTL_OFFSET: usize = 0x38;
const IOMMU_PMEN_OFFSET: usize = 0x64;
const IOMMU_IVA_OFFSET: usize = 0x80;

/// IOMMU命令
const IOMMU_GCMD_TE: u32 = 1 << 31; // 翻译使能
const IOMMU_GCMD_SRTP: u32 = 1 << 30; // 设置根表指针
const IOMMU_GCMD_CFI: u32 = 1 << 23; // 清除故障中断
const IOMMU_GCMD_IR: u32 = 1 << 25; // 中断重映射使能

/// IOMMU状态
const IOMMU_GSTS_TES: u32 = 1 << 31; // 翻译使能状态
const IOMMU_GSTS_RTPS: u32 = 1 << 30; // 根表指针状态

/// DMAR表结构
#[repr(C)]
#[derive(Debug)]
pub struct DmarTable {
    header: SdtHeader,
    host_address_width: u8,
    flags: u8,
    reserved: [u8; 10],
}

/// 重映射硬件单元结构
#[repr(C)]
#[derive(Debug)]
pub struct RemappingHardwareUnit {
    type_: u16,
    length: u16,
    flags: u8,
    reserved: u8,
    segment_number: u16,
    register_base_address: u64,
}

/// IOMMU硬件单元信息
#[derive(Debug)]
pub struct IommuHardwareUnit {
    /// 寄存器基地址
    pub base_address: PhysicalAddress,
    /// 支持的域数量
    pub max_domains: usize,
    /// 段号
    pub segment_number: u16,
}

/// IOMMU域
#[derive(Debug)]
pub struct IommuDomain {
    /// 域ID
    domain_id: usize,
    /// 根表物理地址
    root_table: PhysicalAddress,
    /// 上下文表物理地址
    context_table: PhysicalAddress,
    /// 附加的设备
    devices: Vec<u16>,
    /// 关联的IOMMU硬件单元
    hardware_unit: usize,
}

impl IommuDomain {
    /// 创建新的IOMMU域
    pub fn new(domain_id: usize, hardware_unit: usize) -> Result<Self, IommuError> {
        // 分配根表（4KB对齐）
        let root_table = allocate_aligned_page()?;
        
        // 分配上下文表（4KB对齐）
        let context_table = allocate_aligned_page()?;
        
        Ok(Self {
            domain_id,
            root_table: PhysicalAddress::new(root_table),
            context_table: PhysicalAddress::new(context_table),
            devices: Vec::new(),
            hardware_unit,
        })
    }
    
    /// 获取域ID
    pub fn domain_id(&self) -> usize {
        self.domain_id
    }
    
    /// 附加设备到域
    pub fn attach_device(&mut self, device_id: u16) -> Result<(), IommuError> {
        crate::println!("Attaching device {:x} to IOMMU domain {}", device_id, self.domain_id);
        
        // 配置上下文表
        let bus = (device_id >> 8) & 0xff;
        let devfn = device_id & 0xff;
        
        // 设置上下文表条目
        let context_entry = self.context_table.value() + (bus as usize * 8) + (devfn as usize / 8);
        
        // 简单的上下文条目（实际需要更复杂的配置）
        unsafe {
            ptr::write_volatile(context_entry as *mut u64, 1);
        }
        
        self.devices.push(device_id);
        Ok(())
    }
    
    /// 从域分离设备
    pub fn detach_device(&mut self, device_id: u16) -> Result<(), IommuError> {
        crate::println!("Detaching device {:x} from IOMMU domain {}", device_id, self.domain_id);
        
        if let Some(pos) = self.devices.iter().position(|&id| id == device_id) {
            self.devices.remove(pos);
            Ok(())
        } else {
            Err(IommuError::DeviceDetachFailed)
        }
    }
    
    /// 映射内存
    pub fn map_memory(
        &mut self,
        physical_addr: PhysicalAddress,
        virtual_addr: PhysicalAddress,
        size: usize,
        writable: bool,
    ) -> Result<(), IommuError> {
        crate::println!("Mapping memory: PA={:?}, VA={:?}, size={}", physical_addr, virtual_addr, size);
        
        // 这里需要实现IOMMU页表映射
        // 简单实现，实际需要更复杂的页表管理
        
        Ok(())
    }
    
    /// 取消映射内存
    pub fn unmap_memory(
        &mut self,
        virtual_addr: PhysicalAddress,
        size: usize,
    ) -> Result<(), IommuError> {
        crate::println!("Unmapping memory: VA={:?}, size={}", virtual_addr, size);
        
        // 这里需要实现IOMMU页表取消映射
        
        Ok(())
    }
    
    /// 获取附加的设备
    pub fn attached_devices(&self) -> &[u16] {
        &self.devices
    }
    
    /// 获取根表地址
    pub fn root_table(&self) -> PhysicalAddress {
        self.root_table
    }
}

/// IOMMU管理器
pub struct IommuManager {
    /// 硬件单元列表
    hardware_units: Vec<IommuHardwareUnit>,
    /// 域列表
    domains: Vec<IommuDomain>,
    /// 下一个域ID
    next_domain_id: usize,
}

impl IommuManager {
    /// 创建IOMMU管理器
    pub fn new() -> Self {
        Self {
            hardware_units: Vec::new(),
            domains: Vec::new(),
            next_domain_id: 0,
        }
    }
    
    /// 初始化IOMMU管理器
    pub fn init(&mut self) -> Result<(), IommuError> {
        crate::println!("Initializing IOMMU manager...");
        
        // 解析ACPI DMAR表
        self.parse_dmar_table()?;
        
        // 初始化每个IOMMU硬件单元
        for (i, unit) in self.hardware_units.iter().enumerate() {
            crate::println!("Initializing IOMMU unit {} at {:?}", i, unit.base_address);
            self.init_iommu_unit(unit)?;
        }
        
        Ok(())
    }
    
    /// 解析ACPI DMAR表
    fn parse_dmar_table(&mut self) -> Result<(), IommuError> {
        crate::println!("Parsing ACPI DMAR table...");
        
        // 查找DMAR表
        let acpi_manager = match AcpiManager::new() {
            Ok(mgr) => mgr,
            Err(_) => return Err(IommuError::InitFailed),
        };
        
        if let Some(dmar_table) = acpi_manager.find_table(b"DMAR") {
            let dmar = unsafe { &*(dmar_table as *const SdtHeader as *const DmarTable) };
            crate::println!("Found DMAR table: {:?}", dmar);
            
            // 解析重映射硬件单元
            let mut offset = core::mem::size_of::<DmarTable>();
            while offset < dmar.header.length as usize {
                let entry_ptr = (dmar as *const DmarTable as u8) as usize + offset;
                let entry_type = unsafe { ptr::read_volatile(entry_ptr as *const u16) };
                let entry_length = unsafe { ptr::read_volatile((entry_ptr + 2) as *const u16) };
                
                if entry_type == 0 { // 重映射硬件单元
                    let rhsa = unsafe { &*(entry_ptr as *const RemappingHardwareUnit) };
                    let unit = IommuHardwareUnit {
                        base_address: PhysicalAddress::new(rhsa.register_base_address as usize),
                        max_domains: 256, // 默认值
                        segment_number: rhsa.segment_number,
                    };
                    self.hardware_units.push(unit);
                    crate::println!("Found IOMMU unit: {:?}", unit);
                }
                
                offset += entry_length as usize;
            }
        } else {
            return Err(IommuError::InitFailed);
        }
        
        if self.hardware_units.is_empty() {
            return Err(IommuError::InitFailed);
        }
        
        Ok(())
    }
    
    /// 初始化IOMMU硬件单元
    fn init_iommu_unit(&self, unit: &IommuHardwareUnit) -> Result<(), IommuError> {
        let base = unit.base_address.value();
        
        // 读取能力寄存器
        let cap = unsafe { ptr::read_volatile((base + IOMMU_CAP_OFFSET) as *const u64) };
        crate::println!("IOMMU CAP: {:x}", cap);
        
        // 启用翻译
        unsafe {
            ptr::write_volatile((base + IOMMU_GCMD_OFFSET) as *mut u32, IOMMU_GCMD_TE);
        }
        
        // 等待翻译使能
        let mut timeout = 10000;
        loop {
            let gsts = unsafe { ptr::read_volatile((base + IOMMU_GSTS_OFFSET) as *const u32) };
            if (gsts & IOMMU_GSTS_TES) != 0 {
                break;
            }
            timeout -= 1;
            if timeout == 0 {
                return Err(IommuError::HardwareError);
            }
        }
        
        crate::println!("IOMMU unit initialized successfully");
        Ok(())
    }
    
    /// 分配新的IOMMU域
    pub fn allocate_domain(&mut self) -> Result<&mut IommuDomain, IommuError> {
        if self.hardware_units.is_empty() {
            return Err(IommuError::InitFailed);
        }
        
        let domain_id = self.next_domain_id;
        self.next_domain_id += 1;
        
        let domain = IommuDomain::new(domain_id, 0)?;
        self.domains.push(domain);
        
        Ok(self.domains.last_mut().unwrap())
    }
    
    /// 获取域
    pub fn get_domain(&mut self, domain_id: usize) -> Option<&mut IommuDomain> {
        self.domains.iter_mut().find(|d| d.domain_id() == domain_id)
    }
}

/// 全局IOMMU管理器
static mut IOMMU_MANAGER: Option<IommuManager> = None;

/// 初始化IOMMU子系统
pub fn init_iommu() -> Result<(), EnclaveError> {
    unsafe {
        let mut manager = IommuManager::new();
        manager.init().map_err(|e| EnclaveError::IommuError(e))?;
        IOMMU_MANAGER = Some(manager);
    }
    Ok(())
}

/// 获取IOMMU管理器
pub fn get_iommu_manager() -> &'static mut IommuManager {
    unsafe {
        IOMMU_MANAGER.as_mut().expect("IOMMU manager not initialized")
    }
}

/// 分配4KB对齐的页
fn allocate_aligned_page() -> Result<usize, IommuError> {
    // 这里需要调用内核的物理内存分配器
    // 简单实现
    Ok(0x200000) // 临时地址
}
