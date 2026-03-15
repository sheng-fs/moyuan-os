// 安全模块

pub mod user_group;
pub mod file_perm;

use user_group::{Uid, Gid, SecurityError};
use file_perm::FilePermissions;

// 初始化安全系统
pub fn init() {
    // 初始化用户和组系统
    user_group::init();
    
    // 初始化文件权限系统
    // 目前文件权限系统不需要特殊初始化
}

// 检查进程权限
#[allow(dead_code)]
pub fn check_process_permission(
    process_uid: Uid,
    process_gid: Gid,
    target_uid: Uid,
    target_gid: Gid,
    _requested_perm: u8,
) -> bool {
    // root用户有所有权限
    if process_uid == 0 {
        return true;
    }
    
    // 检查所有者权限
    if process_uid == target_uid {
        return true;
    }
    
    // 检查组权限
    if user_group::is_user_in_group(process_uid, target_gid) || process_gid == target_gid {
        // 组权限检查
        true
    } else {
        // 其他用户权限检查
        false
    }
}

// 检查文件访问权限
#[allow(dead_code)]
pub fn check_file_access(
    permissions: &FilePermissions,
    uid: Uid,
    gid: Gid,
    requested_perm: u8,
) -> bool {
    file_perm::check_file_access(permissions, uid, gid, requested_perm)
}

// 安全操作结果类型
#[allow(dead_code)]
type SecurityResult<T> = Result<T, SecurityError>;
