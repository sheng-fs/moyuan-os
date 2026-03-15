use core::fmt::Debug;
use core::result::Result;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::ToString;

/// 权限位定义
#[derive(Debug, Clone)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }
}

/// 目录项类型
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DentryType {
    File,
    Directory,
}

/// 目录项结构
#[derive(Debug, Clone)]
pub struct Dentry {
    name: String,
    inode: usize,
    dent_type: DentryType,
    #[allow(dead_code)]
    parent: Option<usize>,
    children: Vec<usize>,
}

impl Dentry {
    pub fn new(name: String, inode: usize, dent_type: DentryType, parent: Option<usize>) -> Self {
        Self {
            name,
            inode,
            dent_type,
            parent,
            children: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn inode(&self) -> usize {
        self.inode
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.dent_type, DentryType::Directory)
    }

    pub fn add_child(&mut self, child_dentry: usize) {
        self.children.push(child_dentry);
    }

    pub fn remove_child(&mut self, child_dentry: usize) {
        if let Some(index) = self.children.iter().position(|&d| d == child_dentry) {
            self.children.remove(index);
        }
    }

    pub fn children(&self) -> &Vec<usize> {
        &self.children
    }
}

/// Inode trait - 定义文件/目录的基本操作
pub trait Inode: Debug + Send + Sync {
    /// 获取文件大小
    fn size(&self) -> usize;
    
    /// 获取权限
    fn permissions(&self) -> Permissions;
    
    /// 读取文件内容
    fn read(&mut self, offset: usize, buf: &mut [u8]) -> Result<usize, &'static str>;
    
    /// 写入文件内容
    fn write(&mut self, offset: usize, buf: &[u8]) -> Result<usize, &'static str>;
    
    /// 打开文件
    fn open(&mut self) -> Result<(), &'static str>;
    
    /// 关闭文件
    fn close(&mut self) -> Result<(), &'static str>;
}

/// FileSystem trait - 定义文件系统的基本操作
pub trait FileSystem: Debug + Send + Sync {
    /// 挂载文件系统
    fn mount(&mut self) -> Result<(), &'static str>;
    
    /// 卸载文件系统
    fn unmount(&mut self) -> Result<(), &'static str>;
    
    /// 创建文件
    fn create(&mut self, path: &str) -> Result<usize, &'static str>;
    
    /// 删除文件
    fn unlink(&mut self, path: &str) -> Result<(), &'static str>;
    
    /// 读取目录内容
    fn readdir(&mut self, path: &str) -> Result<Vec<String>, &'static str>;
    
    /// 创建目录
    fn mkdir(&mut self, path: &str) -> Result<(), &'static str>;
    
    /// 打开文件获取inode
    fn open_inode(&mut self, path: &str) -> Result<usize, &'static str>;
    
    /// 根据inode号获取inode
    fn get_inode(&mut self, inode: usize) -> Option<Box<dyn Inode>>;
    
    /// 根目录inode
    fn root_inode(&self) -> usize;
}

/// VFS管理结构
pub struct VFS {
    filesystems: Vec<Box<dyn FileSystem>>,
    mounted_fs: Vec<(String, usize)>, // (挂载点, 文件系统索引)
    #[allow(dead_code)]
    dentries: Vec<Dentry>,
}

impl VFS {
    pub fn new() -> Self {
        Self {
            filesystems: Vec::new(),
            mounted_fs: Vec::new(),
            dentries: Vec::new(),
        }
    }

    /// 挂载文件系统
    pub fn mount(&mut self, mut fs: Box<dyn FileSystem>, mount_point: &str) -> Result<(), &'static str> {
        if let Err(e) = fs.mount() {
            return Err(e);
        }
        
        let fs_index = self.filesystems.len();
        self.filesystems.push(fs);
        self.mounted_fs.push((mount_point.to_string(), fs_index));
        
        Ok(())
    }

    /// 卸载文件系统
    pub fn unmount(&mut self, mount_point: &str) -> Result<(), &'static str> {
        if let Some((index, _)) = self.mounted_fs.iter().enumerate()
            .find(|(_, (path, _))| path == mount_point) {
            
            let (_, fs_index) = self.mounted_fs.remove(index);
            if let Some(fs) = self.filesystems.get_mut(fs_index) {
                return fs.unmount();
            }
        }
        
        Err("Mount point not found")
    }

    /// 创建文件
    pub fn create(&mut self, path: &str) -> Result<usize, &'static str> {
        self.get_filesystem_for_path(path)?.create(path)
    }

    /// 删除文件
    pub fn unlink(&mut self, path: &str) -> Result<(), &'static str> {
        self.get_filesystem_for_path(path)?.unlink(path)
    }

    /// 读取目录
    pub fn readdir(&mut self, path: &str) -> Result<Vec<String>, &'static str> {
        self.get_filesystem_for_path(path)?.readdir(path)
    }

    /// 创建目录
    pub fn mkdir(&mut self, path: &str) -> Result<(), &'static str> {
        self.get_filesystem_for_path(path)?.mkdir(path)
    }

    /// 打开文件
    pub fn open(&mut self, path: &str) -> Result<usize, &'static str> {
        self.get_filesystem_for_path(path)?.open_inode(path)
    }

    /// 获取inode
    pub fn get_inode(&mut self, inode: usize) -> Option<Box<dyn Inode>> {
        // 这里简化处理，实际需要根据inode号找到对应的文件系统
        for fs in &mut self.filesystems {
            if let Some(inode_obj) = fs.get_inode(inode) {
                return Some(inode_obj);
            }
        }
        None
    }

    /// 根据路径找到对应的文件系统
    fn get_filesystem_for_path(&mut self, _path: &str) -> Result<&mut dyn FileSystem, &'static str> {
        // 这里简化处理，实际需要根据路径找到对应的挂载点
        if self.filesystems.is_empty() {
            return Err("No filesystem mounted");
        }
        
        // 返回第一个文件系统（根文件系统）
        Ok(&mut **self.filesystems.first_mut().unwrap())
    }
}
