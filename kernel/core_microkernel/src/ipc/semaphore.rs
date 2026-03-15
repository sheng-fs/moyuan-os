// 信号量模块

extern crate alloc;

use alloc::{vec::Vec, sync::Arc, string::String};
use spin::Mutex;

// 导入IPC结果类型
use super::IpcResult;

// 信号量类型
#[allow(dead_code)]
enum SemaphoreType {
    Binary,      // 二进制信号量
    Counting,    // 计数信号量
}

// 信号量结构
#[allow(dead_code)]
struct Semaphore {
    value: isize,            // 信号量值
    _sem_type: SemaphoreType, // 信号量类型
    name: Option<String>,    // 名称（用于命名信号量）
    waiting_processes: Vec<usize>, // 等待进程队列
    is_initialized: bool,    // 是否已初始化
}

impl Semaphore {
    fn new(value: isize, sem_type: SemaphoreType, name: Option<String>) -> Self {
        Self {
            value,
            _sem_type: sem_type,
            name,
            waiting_processes: Vec::new(),
            is_initialized: true,
        }
    }

    // P操作（等待）
    fn p(&mut self, pid: usize) {
        self.value -= 1;
        if self.value < 0 {
            // 将进程加入等待队列
            self.waiting_processes.push(pid);
            // 阻塞进程
            if let Some(process) = crate::task::process::get_process(pid) {
                process.state = crate::task::process::ProcessState::Blocked;
            }
        }
    }

    // V操作（释放）
    fn v(&mut self) {
        self.value += 1;
        if self.value <= 0 {
            // 从等待队列中唤醒一个进程
            if let Some(pid) = self.waiting_processes.pop() {
                if let Some(process) = crate::task::process::get_process(pid) {
                    process.state = crate::task::process::ProcessState::Ready;
                    // 将进程加入就绪队列
                    crate::task::scheduler::add_to_ready_queue(pid);
                }
            }
        }
    }

    // 获取信号量值
    #[allow(dead_code)]
    fn get_value(&self) -> isize {
        self.value
    }

    // 检查信号量是否已初始化
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

// 信号量表
static mut SEMAPHORES: Vec<Option<Arc<Mutex<Semaphore>>>> = Vec::new();
static mut NEXT_SEM_ID: usize = 0;

// 初始化信号量模块
pub fn init() {
    unsafe {
        let semaphores = &raw mut SEMAPHORES;
        (*semaphores).reserve(64);
    }
}

// 创建信号量
pub fn create_semaphore(value: isize, name: Option<&str>) -> IpcResult<usize> {
    if value < 0 {
        return Err(super::IpcError::InvalidArgument);
    }

    let sem_type = if value <= 1 {
        SemaphoreType::Binary
    } else {
        SemaphoreType::Counting
    };

    unsafe {
        let next_sem_id = &raw mut NEXT_SEM_ID;
        let sem_id = *next_sem_id;
        if sem_id >= 64 {
            return Err(super::IpcError::ResourceBusy);
        }
        *next_sem_id += 1;

        // 检查命名信号量是否已存在
        if let Some(name_str) = name {
            let semaphores = &raw const SEMAPHORES;
            for semaphore in &(*semaphores) {
                if let Some(semaphore) = semaphore {
                    let semaphore = semaphore.lock();
                    if let Some(semaphore_name) = &semaphore.name {
                        if semaphore_name == name_str {
                            return Err(super::IpcError::ResourceBusy);
                        }
                    }
                }
            }
        }

        let semaphore = Arc::new(Mutex::new(Semaphore::new(
            value,
            sem_type,
            name.map(|s| String::from(s)),
        )));

        let semaphores = &raw mut SEMAPHORES;
        (*semaphores).push(Some(semaphore));
        Ok(sem_id)
    }
}

// 打开信号量
pub fn open_semaphore(name: &str) -> IpcResult<usize> {
    unsafe {
        let semaphores = &raw const SEMAPHORES;
        for (i, semaphore) in (*semaphores).iter().enumerate() {
            if let Some(semaphore) = semaphore {
                let semaphore = semaphore.lock();
                if let Some(semaphore_name) = &semaphore.name {
                    if semaphore_name == name {
                        return Ok(i);
                    }
                }
            }
        }
        Err(super::IpcError::NotFound)
    }
}

// P操作（等待）
pub fn semaphore_p(sem_id: usize) -> IpcResult<()> {
    unsafe {
        let semaphores = &raw const SEMAPHORES;
        if sem_id >= (*semaphores).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let semaphore = (&(*semaphores))[sem_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let mut semaphore = semaphore.lock();

        if !semaphore.is_initialized() {
            return Err(super::IpcError::InvalidArgument);
        }

        // 获取当前进程PID
        let current_pid = crate::task::process::get_current_pid().ok_or(super::IpcError::InvalidArgument)?;

        semaphore.p(current_pid);
        Ok(())
    }
}

// V操作（释放）
pub fn semaphore_v(sem_id: usize) -> IpcResult<()> {
    unsafe {
        let semaphores = &raw const SEMAPHORES;
        if sem_id >= (*semaphores).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let semaphore = (&(*semaphores))[sem_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let mut semaphore = semaphore.lock();

        if !semaphore.is_initialized() {
            return Err(super::IpcError::InvalidArgument);
        }

        semaphore.v();
        Ok(())
    }
}

// 获取信号量值
#[allow(dead_code)]
pub fn semaphore_get_value(sem_id: usize) -> IpcResult<isize> {
    unsafe {
        let semaphores = &raw const SEMAPHORES;
        if sem_id >= (*semaphores).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let semaphore = (&(*semaphores))[sem_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let semaphore = semaphore.lock();

        if !semaphore.is_initialized() {
            return Err(super::IpcError::InvalidArgument);
        }

        Ok(semaphore.get_value())
    }
}

// 关闭信号量
pub fn close_semaphore(sem_id: usize) -> IpcResult<()> {
    unsafe {
        let semaphores = &raw mut SEMAPHORES;
        if sem_id >= (*semaphores).len() {
            return Err(super::IpcError::InvalidArgument);
        }

        let semaphore = (&(*semaphores))[sem_id].as_ref().ok_or(super::IpcError::InvalidArgument)?;
        let mut semaphore = semaphore.lock();

        // 唤醒所有等待进程
        for pid in &semaphore.waiting_processes {
            if let Some(process) = crate::task::process::get_process(*pid) {
                process.state = crate::task::process::ProcessState::Ready;
                crate::task::scheduler::add_to_ready_queue(*pid);
            }
        }

        semaphore.waiting_processes.clear();
        semaphore.is_initialized = false;
        (&mut (*semaphores))[sem_id] = None;

        Ok(())
    }
}