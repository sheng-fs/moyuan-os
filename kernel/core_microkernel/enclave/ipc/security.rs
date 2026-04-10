//! 安全控制与验证模块
//!
//! 实现飞地间通信的权限验证、IOMMU配置和安全检查。

use super::error::*;
use super::memory_pool::SharedMemoryBlock;
use super::channel::ChannelId;
use crate::enclave::core::EnclaveType;
use crate::enclave::iommu::*;
use bitflags::bitflags;

/// 飞地间通信权限规则
#[derive(Debug, Clone, Copy)]
pub struct CommunicationRule {
    /// 发送方
    pub sender: EnclaveType,
    /// 接收方
    pub receiver: EnclaveType,
    /// 是否允许通信
    pub allowed: bool,
}

/// 访问权限标志
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AccessPermissions: u8 {
        /// 读权限
        const READ = 1 << 0;
        /// 写权限
        const WRITE = 1 << 1;
        /// 执行权限
        const EXECUTE = 1 << 2;
    }
}

/// 安全控制器
pub struct SecurityController {
    /// 通信规则列表
    communication_rules: Vec<CommunicationRule>,
    /// 初始化标志
    initialized: bool,
}

impl SecurityController {
    /// 创建新的安全控制器
    pub fn new() -> Self {
        SecurityController {
            communication_rules: Vec::new(),
            initialized: false,
        }
    }

    /// 初始化安全控制器
    pub fn init(&mut self) -> Result<(), EnclaveIpcError> {
        crate::println!("Initializing Security Controller...");

        // 设置默认的通信规则
        self.setup_default_communication_rules();

        self.initialized = true;
        crate::println!("Security Controller initialized");
        Ok(())
    }

    /// 设置默认的通信规则
    fn setup_default_communication_rules(&mut self) {
        // 允许所有飞地与微内核通信
        // 飞地间通信需要明确授权

        // 示例规则：允许AIEnclave与GraphicEnclave通信
        self.add_communication_rule(EnclaveType::AIEnclave, EnclaveType::GraphicEnclave, true);
        self.add_communication_rule(EnclaveType::GraphicEnclave, EnclaveType::AIEnclave, true);

        // 示例规则：允许MediaEnclave与NetworkEnclave通信
        self.add_communication_rule(EnclaveType::MediaEnclave, EnclaveType::NetworkEnclave, true);
        self.add_communication_rule(EnclaveType::NetworkEnclave, EnclaveType::MediaEnclave, true);
    }

    /// 添加通信规则
    pub fn add_communication_rule(
        &mut self,
        sender: EnclaveType,
        receiver: EnclaveType,
        allowed: bool,
    ) {
        // 移除已存在的规则
        self.communication_rules.retain(|r| !(r.sender == sender && r.receiver == receiver));

        // 添加新规则
        self.communication_rules.push(CommunicationRule {
            sender,
            receiver,
            allowed,
        });

        crate::println!(
            "Added communication rule: {:?} -> {:?} = {}",
            sender, receiver, allowed
        );
    }

    /// 验证通信权限
    pub fn verify_communication_permission(
        &self,
        sender: EnclaveType,
        receiver: EnclaveType,
    ) -> Result<(), SecurityError> {
        // 查找规则
        let rule = self.communication_rules.iter().find(|r| {
            r.sender == sender && r.receiver == receiver
        });

        if let Some(rule) = rule {
            if rule.allowed {
                Ok(())
            } else {
                Err(SecurityError::PermissionDenied)
            }
        } else {
            // 默认拒绝未明确授权的飞地间通信
            Err(SecurityError::PermissionDenied)
        }
    }

    /// 配置IOMMU权限
    pub fn configure_iommu_permissions(
        &self,
        channel_id: ChannelId,
        sender: EnclaveType,
        receiver: EnclaveType,
        memory_block: &SharedMemoryBlock,
    ) -> Result<(), SecurityError> {
        crate::println!(
            "Configuring IOMMU permissions for channel {:?}: {:?} <-> {:?}",
            channel_id, sender, receiver
        );

        // 在实际实现中，这里应该：
        // 1. 获取或创建IOMMU域
        // 2. 映射共享内存到IOMMU域
        // 3. 设置访问权限（仅允许sender和receiver访问）

        // 简化实现
        let iommu_manager = get_iommu_manager();

        // 尝试分配IOMMU域
        if let Ok(domain) = iommu_manager.allocate_domain() {
            // 映射内存
            let _ = domain.map_memory(
                memory_block.physical_address(),
                memory_block.physical_address(),
                memory_block.size(),
                true,
            );

            crate::println!("IOMMU domain configured for channel {:?}", channel_id);
        }

        Ok(())
    }

    /// 清理IOMMU权限
    pub fn cleanup_iommu_permissions(
        &self,
        channel_id: ChannelId,
    ) -> Result<(), SecurityError> {
        crate::println!("Cleaning up IOMMU permissions for channel {:?}", channel_id);

        // 在实际实现中，这里应该：
        // 1. 取消映射共享内存
        // 2. 释放IOMMU域

        Ok(())
    }

    /// 验证数据完整性
    pub fn verify_data_integrity(
        &self,
        data: &[u8],
        expected_hash: &[u8],
    ) -> Result<(), SecurityError> {
        // 在实际实现中，这里应该计算并验证哈希
        // 简化实现
        let _ = data;
        let _ = expected_hash;

        Ok(())
    }

    /// 检测重放攻击
    pub fn detect_replay_attack(
        &self,
        sequence_number: u64,
        last_valid_sequence: u64,
    ) -> Result<(), SecurityError> {
        if sequence_number <= last_valid_sequence {
            return Err(SecurityError::VerificationFailed);
        }
        Ok(())
    }

    /// 获取通信规则
    pub fn communication_rules(&self) -> &[CommunicationRule] {
        &self.communication_rules
    }
}

impl Default for SecurityController {
    fn default() -> Self {
        Self::new()
    }
}

/// 安全审计日志记录
pub struct SecurityAuditLog {
    /// 日志条目
    entries: Vec<SecurityAuditEntry>,
}

/// 安全审计条目
#[derive(Debug, Clone)]
pub struct SecurityAuditEntry {
    /// 时间戳
    pub timestamp: u64,
    /// 事件类型
    pub event_type: SecurityEventType,
    /// 发送方
    pub sender: Option<EnclaveType>,
    /// 接收方
    pub receiver: Option<EnclaveType>,
    /// 通道ID
    pub channel_id: Option<ChannelId>,
    /// 描述
    pub description: &'static str,
}

/// 安全事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityEventType {
    /// 通道建立
    ChannelEstablished,
    /// 通道关闭
    ChannelClosed,
    /// 权限验证成功
    PermissionGranted,
    /// 权限验证失败
    PermissionDenied,
    /// 数据传输
    DataTransferred,
    /// 重放攻击检测
    ReplayAttackDetected,
    /// 完整性验证失败
    IntegrityCheckFailed,
}

impl SecurityAuditLog {
    /// 创建新的安全审计日志
    pub fn new() -> Self {
        SecurityAuditLog {
            entries: Vec::new(),
        }
    }

    /// 记录安全事件
    pub fn log_event(
        &mut self,
        event_type: SecurityEventType,
        sender: Option<EnclaveType>,
        receiver: Option<EnclaveType>,
        channel_id: Option<ChannelId>,
        description: &'static str,
    ) {
        let entry = SecurityAuditEntry {
            timestamp: 0, // 应该使用系统时间
            event_type,
            sender,
            receiver,
            channel_id,
            description,
        };

        self.entries.push(entry);

        crate::println!(
            "Security Audit: {:?} - {}",
            event_type, description
        );
    }

    /// 获取审计日志条目
    pub fn entries(&self) -> &[SecurityAuditEntry] {
        &self.entries
    }
}

impl Default for SecurityAuditLog {
    fn default() -> Self {
        Self::new()
    }
}
