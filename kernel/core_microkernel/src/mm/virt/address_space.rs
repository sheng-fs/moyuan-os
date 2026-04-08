// 虚拟地址空间

use crate::mm::physical;
use crate::console;

#[derive(Copy, Clone)]
pub struct AddressSpace {
    // 页表根目录地址
    page_table_root: u64,
}

impl AddressSpace {
    // 创建新的地址空间
    pub fn new() -> Self {
        AddressSpace {
            page_table_root: 0, // 暂时设置为0，后续需要初始化
        }
    }
    
    // 获取页表根目录地址
    #[allow(dead_code)]
    pub fn page_table_root(&self) -> u64 {
        self.page_table_root
    }
    
    // 设置页表根目录地址
    #[allow(dead_code)]
    pub fn set_page_table_root(&mut self, root: u64) {
        self.page_table_root = root;
    }
    
    // 初始化页表
    pub fn init(&mut self) {
        // 分配页表根目录（PML4）
        match physical::allocate_page() {
            Ok(pml4_addr) => {
                self.page_table_root = pml4_addr as u64;
                // 清空页表
                unsafe {
                    let pml4 = core::slice::from_raw_parts_mut(pml4_addr as *mut u64, 512);
                    for entry in pml4.iter_mut() {
                        *entry = 0;
                    }
                }
                console::print(core::format_args!("页表根目录初始化成功: {:#x}\n", self.page_table_root));
            },
            Err(err) => {
                console::print(core::format_args!("错误: 分配页表根目录失败: {:?}\n", err));
                self.page_table_root = 0;
            }
        }
    }
    
    // 映射虚拟地址到物理地址
    pub fn map(&mut self, virtual_addr: u64, physical_addr: u64, flags: u64) {
        if self.page_table_root == 0 {
            self.init();
        }
        
        // 验证物理地址是否页对齐
        debug_assert!(physical_addr & 0xFFF == 0, "物理地址未对齐到页边界: {:#x}", physical_addr);
        
        // 解析虚拟地址
        let pml4_index = ((virtual_addr >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virtual_addr >> 30) & 0x1FF) as usize;
        let pd_index = ((virtual_addr >> 21) & 0x1FF) as usize;
        let pt_index = ((virtual_addr >> 12) & 0x1FF) as usize;
        
        unsafe {
            // 获取PML4
            let pml4 = core::slice::from_raw_parts_mut(self.page_table_root as *mut u64, 512);
            
            // 获取或创建PDPT
            let pdpt_addr = if (pml4[pml4_index] & 1) != 0 {
                let addr = pml4[pml4_index] & !0xFFF;
                // 验证PDPT地址是否页对齐
                debug_assert!(addr & 0xFFF == 0, "PDPT地址未对齐到页边界: {:#x}", addr);
                addr
            } else {
                match physical::allocate_page() {
                    Ok(addr) => {
                        // 验证分配的地址是否页对齐
                        debug_assert!(addr & 0xFFF == 0, "分配的PDPT地址未对齐到页边界: {:#x}", addr);
                        pml4[pml4_index] = addr as u64 | flags | 1;
                        let pdpt = core::slice::from_raw_parts_mut(addr as *mut u64, 512);
                        for entry in pdpt.iter_mut() {
                            *entry = 0;
                        }
                        addr as u64
                    },
                    Err(err) => {
                        crate::console::print(core::format_args!("错误: 分配PDPT失败: {:?}\n", err));
                        return;
                    }
                }
            };
            
            // 获取或创建PD
            let pdpt = core::slice::from_raw_parts_mut(pdpt_addr as *mut u64, 512);
            let pd_addr = if (pdpt[pdpt_index] & 1) != 0 {
                let addr = pdpt[pdpt_index] & !0xFFF;
                // 验证PD地址是否页对齐
                debug_assert!(addr & 0xFFF == 0, "PD地址未对齐到页边界: {:#x}", addr);
                addr
            } else {
                match physical::allocate_page() {
                    Ok(addr) => {
                        // 验证分配的地址是否页对齐
                        debug_assert!(addr & 0xFFF == 0, "分配的PD地址未对齐到页边界: {:#x}", addr);
                        pdpt[pdpt_index] = addr as u64 | flags | 1;
                        let pd = core::slice::from_raw_parts_mut(addr as *mut u64, 512);
                        for entry in pd.iter_mut() {
                            *entry = 0;
                        }
                        addr as u64
                    },
                    Err(err) => {
                        crate::console::print(core::format_args!("错误: 分配PD失败: {:?}\n", err));
                        return;
                    }
                }
            };
            
            // 获取或创建PT
            let pd = core::slice::from_raw_parts_mut(pd_addr as *mut u64, 512);
            let pt_addr = if (pd[pd_index] & 1) != 0 {
                let addr = pd[pd_index] & !0xFFF;
                // 验证PT地址是否页对齐
                debug_assert!(addr & 0xFFF == 0, "PT地址未对齐到页边界: {:#x}", addr);
                addr
            } else {
                match physical::allocate_page() {
                    Ok(addr) => {
                        // 验证分配的地址是否页对齐
                        debug_assert!(addr & 0xFFF == 0, "分配的PT地址未对齐到页边界: {:#x}", addr);
                        pd[pd_index] = addr as u64 | flags | 1;
                        let pt = core::slice::from_raw_parts_mut(addr as *mut u64, 512);
                        for entry in pt.iter_mut() {
                            *entry = 0;
                        }
                        addr as u64
                    },
                    Err(err) => {
                        crate::console::print(core::format_args!("错误: 分配PT失败: {:?}\n", err));
                        return;
                    }
                }
            };
            
            // 设置页表项
            let pt = core::slice::from_raw_parts_mut(pt_addr as *mut u64, 512);
            // 确保物理地址对齐到页边界
            let aligned_physical_addr = physical_addr & !0xFFF;
            debug_assert!(aligned_physical_addr & 0xFFF == 0, "物理地址未对齐到页边界: {:#x}", aligned_physical_addr);
            pt[pt_index] = aligned_physical_addr | flags | 1;
        }
    }
    
    // 解除虚拟地址映射
    #[allow(dead_code)]
    pub fn unmap(&mut self, virtual_addr: u64) {
        if self.page_table_root == 0 {
            return;
        }
        
        // 解析虚拟地址
        let pml4_index = ((virtual_addr >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virtual_addr >> 30) & 0x1FF) as usize;
        let pd_index = ((virtual_addr >> 21) & 0x1FF) as usize;
        let pt_index = ((virtual_addr >> 12) & 0x1FF) as usize;
        
        unsafe {
            // 获取PML4
            let pml4 = core::slice::from_raw_parts_mut(self.page_table_root as *mut u64, 512);
            
            // 检查PML4项是否存在
            if (pml4[pml4_index] & 1) == 0 {
                return;
            }
            
            // 获取PDPT
            let pdpt_addr = pml4[pml4_index] & !0xFFF;
            let pdpt = core::slice::from_raw_parts_mut(pdpt_addr as *mut u64, 512);
            
            // 检查PDPT项是否存在
            if (pdpt[pdpt_index] & 1) == 0 {
                return;
            }
            
            // 获取PD
            let pd_addr = pdpt[pdpt_index] & !0xFFF;
            let pd = core::slice::from_raw_parts_mut(pd_addr as *mut u64, 512);
            
            // 检查PD项是否存在
            if (pd[pd_index] & 1) == 0 {
                return;
            }
            
            // 获取PT
            let pt_addr = pd[pd_index] & !0xFFF;
            let pt = core::slice::from_raw_parts_mut(pt_addr as *mut u64, 512);
            
            // 清除页表项
            pt[pt_index] = 0;
        }
    }
    
    // 查找虚拟地址对应的物理地址
    #[allow(dead_code)]
    pub fn translate(&self, virtual_addr: u64) -> Option<u64> {
        if self.page_table_root == 0 {
            return None;
        }
        
        // 解析虚拟地址
        let pml4_index = ((virtual_addr >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virtual_addr >> 30) & 0x1FF) as usize;
        let pd_index = ((virtual_addr >> 21) & 0x1FF) as usize;
        let pt_index = ((virtual_addr >> 12) & 0x1FF) as usize;
        
        unsafe {
            // 获取PML4
            let pml4 = core::slice::from_raw_parts(self.page_table_root as *const u64, 512);
            
            // 检查PML4项是否存在
            if (pml4[pml4_index] & 1) == 0 {
                return None;
            }
            
            // 获取PDPT
            let pdpt_addr = pml4[pml4_index] & !0xFFF;
            let pdpt = core::slice::from_raw_parts(pdpt_addr as *const u64, 512);
            
            // 检查PDPT项是否存在
            if (pdpt[pdpt_index] & 1) == 0 {
                return None;
            }
            
            // 获取PD
            let pd_addr = pdpt[pdpt_index] & !0xFFF;
            let pd = core::slice::from_raw_parts(pd_addr as *const u64, 512);
            
            // 检查PD项是否存在
            if (pd[pd_index] & 1) == 0 {
                return None;
            }
            
            // 获取PT
            let pt_addr = pd[pd_index] & !0xFFF;
            let pt = core::slice::from_raw_parts(pt_addr as *const u64, 512);
            
            // 检查PT项是否存在
            if (pt[pt_index] & 1) == 0 {
                return None;
            }
            
            // 计算物理地址
            let physical_addr = (pt[pt_index] & !0xFFF) | (virtual_addr & 0xFFF);
            Some(physical_addr)
        }
    }
    
    // 激活地址空间
    pub fn activate(&self) {
        if self.page_table_root != 0 {
            unsafe {
                core::arch::asm!(
                    "mov cr3, rax",
                    in("rax") self.page_table_root,
                    options(nomem, nostack)
                );
            }
        }
    }
    
    // 释放地址空间资源
    pub fn deinit(&mut self) {
        if self.page_table_root != 0 {
            unsafe {
                // 释放页表结构
                self.free_page_tables(self.page_table_root);
                
                // 重置页表根目录地址
                self.page_table_root = 0;
            }
        }
    }
    
    // 递归释放页表
    unsafe fn free_page_tables(&self, table_addr: u64) {
        let table = core::slice::from_raw_parts_mut(table_addr as *mut u64, 512);
        
        // 遍历页表项
        for entry in table.iter() {
            if (*entry & 1) != 0 {
                let child_addr = *entry & !0xFFF;
                // 检查是否是叶子页表（PT）
                // 这里简化处理，假设所有非叶子页表都需要递归释放
                self.free_page_tables(child_addr);
            }
        }
        
        // 释放当前页表
        let _ = physical::free_page(table_addr as usize);
    }
}

// 页表标志位
pub const PAGE_FLAG_PRESENT: u64 = 1 << 0;
pub const PAGE_FLAG_WRITABLE: u64 = 1 << 1;
#[allow(dead_code)]
pub const PAGE_FLAG_USER: u64 = 1 << 2;
#[allow(dead_code)]
pub const PAGE_FLAG_PWT: u64 = 1 << 3;
#[allow(dead_code)]
pub const PAGE_FLAG_PCD: u64 = 1 << 4;
#[allow(dead_code)]
pub const PAGE_FLAG_ACCESSED: u64 = 1 << 5;
#[allow(dead_code)]
pub const PAGE_FLAG_DIRTY: u64 = 1 << 6;
#[allow(dead_code)]
pub const PAGE_FLAG_PSE: u64 = 1 << 7;
pub const PAGE_FLAG_GLOBAL: u64 = 1 << 8;
