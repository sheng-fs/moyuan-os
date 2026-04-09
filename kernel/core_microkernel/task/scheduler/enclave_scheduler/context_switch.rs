//! 飞地进程上下文切换优化模块
//!
//! 实现飞地进程的轻量级上下文切换，利用硬件辅助指令加速切换。

use crate::enclave::core::EnclaveType;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// 上下文切换统计信息
#[derive(Debug, Clone, Copy)]
pub struct ContextSwitchStats {
    /// 总切换次数
    pub total_switches: usize,
    /// 总切换耗时（纳秒）
    pub total_switch_time_ns: u64,
    /// 最小切换耗时（纳秒）
    pub min_switch_time_ns: u64,
    /// 最大切换耗时（纳秒）
    pub max_switch_time_ns: u64,
}

/// 飞地上下文存储区域
#[repr(C)]
pub struct EnclaveContext {
    /// 通用寄存器（x86_64 架构）
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// 指令指针
    pub rip: u64,
    /// 标志寄存器
    pub rflags: u64,
    /// 是否使用浮点/SIMD 寄存器
    pub uses_fpu: bool,
}

impl EnclaveContext {
    /// 创建新的飞地上下文
    pub fn new() -> Self {
        EnclaveContext {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0,
            uses_fpu: false,
        }
    }
}

impl Default for EnclaveContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 上下文切换管理器
pub struct ContextSwitchManager {
    /// 统计信息
    stats: ContextSwitchStats,
    /// 总切换次数计数器
    total_switches: AtomicUsize,
    /// 总切换耗时
    total_switch_time: AtomicU64,
    /// 最小切换耗时
    min_switch_time: AtomicU64,
    /// 最大切换耗时
    max_switch_time: AtomicU64,
}

impl ContextSwitchManager {
    /// 创建新的上下文切换管理器
    pub fn new() -> Self {
        ContextSwitchManager {
            stats: ContextSwitchStats {
                total_switches: 0,
                total_switch_time_ns: 0,
                min_switch_time_ns: u64::MAX,
                max_switch_time_ns: 0,
            },
            total_switches: AtomicUsize::new(0),
            total_switch_time: AtomicU64::new(0),
            min_switch_time: AtomicU64::new(u64::MAX),
            max_switch_time: AtomicU64::new(0),
        }
    }

    /// 读取时间戳计数器（RDTSC）
    #[inline(always)]
    pub fn read_tsc(&self) -> u64 {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let low: u32;
            let high: u32;
            core::arch::asm!(
                "rdtsc",
                out("eax") low,
                out("edx") high,
                options(nomem, nostack, preserves_flags)
            );
            (high as u64) << 32 | (low as u64)
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            // 非 x86_64 架构的备用实现
            0
        }
    }

    /// 保存飞地上下文
    pub fn save_context(&self, context: &mut EnclaveContext) {
        // 在实际实现中，这里应该使用汇编保存寄存器
        // 暂时简化实现
        crate::println!("Saving enclave context...");

        // 使用 XSAVE 指令保存浮点/SIMD 状态（如果需要）
        if context.uses_fpu {
            self.xsave_context(context);
        }
    }

    /// 恢复飞地上下文
    pub fn restore_context(&self, context: &EnclaveContext) {
        // 在实际实现中，这里应该使用汇编恢复寄存器
        // 暂时简化实现
        crate::println!("Restoring enclave context...");

        // 使用 XRSTOR 指令恢复浮点/SIMD 状态（如果需要）
        if context.uses_fpu {
            self.xrstor_context(context);
        }
    }

    /// 使用 XSAVE 保存浮点/SIMD 上下文
    fn xsave_context(&self, _context: &mut EnclaveContext) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            // 这里应该实现 XSAVE 指令调用
            // 需要分配 XSAVE 区域并设置正确的掩码
            crate::println!("XSAVE context saved");
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            crate::println!("FPU context saved (non-x86_64)");
        }
    }

    /// 使用 XRSTOR 恢复浮点/SIMD 上下文
    fn xrstor_context(&self, _context: &EnclaveContext) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            // 这里应该实现 XRSTOR 指令调用
            crate::println!("XRSTOR context restored");
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            crate::println!("FPU context restored (non-x86_64)");
        }
    }

    /// 执行飞地之间的上下文切换
    pub fn switch_enclaves(
        &self,
        prev_type: Option<EnclaveType>,
        next_type: EnclaveType,
        prev_context: Option<&mut EnclaveContext>,
        next_context: &EnclaveContext,
    ) {
        let start_time = self.read_tsc();

        crate::println!("Switching from {:?} to {:?}", prev_type, next_type);

        // 保存前一个飞地的上下文
        if let Some(ctx) = prev_context {
            self.save_context(ctx);
        }

        // 恢复下一个飞地的上下文
        self.restore_context(next_context);

        let end_time = self.read_tsc();
        let switch_time = end_time - start_time;

        // 记录统计信息
        self.record_switch(switch_time);
    }

    /// 记录一次上下文切换
    fn record_switch(&self, time_ns: u64) {
        self.total_switches.fetch_add(1, Ordering::Relaxed);
        self.total_switch_time.fetch_add(time_ns, Ordering::Relaxed);

        // 更新最小值
        let mut current_min = self.min_switch_time.load(Ordering::Relaxed);
        while time_ns < current_min {
            match self.min_switch_time.compare_exchange_weak(
                current_min,
                time_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_min) => current_min = new_min,
            }
        }

        // 更新最大值
        let mut current_max = self.max_switch_time.load(Ordering::Relaxed);
        while time_ns > current_max {
            match self.max_switch_time.compare_exchange_weak(
                current_max,
                time_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => current_max = new_max,
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ContextSwitchStats {
        ContextSwitchStats {
            total_switches: self.total_switches.load(Ordering::Relaxed),
            total_switch_time_ns: self.total_switch_time.load(Ordering::Relaxed),
            min_switch_time_ns: self.min_switch_time.load(Ordering::Relaxed),
            max_switch_time_ns: self.max_switch_time.load(Ordering::Relaxed),
        }
    }

    /// 获取平均切换耗时
    pub fn average_switch_time(&self) -> u64 {
        let total = self.total_switches.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        self.total_switch_time.load(Ordering::Relaxed) / total as u64
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.total_switches.store(0, Ordering::Relaxed);
        self.total_switch_time.store(0, Ordering::Relaxed);
        self.min_switch_time.store(u64::MAX, Ordering::Relaxed);
        self.max_switch_time.store(0, Ordering::Relaxed);
    }
}

impl Default for ContextSwitchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 抢占控制
pub struct PreemptionControl {
    /// 是否允许抢占
    preemption_enabled: bool,
    /// 当前正在运行的飞地优先级
    current_priority: u8,
}

impl PreemptionControl {
    /// 创建新的抢占控制器
    pub fn new() -> Self {
        PreemptionControl {
            preemption_enabled: true,
            current_priority: 0,
        }
    }

    /// 检查是否应该抢占
    pub fn should_preempt(&self, new_priority: u8) -> bool {
        if !self.preemption_enabled {
            return false;
        }
        // 只有更高优先级的飞地才能抢占
        new_priority > self.current_priority
    }

    /// 设置当前优先级
    pub fn set_current_priority(&mut self, priority: u8) {
        self.current_priority = priority;
    }

    /// 启用/禁用抢占
    pub fn set_preemption_enabled(&mut self, enabled: bool) {
        self.preemption_enabled = enabled;
    }

    /// 检查抢占是否启用
    pub fn is_preemption_enabled(&self) -> bool {
        self.preemption_enabled
    }
}

impl Default for PreemptionControl {
    fn default() -> Self {
        Self::new()
    }
}
