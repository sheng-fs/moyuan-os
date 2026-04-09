//! CPU 核心绑定与隔离模块
//!
//! 实现飞地进程的 CPU 核心预分配、亲和性设置和核心隔离功能。

use crate::enclave::core::EnclaveType;
use bitflags::bitflags;
use core::sync::atomic::{AtomicU64, Ordering};

/// CPU 核心掩码
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CpuMask: u64 {
        /// 核心 0
        const CORE_0 = 1 << 0;
        /// 核心 1
        const CORE_1 = 1 << 1;
        /// 核心 2
        const CORE_2 = 1 << 2;
        /// 核心 3
        const CORE_3 = 1 << 3;
        /// 核心 4
        const CORE_4 = 1 << 4;
        /// 核心 5
        const CORE_5 = 1 << 5;
        /// 核心 6
        const CORE_6 = 1 << 6;
        /// 核心 7
        const CORE_7 = 1 << 7;
        /// 核心 8-63 可以继续扩展
        const CORE_8 = 1 << 8;
        const CORE_9 = 1 << 9;
        const CORE_10 = 1 << 10;
        const CORE_11 = 1 << 11;
        const CORE_12 = 1 << 12;
        const CORE_13 = 1 << 13;
        const CORE_14 = 1 << 14;
        const CORE_15 = 1 << 15;
    }
}

impl CpuMask {
    /// 创建空的核心掩码
    pub fn empty() -> Self {
        CpuMask::from_bits_truncate(0)
    }

    /// 检查是否包含指定核心
    pub fn contains_core(&self, core_id: u8) -> bool {
        if core_id >= 64 {
            return false;
        }
        self.bits() & (1 << core_id) != 0
    }

    /// 添加核心
    pub fn add_core(&mut self, core_id: u8) {
        if core_id < 64 {
            *self |= CpuMask::from_bits_truncate(1 << core_id);
        }
    }

    /// 移除核心
    pub fn remove_core(&mut self, core_id: u8) {
        if core_id < 64 {
            *self &= !CpuMask::from_bits_truncate(1 << core_id);
        }
    }

    /// 获取核心数量
    pub fn core_count(&self) -> usize {
        self.bits().count_ones() as usize
    }

    /// 获取第一个可用核心
    pub fn first_core(&self) -> Option<u8> {
        for core in 0..64 {
            if self.contains_core(core) {
                return Some(core);
            }
        }
        None
    }
}

/// CPU 核心配置
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// AIEnclave 预分配的核心
    pub ai_cores: CpuMask,
    /// GraphicEnclave 预分配的核心
    pub graphic_cores: CpuMask,
    /// MediaEnclave 预分配的核心
    pub media_cores: CpuMask,
    /// NetworkEnclave 预分配的核心
    pub network_cores: CpuMask,
    /// 普通进程可用的核心
    pub normal_cores: CpuMask,
}

impl Default for CoreConfig {
    fn default() -> Self {
        // 默认 4 核心配置
        CoreConfig {
            ai_cores: CpuMask::CORE_2 | CpuMask::CORE_3,
            graphic_cores: CpuMask::CORE_1,
            media_cores: CpuMask::CORE_0,
            network_cores: CpuMask::CORE_0,
            normal_cores: CpuMask::CORE_0,
        }
    }
}

/// CPU 核心绑定管理器
pub struct CoreBindingManager {
    /// 核心配置
    config: CoreConfig,
    /// 系统总核心数
    total_cores: AtomicU64,
    /// 初始化标志
    initialized: bool,
}

impl CoreBindingManager {
    /// 创建新的核心绑定管理器
    pub fn new() -> Self {
        CoreBindingManager {
            config: CoreConfig::default(),
            total_cores: AtomicU64::new(4), // 默认 4 核心
            initialized: false,
        }
    }

    /// 初始化核心绑定管理器
    pub fn init(&mut self) {
        crate::println!("Initializing CPU Core Binding Manager...");

        // 检测 CPU 核心数
        self.detect_cpu_cores();

        // 设置默认核心配置
        self.setup_default_core_allocation();

        self.initialized = true;
        crate::println!("CPU Core Binding Manager initialized");
    }

    /// 检测 CPU 核心数量
    fn detect_cpu_cores(&mut self) {
        // 在实际硬件上，这里应该使用 CPUID 指令或 ACPI 表来检测核心数
        // 这里简化处理，默认设置为 4 核心
        let cores = 4;
        self.total_cores.store(cores, Ordering::Relaxed);
        crate::println!("Detected {} CPU cores", cores);
    }

    /// 设置默认核心分配
    fn setup_default_core_allocation(&mut self) {
        let total_cores = self.total_cores.load(Ordering::Relaxed) as u8;

        // 根据核心数动态调整配置
        match total_cores {
            1 => {
                // 单核系统 - 所有都在核心 0
                self.config.ai_cores = CpuMask::CORE_0;
                self.config.graphic_cores = CpuMask::CORE_0;
                self.config.media_cores = CpuMask::CORE_0;
                self.config.network_cores = CpuMask::CORE_0;
                self.config.normal_cores = CpuMask::CORE_0;
            }
            2 => {
                // 双核系统
                self.config.ai_cores = CpuMask::CORE_1;
                self.config.graphic_cores = CpuMask::CORE_1;
                self.config.media_cores = CpuMask::CORE_0;
                self.config.network_cores = CpuMask::CORE_0;
                self.config.normal_cores = CpuMask::CORE_0;
            }
            4 => {
                // 4 核系统 - 使用默认配置
                self.config = CoreConfig::default();
            }
            _ => {
                // 更多核心的系统 - 扩展配置
                let mut ai_cores = CpuMask::empty();
                let mut graphic_cores = CpuMask::empty();
                let mut media_cores = CpuMask::empty();
                let mut network_cores = CpuMask::empty();
                let mut normal_cores = CpuMask::empty();

                for core in 0..total_cores {
                    match core {
                        0 => {
                            media_cores.add_core(core);
                            network_cores.add_core(core);
                            normal_cores.add_core(core);
                        }
                        1 => {
                            graphic_cores.add_core(core);
                        }
                        2..=3 => {
                            ai_cores.add_core(core);
                        }
                        _ => {
                            // 额外的核心分配给 AIEnclave
                            ai_cores.add_core(core);
                        }
                    }
                }

                self.config.ai_cores = ai_cores;
                self.config.graphic_cores = graphic_cores;
                self.config.media_cores = media_cores;
                self.config.network_cores = network_cores;
                self.config.normal_cores = normal_cores;
            }
        }

        self.print_core_allocation();
    }

    /// 打印核心分配信息
    fn print_core_allocation(&self) {
        crate::println!("Core Allocation:");
        crate::println!("  AIEnclave:      {:?}", self.config.ai_cores);
        crate::println!("  GraphicEnclave: {:?}", self.config.graphic_cores);
        crate::println!("  MediaEnclave:   {:?}", self.config.media_cores);
        crate::println!("  NetworkEnclave: {:?}", self.config.network_cores);
        crate::println!("  Normal Process: {:?}", self.config.normal_cores);
    }

    /// 获取指定飞地类型的预分配核心
    pub fn get_enclave_cores(&self, enclave_type: EnclaveType) -> CpuMask {
        match enclave_type {
            EnclaveType::AIEnclave => self.config.ai_cores,
            EnclaveType::GraphicEnclave => self.config.graphic_cores,
            EnclaveType::MediaEnclave => self.config.media_cores,
            EnclaveType::NetworkEnclave => self.config.network_cores,
        }
    }

    /// 获取飞地进程的绑定核心（选择第一个可用核心）
    pub fn get_binded_core(&self, enclave_type: EnclaveType) -> Option<u8> {
        let cores = self.get_enclave_cores(enclave_type);
        cores.first_core()
    }

    /// 获取普通进程可用的核心
    pub fn get_normal_cores(&self) -> CpuMask {
        self.config.normal_cores
    }

    /// 获取飞地独占的核心（普通进程不可用）
    pub fn get_enclave_exclusive_cores(&self) -> CpuMask {
        let all_enclave_cores = self.config.ai_cores |
                                 self.config.graphic_cores |
                                 self.config.media_cores |
                                 self.config.network_cores;
        all_enclave_cores & !self.config.normal_cores
    }

    /// 检查核心是否为飞地专属
    pub fn is_enclave_core(&self, core_id: u8) -> bool {
        let exclusive_cores = self.get_enclave_exclusive_cores();
        exclusive_cores.contains_core(core_id)
    }

    /// 动态调整核心分配（需要权限验证）
    pub fn reallocate_cores(&mut self, enclave_type: EnclaveType, new_cores: CpuMask) -> Result<(), &'static str> {
        // 这里应该添加严格的权限验证
        // 暂时简化实现

        match enclave_type {
            EnclaveType::AIEnclave => self.config.ai_cores = new_cores,
            EnclaveType::GraphicEnclave => self.config.graphic_cores = new_cores,
            EnclaveType::MediaEnclave => self.config.media_cores = new_cores,
            EnclaveType::NetworkEnclave => self.config.network_cores = new_cores,
        }

        crate::println!("Reallocated cores for {:?}: {:?}", enclave_type, new_cores);
        Ok(())
    }

    /// 设置 CPU 亲和性
    pub fn set_cpu_affinity(&self, enclave_type: EnclaveType) -> Result<(), &'static str> {
        let cores = self.get_enclave_cores(enclave_type);

        // 在实际实现中，这里应该调用硬件相关的 API 来设置进程的 CPU 亲和性
        // 例如使用 x86_64 的 IA32_TSC_AUX MSR 或其他机制

        crate::println!("Set CPU affinity for {:?} to cores {:?}", enclave_type, cores);
        Ok(())
    }

    /// 获取当前核心配置
    pub fn config(&self) -> &CoreConfig {
        &self.config
    }

    /// 获取系统总核心数
    pub fn total_cores(&self) -> u64 {
        self.total_cores.load(Ordering::Relaxed)
    }
}

impl Default for CoreBindingManager {
    fn default() -> Self {
        Self::new()
    }
}
