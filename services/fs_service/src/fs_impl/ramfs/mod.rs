extern crate alloc;

use core::fmt::Debug;
use core::result::Result;
use core::option::Option;
use core::clone::Clone;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::string::ToString;

use crate::vfs::{Inode, FileSystem, Permissions, Dentry};

/// Ramfs的Inode实现
#[derive(Debug)]
pub struct RamfsInode {
    inode: usize,
    size: usize,
    permissions: Permissions,
    data: Vec<u8>,
    is_directory: bool,
    children: BTreeMap<String, usize>, // 目录项: 文件名 -> inode号
}

impl RamfsInode {
    pub fn new(inode: usize, is_directory: bool) -> Self {
        Self {
            inode,
            size: 0,
            permissions: Permissions::default(),
            data: Vec::new(),
            is_directory,
            children: BTreeMap::new(),
        }
    }

    pub fn add_child(&mut self, name: String, inode: usize) {
        self.children.insert(name, inode);
    }

    pub fn remove_child(&mut self, name: &str) -> Option<usize> {
        self.children.remove(name)
    }

    pub fn get_child(&self, name: &str) -> Option<usize> {
        self.children.get(name).copied()
    }

    pub fn children(&self) -> &BTreeMap<String, usize> {
        &self.children
    }
}

impl Inode for RamfsInode {
    fn size(&self) -> usize {
        self.size
    }

    fn permissions(&self) -> Permissions {
        self.permissions.clone()
    }

    fn read(&mut self, offset: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.is_directory {
            return Err("Cannot read directory");
        }

        let start = offset.min(self.size);
        let end = (offset + buf.len()).min(self.size);
        let count = end - start;

        if count > 0 {
            buf[..count].copy_from_slice(&self.data[start..end]);
        }

        Ok(count)
    }

    fn write(&mut self, offset: usize, buf: &[u8]) -> Result<usize, &'static str> {
        if self.is_directory {
            return Err("Cannot write to directory");
        }

        let new_size = offset + buf.len();
        if new_size > self.data.len() {
            self.data.resize(new_size, 0);
        }

        self.data[offset..offset + buf.len()].copy_from_slice(buf);
        self.size = new_size;

        Ok(buf.len())
    }

    fn open(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
}

/// Ramfs文件系统实现
#[derive(Debug)]
pub struct Ramfs {
    inodes: BTreeMap<usize, RamfsInode>,
    next_inode: usize,
    root_inode: usize,
    dentries: Vec<Dentry>,
}

impl Default for Ramfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Ramfs {
    pub fn new() -> Self {
        let mut inodes = BTreeMap::new();
        let root_inode = 1;
        
        // 创建根目录inode
        let root = RamfsInode::new(root_inode, true);
        inodes.insert(root_inode, root);

        Self {
            inodes,
            next_inode: 2,
            root_inode,
            dentries: Vec::new(),
        }
    }

    /// 解析路径，返回最后一个目录的inode和文件名
    fn parse_path(&self, path: &str) -> Result<(usize, String), &'static str> {
        let mut components: Vec<&str> = path.split('/').filter(|&s| !s.is_empty()).collect();
        
        if components.is_empty() {
            return Ok((self.root_inode, String::new()));
        }

        let filename = components.pop().unwrap().to_string();
        let mut current_inode = self.root_inode;

        for component in components {
            let inode = self.inodes.get(&current_inode).ok_or("Directory not found")?;
            if !inode.is_directory {
                return Err("Not a directory");
            }
            current_inode = inode.get_child(component).ok_or("Directory not found")?;
        }

        Ok((current_inode, filename))
    }

    /// 创建新的inode
    fn allocate_inode(&mut self, is_directory: bool) -> usize {
        let inode = self.next_inode;
        self.next_inode += 1;
        
        let ramfs_inode = RamfsInode::new(inode, is_directory);
        self.inodes.insert(inode, ramfs_inode);
        
        inode
    }
}

impl FileSystem for Ramfs {
    fn mount(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn unmount(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn create(&mut self, path: &str) -> Result<usize, &'static str> {
        let (parent_inode, filename) = self.parse_path(path)?;
        
        // 先检查文件是否存在
        { 
            let parent = self.inodes.get(&parent_inode).ok_or("Parent directory not found")?;
            if !parent.is_directory {
                return Err("Parent is not a directory");
            }

            if parent.get_child(&filename).is_some() {
                return Err("File already exists");
            }
        }

        // 分配新的inode
        let new_inode = self.allocate_inode(false);
        
        // 添加到父目录
        let parent = self.inodes.get_mut(&parent_inode).ok_or("Parent directory not found")?;
        parent.add_child(filename, new_inode);

        Ok(new_inode)
    }

    fn unlink(&mut self, path: &str) -> Result<(), &'static str> {
        let (parent_inode, filename) = self.parse_path(path)?;
        
        let parent = self.inodes.get_mut(&parent_inode).ok_or("Parent directory not found")?;
        if !parent.is_directory {
            return Err("Parent is not a directory");
        }

        let inode = parent.remove_child(&filename).ok_or("File not found")?;
        self.inodes.remove(&inode);

        Ok(())
    }

    fn readdir(&mut self, path: &str) -> Result<Vec<String>, &'static str> {
        let (inode, _) = self.parse_path(path)?;
        
        let dir = self.inodes.get(&inode).ok_or("Directory not found")?;
        if !dir.is_directory {
            return Err("Not a directory");
        }

        let mut entries = Vec::new();
        entries.push(".".to_string());
        entries.push("..".to_string());
        
        for name in dir.children().keys() {
            entries.push(name.clone());
        }

        Ok(entries)
    }

    fn mkdir(&mut self, path: &str) -> Result<(), &'static str> {
        let (parent_inode, dirname) = self.parse_path(path)?;
        
        // 先检查目录是否存在
        { 
            let parent = self.inodes.get(&parent_inode).ok_or("Parent directory not found")?;
            if !parent.is_directory {
                return Err("Parent is not a directory");
            }

            if parent.get_child(&dirname).is_some() {
                return Err("Directory already exists");
            }
        }

        // 分配新的inode
        let new_inode = self.allocate_inode(true);
        
        // 添加到父目录
        let parent = self.inodes.get_mut(&parent_inode).ok_or("Parent directory not found")?;
        parent.add_child(dirname, new_inode);

        Ok(())
    }

    fn open_inode(&mut self, path: &str) -> Result<usize, &'static str> {
        let (parent_inode, filename) = self.parse_path(path)?;
        
        if filename.is_empty() {
            return Ok(parent_inode); // 打开目录
        }

        let parent = self.inodes.get(&parent_inode).ok_or("Parent directory not found")?;
        if !parent.is_directory {
            return Err("Parent is not a directory");
        }

        parent.get_child(&filename).ok_or("File not found")
    }

    fn get_inode(&mut self, inode: usize) -> Option<Box<dyn Inode>> {
        if let Some(ramfs_inode) = self.inodes.get_mut(&inode) {
            Some(Box::new(ramfs_inode.clone()))
        } else {
            None
        }
    }

    fn root_inode(&self) -> usize {
        self.root_inode
    }
}

impl Clone for RamfsInode {
    fn clone(&self) -> Self {
        Self {
            inode: self.inode,
            size: self.size,
            permissions: self.permissions.clone(),
            data: self.data.clone(),
            is_directory: self.is_directory,
            children: self.children.clone(),
        }
    }
}
