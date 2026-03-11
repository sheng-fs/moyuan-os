// 导入 alloc crate 中的 Vec
extern crate alloc;
use alloc::vec::Vec;

// 调度器
pub struct Scheduler {
    // 就绪队列，使用 Vec 存储进程 ID
    ready_queue: Vec<usize>,
    // 当前运行的进程ID
    current_pid: Option<usize>,
}

// 在裸机环境中，只有一个核心在访问调度器，因此可以安全地实现 Send 和 Sync
unsafe impl Send for Scheduler {}
unsafe impl Sync for Scheduler {}

use spin::Mutex;

// 全局调度器
static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

// 初始化调度器
pub fn init() {
    *SCHEDULER.lock() = Some(Scheduler {
        ready_queue: Vec::new(),
        current_pid: None,
    });
}

// 添加进程到就绪队列
pub fn add_to_ready_queue(pid: usize) {
    if let Some(ref mut scheduler) = *SCHEDULER.lock() {
        // 将进程 ID 添加到就绪队列尾部
        scheduler.ready_queue.push(pid);
    }
}

// 从就绪队列中移除进程
pub fn remove_from_ready_queue(pid: usize) {
    if let Some(ref mut scheduler) = *SCHEDULER.lock() {
        // 查找并移除进程 ID
        if let Some(index) = scheduler.ready_queue.iter().position(|&p| p == pid) {
            scheduler.ready_queue.remove(index);
        }
    }
}

// 调度下一个进程
pub fn schedule() {
    if let Some(ref mut scheduler) = *SCHEDULER.lock() {
        // 保存当前进程状态
        if let Some(current_pid) = scheduler.current_pid {
            if let Some(ref mut process) = crate::task::process::get_process(current_pid) {
                if process.state == crate::task::process::ProcessState::Running {
                    process.state = crate::task::process::ProcessState::Ready;
                    add_to_ready_queue(current_pid);
                }
            }
        }
        
        // 选择下一个进程
        if !scheduler.ready_queue.is_empty() {
            // 轮转调度：取出队首进程
            let next_pid = scheduler.ready_queue.remove(0);
            
            // 更新调度器状态
            scheduler.current_pid = Some(next_pid);
            
            // 更新进程状态
            if let Some(ref mut process) = crate::task::process::get_process(next_pid) {
                process.state = crate::task::process::ProcessState::Running;
                crate::task::process::set_current_process(next_pid);
                
                // 切换到新进程
                switch_to_process(process);
            }
        } else {
            // 没有就绪进程，保持当前进程
            if let Some(current_pid) = scheduler.current_pid {
                if let Some(ref mut process) = crate::task::process::get_process(current_pid) {
                    process.state = crate::task::process::ProcessState::Running;
                }
            }
        }
    }
}

// 切换到指定进程
fn switch_to_process(process: &mut crate::task::process::ProcessControlBlock) {
    // 激活进程的地址空间
    process.address_space.activate();
    
    // 获取当前进程
    let current_pid = crate::task::process::get_current_pid();
    
    // 保存当前进程的上下文
    if let Some(pid) = current_pid {
        if let Some(current_process) = crate::task::process::get_process(pid) {
            // 调用上下文切换函数，保存当前上下文到当前进程的PCB
            switch_context(
                &mut current_process.stack_pointer,
                &mut current_process.program_counter,
                process.stack_pointer,
                process.program_counter
            );
        }
    } else {
        // 没有当前进程，直接切换到新进程
        let mut dummy_sp: u64 = 0;
        let mut dummy_pc: u64 = 0;
        switch_context(&mut dummy_sp, &mut dummy_pc, process.stack_pointer, process.program_counter);
    }
}

// 上下文切换汇编实现
#[no_mangle]
extern "C" fn switch_context(old_sp: *mut u64, old_pc: *mut u64, new_sp: u64, new_pc: u64) {
    unsafe {
        // 保存当前上下文
        core::arch::asm!(
            // 保存所有通用寄存器
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "pushfq",
            
            // 保存栈指针和程序计数器
            "mov [rdi], rsp",
            "lea rax, [rip]",
            "mov [rsi], rax",
            
            // 跳转到恢复上下文的代码
            "jmp 2f",
            
            // 恢复上下文
            "2:",
            "mov rsp, rdx",
            "popfq",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "jmp rcx",
            in("rdi") old_sp,
            in("rsi") old_pc,
            in("rdx") new_sp,
            in("rcx") new_pc,
            options(nostack, noreturn)
        );
    }
}
