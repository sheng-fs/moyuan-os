// 导入mm模块
use crate::mm::{physical, virt};

// 文件描述符结构
#[allow(dead_code)]
#[derive(Clone)]
pub struct FileDescriptor {
    pub inode: usize,
    pub offset: usize,
    pub flags: u32,
}

// 进程控制块
#[allow(dead_code)]
#[derive(Clone)]
pub struct ProcessControlBlock {
    // 进程ID
    pub pid: usize,
    // 进程状态
    pub state: ProcessState,
    // 进程优先级
    pub priority: u8,
    // 进程的地址空间
    pub address_space: virt::AddressSpace,
    // 进程的堆栈指针
    pub stack_pointer: u64,
    // 进程的程序计数器
    pub program_counter: u64,
    // 通用寄存器
    pub registers: [u64; 16], // rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp, r8-r15
    // 标志寄存器
    pub rflags: u64,
    // 文件描述符表
    pub file_descriptors: [Option<FileDescriptor>; 64],
    // 下一个可用的文件描述符
    pub next_fd: usize,
    // 用户ID
    pub uid: usize,
    // 组ID
    pub gid: usize,
    // 有效用户ID
    pub euid: usize,
    // 有效组ID
    pub egid: usize,
    // 下一个进程（用于链表）
    pub next: Option<usize>,
}

// 进程状态
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessState {
    Running,
    Ready,
    Blocked,
    Terminated,
}

// 进程列表
static mut PROCESSES: [Option<ProcessControlBlock>; 64] = [const { None }; 64];
// 下一个可用的PID
static mut NEXT_PID: usize = 1;
// 当前运行的进程PID
static mut CURRENT_PID: Option<usize> = None;

// 获取当前运行的进程PID
pub fn get_current_pid() -> Option<usize> {
    unsafe {
        // 裸机环境下，单核心调度无竞态，因此可以安全访问
        CURRENT_PID
    }
}

// 设置当前运行的进程PID
#[allow(dead_code)]
pub fn set_current_pid(pid: Option<usize>) {
    unsafe {
        // 裸机环境下，单核心调度无竞态，因此可以安全访问
        CURRENT_PID = pid;
    }
}

// 初始化进程管理
pub fn init() {
    // 创建init进程（PID 1）
    match process_create(0, 0) {
        Ok(pid) => {
            unsafe {
                CURRENT_PID = Some(pid);
                if let Some(ref mut process) = PROCESSES[pid] {
                    process.state = ProcessState::Running;
                }
            }
            crate::console::print(core::format_args!("初始化进程创建成功: PID {}\n", pid));
        },
        Err(err) => {
            crate::console::print(core::format_args!("错误: 创建初始化进程失败: {:?}\n", err));
        }
    }
}

// 进程创建错误类型
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProcessError {
    OutOfPids,
    AddressSpaceInitFailed,
    StackAllocationFailed,
    InternalError,
}

// 创建进程
pub fn process_create(entry_point: u64, _stack_size: u64) -> Result<usize, ProcessError> {
    unsafe {
        // 分配PID
        let pid = NEXT_PID;
        if pid >= 64 {
            return Err(ProcessError::OutOfPids);
        }
        NEXT_PID += 1;
        
        // 创建地址空间
        let mut address_space = virt::AddressSpace::new();
        address_space.init();
        
        // 检查地址空间初始化是否成功
        if address_space.page_table_root() == 0 {
            return Err(ProcessError::AddressSpaceInitFailed);
        }
        
        // 分配堆栈
        let stack_start = match physical::allocate_page() {
            Ok(addr) => addr as u64,
            Err(_) => return Err(ProcessError::StackAllocationFailed),
        };
        let stack_pointer = stack_start + 4096;
        
        // 初始化用户ID和组ID
        // 默认使用root权限
        let uid = 0;
        let gid = 0;
        let euid = 0;
        let egid = 0;
        
        // 创建PCB
        let process = ProcessControlBlock {
            pid,
            state: ProcessState::Ready,
            priority: 128,
            address_space,
            stack_pointer,
            program_counter: entry_point,
            registers: [0; 16],
            rflags: 0x202, // IF=1
            file_descriptors: [const { None }; 64],
            next_fd: 3, // 0,1,2 分别是stdin, stdout, stderr
            uid,
            gid,
            euid,
            egid,
            next: None,
        };
        
        PROCESSES[pid] = Some(process);
        Ok(pid)
    }
}

// 销毁进程
pub fn process_exit(pid: usize) {
    unsafe {
        if let Some(ref mut process) = PROCESSES[pid] {
            process.state = ProcessState::Terminated;
            
            // 释放地址空间
            process.address_space.deinit();
            
            // 释放堆栈
            let stack_start = process.stack_pointer - 4096;
            match physical::free_page(stack_start as usize) {
                Ok(()) => {},
                Err(err) => {
                    crate::console::print(core::format_args!("警告: 释放进程堆栈失败: {:?}\n", err));
                }
            }
            
            // 从进程列表中移除
            PROCESSES[pid] = None;
            
            // 如果是当前进程，切换到其他进程
            if CURRENT_PID == Some(pid) {
                crate::task::scheduler::schedule();
            }
        }
    }
}

// 为ProcessControlBlock实现Drop trait，确保资源自动释放
impl Drop for ProcessControlBlock {
    fn drop(&mut self) {
        // 释放地址空间
        self.address_space.deinit();
        
        // 释放堆栈
        let stack_start = self.stack_pointer - 4096;
        if let Err(err) = physical::free_page(stack_start as usize) {
            // 这里不能使用print，因为drop可能在任何上下文中被调用
            // 只记录错误，不做其他处理
            core::panic!("释放进程堆栈失败: {:?}", err);
        }
    }
}

// 获取当前进程
pub fn get_current_process() -> Option<&'static mut ProcessControlBlock> {
    unsafe {
        if let Some(pid) = CURRENT_PID {
            PROCESSES[pid].as_mut()
        } else {
            None
        }
    }
}

// 设置当前进程
pub fn set_current_process(pid: usize) {
    unsafe {
        CURRENT_PID = Some(pid);
        if let Some(ref mut process) = PROCESSES[pid] {
            process.state = ProcessState::Running;
        }
    }
}

// 获取进程
pub fn get_process(pid: usize) -> Option<&'static mut ProcessControlBlock> {
    unsafe {
        PROCESSES[pid].as_mut()
    }
}
