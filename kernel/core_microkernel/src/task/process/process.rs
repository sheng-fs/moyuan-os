// 进程控制块
#[derive(Copy, Clone)]
pub struct ProcessControlBlock {
    // 进程ID
    pub pid: usize,
    // 进程状态
    pub state: ProcessState,
    // 进程优先级
    pub priority: u8,
    // 进程的地址空间
    pub address_space: crate::mm::virt::AddressSpace,
    // 进程的堆栈指针
    pub stack_pointer: u64,
    // 进程的程序计数器
    pub program_counter: u64,
    // 通用寄存器
    pub registers: [u64; 16], // rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp, r8-r15
    // 标志寄存器
    pub rflags: u64,
    // 下一个进程（用于链表）
    pub next: Option<usize>,
}

// 进程状态
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ProcessState {
    Running,
    Ready,
    Blocked,
    Terminated,
}

// 进程列表
static mut PROCESSES: [Option<ProcessControlBlock>; 64] = [None; 64];
// 下一个可用的PID
static mut NEXT_PID: usize = 1;
// 当前运行的进程PID
static mut CURRENT_PID: Option<usize> = None;

// 初始化进程管理
pub fn init() {
    // 创建init进程（PID 1）
    let init_pid = process_create(0, 0);
    if let Some(pid) = init_pid {
        unsafe {
            CURRENT_PID = Some(pid);
            if let Some(ref mut process) = PROCESSES[pid] {
                process.state = ProcessState::Running;
            }
        }
    }
}

// 创建进程
pub fn process_create(entry_point: u64, stack_size: u64) -> Option<usize> {
    unsafe {
        // 分配PID
        let pid = NEXT_PID;
        if pid >= PROCESSES.len() {
            return None;
        }
        NEXT_PID += 1;
        
        // 创建地址空间
        let mut address_space = crate::mm::virt::AddressSpace::new();
        address_space.init();
        
        // 分配堆栈
        let stack_start = crate::mm::physical::allocate_page().unwrap_or(0x800000) as u64;
        let stack_pointer = stack_start + 4096;
        
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
            next: None,
        };
        
        PROCESSES[pid] = Some(process);
        Some(pid)
    }
}

// 销毁进程
pub fn process_exit(pid: usize) {
    unsafe {
        if let Some(ref mut process) = PROCESSES[pid] {
            process.state = ProcessState::Terminated;
            
            // 释放地址空间
            // 这里简化处理，实际需要释放页表等资源
            
            // 释放堆栈
            let stack_start = process.stack_pointer - 4096;
            crate::mm::physical::free_page(stack_start as usize);
            
            // 从进程列表中移除
            PROCESSES[pid] = None;
            
            // 如果是当前进程，切换到其他进程
            if CURRENT_PID == Some(pid) {
                crate::task::scheduler::schedule();
            }
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
