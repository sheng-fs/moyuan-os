#![no_std]

extern crate alloc;

pub mod vfs;
pub mod fs_impl;

use vfs::VFS;

/// 文件系统服务
pub struct FSService {
    vfs: VFS,
}

impl Default for FSService {
    fn default() -> Self {
        Self::new()
    }
}

impl FSService {
    pub fn new() -> Self {
        Self {
            vfs: VFS::new(),
        }
    }

    pub fn vfs(&mut self) -> &mut VFS {
        &mut self.vfs
    }
}
