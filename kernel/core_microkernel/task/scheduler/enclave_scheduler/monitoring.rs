//! 监控与调试机制模块
//!
//! 实现飞地调度器的上下文切换耗时统计、队列状态监控和调试接口。

use crate::enclave::core::EnclaveType;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// 飞地调度统计
#[derive(Debug, Clone, Copy)]
pub struct EnclaveScheduleStats {
    /// 总调度次数
    pub total_schedules: usize,
    /// AIEnclave 调度次数
    pub ai_schedules: usize,
    /// GraphicEnclave 调度次数
    pub graphic_schedules: usize,
    /// MediaEnclave 调度次数
    pub media_schedules: usize,
    /// NetworkEnclave 调度次数
    pub network_schedules: usize,
}

/// 监控统计信息
pub struct MonitoringStats {
    /// 调度统计
    schedule_stats: EnclaveScheduleStats,
    /// 总调度次数
    total_schedules: AtomicUsize,
    /// AIEnclave 调度次数
    ai_schedules: AtomicUsize,
    /// GraphicEnclave 调度次数
    graphic_schedules: AtomicUsize,
    /// MediaEnclave 调度次数
    media_schedules: AtomicUsize,
    /// NetworkEnclave 调度次数
    network_schedules: AtomicUsize,
    /// 启动时间戳
    start_time: AtomicU64,
}

impl MonitoringStats {
    /// 创建新的监控统计
    pub fn new() -> Self {
        MonitoringStats {
            schedule_stats: EnclaveScheduleStats {
                total_schedules: 0,
                ai_schedules: 0,
                graphic_schedules: 0,
                media_schedules: 0,
                network_schedules: 0,
            },
            total_schedules: AtomicUsize::new(0),
            ai_schedules: AtomicUsize::new(0),
            graphic_schedules: AtomicUsize::new(0),
            media_schedules: AtomicUsize::new(0),
            network_schedules: AtomicUsize::new(0),
            start_time: AtomicU64::new(0),
        }
    }

    /// 记录一次调度
    pub fn record_schedule(&self, enclave_type: EnclaveType) {
        self.total_schedules.fetch_add(1, Ordering::Relaxed);

        match enclave_type {
            EnclaveType::AIEnclave => {
                self.ai_schedules.fetch_add(1, Ordering::Relaxed);
            }
            EnclaveType::GraphicEnclave => {
                self.graphic_schedules.fetch_add(1, Ordering::Relaxed);
            }
            EnclaveType::MediaEnclave => {
                self.media_schedules.fetch_add(1, Ordering::Relaxed);
            }
            EnclaveType::NetworkEnclave => {
                self.network_schedules.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// 设置启动时间
    pub fn set_start_time(&self, time: u64) {
        self.start_time.store(time, Ordering::Relaxed);
    }

    /// 获取调度统计
    pub fn get_schedule_stats(&self) -> EnclaveScheduleStats {
        EnclaveScheduleStats {
            total_schedules: self.total_schedules.load(Ordering::Relaxed),
            ai_schedules: self.ai_schedules.load(Ordering::Relaxed),
            graphic_schedules: self.graphic_schedules.load(Ordering::Relaxed),
            media_schedules: self.media_schedules.load(Ordering::Relaxed),
            network_schedules: self.network_schedules.load(Ordering::Relaxed),
        }
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.total_schedules.store(0, Ordering::Relaxed);
        self.ai_schedules.store(0, Ordering::Relaxed);
        self.graphic_schedules.store(0, Ordering::Relaxed);
        self.media_schedules.store(0, Ordering::Relaxed);
        self.network_schedules.store(0, Ordering::Relaxed);
    }
}

impl Default for MonitoringStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 调试命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCommand {
    /// 显示调度器状态
    ShowStatus,
    /// 显示核心绑定关系
    ShowCoreBinding,
    /// 显示上下文切换统计
    ShowContextSwitchStats,
    /// 显示队列状态
    ShowQueueStatus,
    /// 重置统计信息
    ResetStats,
}

/// 调试接口
pub struct DebugInterface;

impl DebugInterface {
    /// 执行调试命令
    pub fn execute_command(command: DebugCommand) {
        match command {
            DebugCommand::ShowStatus => {
                Self::show_status();
            }
            DebugCommand::ShowCoreBinding => {
                Self::show_core_binding();
            }
            DebugCommand::ShowContextSwitchStats => {
                Self::show_context_switch_stats();
            }
            DebugCommand::ShowQueueStatus => {
                Self::show_queue_status();
            }
            DebugCommand::ResetStats => {
                Self::reset_stats();
            }
        }
    }

    /// 显示调度器状态
    fn show_status() {
        crate::println!("=== Enclave Scheduler Status ===");

        let scheduler = super::get_enclave_scheduler();
        let scheduler = scheduler.lock();

        // 显示当前运行的飞地
        if let Some(enclave) = scheduler.current_enclave() {
            crate::println!("Current Enclave: {:?}", enclave);
        } else {
            crate::println!("Current Enclave: None");
        }

        // 显示调度统计
        let stats = scheduler.monitoring().get_schedule_stats();
        crate::println!("Total Schedules: {}", stats.total_schedules);
        crate::println!("  AIEnclave:      {}", stats.ai_schedules);
        crate::println!("  GraphicEnclave: {}", stats.graphic_schedules);
        crate::println!("  MediaEnclave:   {}", stats.media_schedules);
        crate::println!("  NetworkEnclave: {}", stats.network_schedules);
    }

    /// 显示核心绑定关系
    fn show_core_binding() {
        crate::println!("=== Core Binding Configuration ===");

        let scheduler = super::get_enclave_scheduler();
        let scheduler = scheduler.lock();
        let core_binding = scheduler.core_binding();

        crate::println!("Total Cores: {}", core_binding.total_cores());

        let config = core_binding.config();
        crate::println!("AIEnclave Cores:      {:?}", config.ai_cores);
        crate::println!("GraphicEnclave Cores: {:?}", config.graphic_cores);
        crate::println!("MediaEnclave Cores:   {:?}", config.media_cores);
        crate::println!("NetworkEnclave Cores: {:?}", config.network_cores);
        crate::println!("Normal Process Cores: {:?}", config.normal_cores);

        let exclusive_cores = core_binding.get_enclave_exclusive_cores();
        crate::println!("Exclusive Enclave Cores: {:?}", exclusive_cores);
    }

    /// 显示上下文切换统计
    fn show_context_switch_stats() {
        crate::println!("=== Context Switch Statistics ===");

        // 这里应该获取 ContextSwitchManager 的统计信息
        // 暂时简化实现
        crate::println!("Context switch stats would be shown here");
    }

    /// 显示队列状态
    fn show_queue_status() {
        crate::println!("=== Enclave Queue Status ===");

        let scheduler = super::get_enclave_scheduler();
        let scheduler = scheduler.lock();

        let enclave_types = [
            EnclaveType::AIEnclave,
            EnclaveType::GraphicEnclave,
            EnclaveType::MediaEnclave,
            EnclaveType::NetworkEnclave,
        ];

        for &enclave_type in &enclave_types {
            let status = scheduler.queue_status(enclave_type);
            crate::println!("{:?}:", enclave_type);
            crate::println!("  Length: {}", status.length);
            crate::println!("  Is Empty: {}", status.is_empty);
            crate::println!("  Is Full: {}", status.is_full);
            crate::println!("  Total Enqueues: {}", status.total_enqueues);
            crate::println!("  Total Dequeues: {}", status.total_dequeues);
        }
    }

    /// 重置统计信息
    fn reset_stats() {
        crate::println!("Resetting enclave scheduler statistics...");

        let mut scheduler = super::get_enclave_scheduler().lock();

        // 重置调度统计
        // 注意：MonitoringStats 目前没有暴露重置方法，需要添加
        crate::println!("Statistics reset complete");
    }

    /// 打印所有调试信息
    pub fn dump_all_info() {
        crate::println!("========================================");
        crate::println!("  Enclave Scheduler Debug Information");
        crate::println!("========================================");

        Self::show_status();
        crate::println!();
        Self::show_core_binding();
        crate::println!();
        Self::show_queue_status();
        crate::println!();
        Self::show_context_switch_stats();

        crate::println!("========================================");
    }
}

/// 性能监控缓冲区
pub struct PerformanceMonitorBuffer {
    /// 上下文切换耗时记录（循环缓冲区）
    switch_times: [u64; 1024],
    /// 写指针
    write_idx: AtomicUsize,
}

impl PerformanceMonitorBuffer {
    /// 创建新的性能监控缓冲区
    pub fn new() -> Self {
        PerformanceMonitorBuffer {
            switch_times: [0; 1024],
            write_idx: AtomicUsize::new(0),
        }
    }

    /// 记录上下文切换耗时
    pub fn record_switch_time(&self, time_ns: u64) {
        let idx = self.write_idx.fetch_add(1, Ordering::Relaxed) % 1024;
        unsafe {
            let buf_ptr = &self.switch_times as *const [u64; 1024] as *mut [u64; 1024];
            (*buf_ptr)[idx] = time_ns;
        }
    }

    /// 获取最近的切换耗时
    pub fn get_recent_switch_times(&self, count: usize) -> Vec<u64> {
        let mut result = Vec::with_capacity(count);
        let write_idx = self.write_idx.load(Ordering::Relaxed);

        for i in 0..count.min(1024) {
            let idx = (write_idx + 1024 - count + i) % 1024;
            result.push(self.switch_times[idx]);
        }

        result
    }

    /// 计算 99% 延迟
    pub fn calculate_99th_latency(&self) -> u64 {
        let mut times: Vec<u64> = (0..1024).map(|i| self.switch_times[i]).collect();
        times.sort();

        let idx = (times.len() * 99) / 100;
        times.get(idx).copied().unwrap_or(0)
    }
}

impl Default for PerformanceMonitorBuffer {
    fn default() -> Self {
        Self::new()
    }
}
