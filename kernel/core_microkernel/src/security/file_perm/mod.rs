// 文件权限管理模块

use super::user_group::{Uid, Gid, is_user_in_group};

// 文件权限位定义
#[allow(dead_code)]
pub const PERM_READ: u8 = 0o4;
#[allow(dead_code)]
pub const PERM_WRITE: u8 = 0o2;
#[allow(dead_code)]
pub const PERM_EXECUTE: u8 = 0o1;

// 特殊权限位
#[allow(dead_code)]
pub const PERM_SUID: u16 = 0o4000;
#[allow(dead_code)]
pub const PERM_SGID: u16 = 0o2000;
#[allow(dead_code)]
pub const PERM_STICKY: u16 = 0o1000;

// 文件权限结构体
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FilePermissions {
    pub mode: u16,      // 权限模式
    pub uid: Uid,       // 文件所有者ID
    pub gid: Gid,       // 文件组ID
}

impl FilePermissions {
    // 创建新的文件权限
    #[allow(dead_code)]
    pub fn new(mode: u16, uid: Uid, gid: Gid) -> Self {
        Self {
            mode,
            uid,
            gid,
        }
    }
    
    // 检查用户是否有读取权限
    #[allow(dead_code)]
    pub fn can_read(&self, uid: Uid, gid: Gid) -> bool {
        self.check_permission(uid, gid, PERM_READ)
    }
    
    // 检查用户是否有写入权限
    #[allow(dead_code)]
    pub fn can_write(&self, uid: Uid, gid: Gid) -> bool {
        self.check_permission(uid, gid, PERM_WRITE)
    }
    
    // 检查用户是否有执行权限
    #[allow(dead_code)]
    pub fn can_execute(&self, uid: Uid, gid: Gid) -> bool {
        self.check_permission(uid, gid, PERM_EXECUTE)
    }
    
    // 检查用户是否有指定权限
    #[allow(dead_code)]
    fn check_permission(&self, uid: Uid, gid: Gid, perm: u8) -> bool {
        // root用户有所有权限
        if uid == 0 {
            return true;
        }
        
        // 检查所有者权限
        if uid == self.uid {
            return (self.mode & (perm as u16)) != 0;
        }
        
        // 检查组权限
        if is_user_in_group(uid, self.gid) || gid == self.gid {
            return (self.mode & ((perm as u16) << 3)) != 0;
        }
        
        // 检查其他用户权限
        (self.mode & ((perm as u16) << 6)) != 0
    }
    
    // 设置权限
    #[allow(dead_code)]
    pub fn set_permission(&mut self, mode: u16) {
        self.mode = mode;
    }
    
    // 检查是否设置了SUID位
    #[allow(dead_code)]
    pub fn has_suid(&self) -> bool {
        (self.mode & PERM_SUID as u16) != 0
    }
    
    // 检查是否设置了SGID位
    #[allow(dead_code)]
    pub fn has_sgid(&self) -> bool {
        (self.mode & PERM_SGID as u16) != 0
    }
    
    // 检查是否设置了Sticky位
    #[allow(dead_code)]
    pub fn has_sticky(&self) -> bool {
        (self.mode & PERM_STICKY as u16) != 0
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
    match requested_perm {
        PERM_READ => permissions.can_read(uid, gid),
        PERM_WRITE => permissions.can_write(uid, gid),
        PERM_EXECUTE => permissions.can_execute(uid, gid),
        _ => false,
    }
}

// 从umask创建默认权限
#[allow(dead_code)]
pub fn create_default_permissions(uid: Uid, gid: Gid, umask: u16) -> FilePermissions {
    // 默认权限：文件 0666，目录 0777
    let default_mode = 0o666 & !umask;
    FilePermissions::new(default_mode, uid, gid)
}
