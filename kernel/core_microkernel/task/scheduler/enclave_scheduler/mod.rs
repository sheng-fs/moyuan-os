//! 飞地调度器模块
//!
//! 墨渊 OS 微内核的独立飞地调度子模块，负责管理 GraphicEnclave、AIEnclave、MediaEnclave、NetworkEnclave 这四类飞地进程。

pub mod lockfree_ringbuffer;
pub mod enclave_queue;
pub mod core_binding;
pub mod context_switch;
pub mod monitoring;

pub use self::lockfree_ringbuffer::*;
pub use self::enclave_queue::*;
pub use self::core_binding::*;
pub use self::context_switch::*;
pub use self::monitoring::*;

use crate::enclave::core::EnclaveType;
use spin::Mutex;

/// 飞地调度器主结构
pub struct EnclaveScheduler {
    /// AIEnclave 队列（最高优先级）
    ai_queue: EnclaveQueue,
    /// GraphicEnclave 队列
    graphic_queue: EnclaveQueue,
    /// MediaEnclave 队列
    media_queue: EnclaveQueue,
    /// NetworkEnclave 队列（最低优先级）
    network_queue: EnclaveQueue,
    /// CPU 核心绑定管理器
    core_binding: CoreBindingManager,
    /// 监控信息
    monitoring: MonitoringStats,
    /// 当前运行的飞地进程
    current_enclave: Option<EnclaveType>,
}

impl EnclaveScheduler {
    /// 创建新的飞地调度器
    pub fn new() -> Self {
        Self {
            ai_queue: EnclaveQueue::new(EnclaveType::AIEnclave),
            graphic_queue: EnclaveQueue::new(EnclaveType::GraphicEnclave),
            media_queue: EnclaveQueue::new(EnclaveType::MediaEnclave),
            network_queue: EnclaveQueue::new(EnclaveType::NetworkEnclave),
            core_binding: CoreBindingManager::new(),
            monitoring: MonitoringStats::new(),
            current_enclave: None,
        }
    }

    /// 初始化飞地调度器
    pub fn init(&mut self) {
        crate::println!("Initializing Enclave Scheduler...");

        // 初始化 CPU 核心绑定
        self.core_binding.init();

        crate::println!("Enclave Scheduler initialized");
    }

    /// 添加飞地进程到对应队列
    pub fn enqueue_enclave(&mut self, enclave_type: EnclaveType) -> Result<(), &'static str> {
        match enclave_type {
            EnclaveType::AIEnclave => self.ai_queue.enqueue(enclave_type),
            EnclaveType::GraphicEnclave => self.graphic_queue.enqueue(enclave_type),
            EnclaveType::MediaEnclave => self.media_queue.enqueue(enclave_type),
            EnclaveType::NetworkEnclave => self.network_queue.enqueue(enclave_type),
        }
    }

    /// 选择下一个要运行的飞地进程（按优先级顺序）
    pub fn select_next_enclave(&mut self) -> Option<EnclaveType> {
        // 按优先级顺序检查队列：AIEnclave > GraphicEnclave > MediaEnclave > NetworkEnclave
        if let Some(enclave) = self.ai_queue.dequeue() {
            self.monitoring.record_schedule(EnclaveType::AIEnclave);
            return Some(enclave);
        }

        if let Some(enclave) = self.graphic_queue.dequeue() {
            self.monitoring.record_schedule(EnclaveType::GraphicEnclave);
            return Some(enclave);
        }

        if let Some(enclave) = self.media_queue.dequeue() {
            self.monitoring.record_schedule(EnclaveType::MediaEnclave);
            return Some(enclave);
        }

        if let Some(enclave) = self.network_queue.dequeue() {
            self.monitoring.record_schedule(EnclaveType::NetworkEnclave);
            return Some(enclave);
        }

        None
    }

    /// 检查是否有可运行的飞地进程
    pub fn has_pending_enclaves(&self) -> bool {
        !self.ai_queue.is_empty() ||
        !self.graphic_queue.is_empty() ||
        !self.media_queue.is_empty() ||
        !self.network_queue.is_empty()
    }

    /// 获取当前运行的飞地
    pub fn current_enclave(&self) -> Option<EnclaveType> {
        self.current_enclave
    }

    /// 设置当前运行的飞地
    pub fn set_current_enclave(&mut self, enclave: Option<EnclaveType>) {
        self.current_enclave = enclave;
    }

    /// 获取核心绑定管理器
    pub fn core_binding(&self) -> &CoreBindingManager {
        &self.core_binding
    }

    /// 获取可变的核心绑定管理器
    pub fn core_binding_mut(&mut self) -> &mut CoreBindingManager {
        &mut self.core_binding
    }

    /// 获取监控统计信息
    pub fn monitoring(&self) -> &MonitoringStats {
        &self.monitoring
    }

    /// 获取特定队列的状态
    pub fn queue_status(&self, enclave_type: EnclaveType) -> QueueStatus {
        match enclave_type {
            EnclaveType::AIEnclave => self.ai_queue.status(),
            EnclaveType::GraphicEnclave => self.graphic_queue.status(),
            EnclaveType::MediaEnclave => self.media_queue.status(),
            EnclaveType::NetworkEnclave => self.network_queue.status(),
        }
    }
}

/// 全局飞地调度器实例
static mut ENCLAVE_SCHEDULER: Option<Mutex<EnclaveScheduler>> = None;

/// 初始化飞地调度器
pub fn init_enclave_scheduler() {
    unsafe {
        ENCLAVE_SCHEDULER = Some(Mutex::new(EnclaveScheduler::new()));
        if let Some(scheduler) = &mut ENCLAVE_SCHEDULER {
            scheduler.lock().init();
        }
    }
}

/// 获取飞地调度器
pub fn get_enclave_scheduler() -> &'static Mutex<EnclaveScheduler> {
    unsafe {
        ENCLAVE_SCHEDULER.as_ref().expect("Enclave scheduler not initialized")
    }
}
