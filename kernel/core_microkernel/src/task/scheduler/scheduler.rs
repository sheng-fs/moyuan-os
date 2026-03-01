// 调度器
pub struct Scheduler {
    // 就绪队列
    ready_queue: [usize; 64], // 使用固定大小的数组替代 Vec
    // 就绪队列长度
    queue_length: usize,
    // 当前运行的进程ID
    current_pid: Option<usize>,
}

// 全局调度器
static mut SCHEDULER: Option<Scheduler> = None;

// 初始化调度器
pub fn init() {
    unsafe {
        SCHEDULER = Some(Scheduler {
            ready_queue: [0; 64],
            queue_length: 0,
            current_pid: None,
        });
    }
}

// 添加进程到就绪队列
pub fn add_to_ready_queue(pid: usize) {
    unsafe {
        if let Some(ref mut scheduler) = SCHEDULER {
            if scheduler.queue_length < 64 {
                scheduler.ready_queue[scheduler.queue_length] = pid;
                scheduler.queue_length += 1;
            }
        }
    }
}

// 从就绪队列中移除进程
pub fn remove_from_ready_queue(pid: usize) {
    unsafe {
        if let Some(ref mut scheduler) = SCHEDULER {
            for i in 0..scheduler.queue_length {
                if scheduler.ready_queue[i] == pid {
                    // 移动后面的进程
                    for j in i..scheduler.queue_length - 1 {
                        scheduler.ready_queue[j] = scheduler.ready_queue[j + 1];
                    }
                    scheduler.queue_length -= 1;
                    break;
                }
            }
        }
    }
}

// 调度下一个进程
pub fn schedule() {
    unsafe {
        if let Some(ref mut scheduler) = SCHEDULER {
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
            if scheduler.queue_length > 0 {
                // 轮转调度：取出队首进程
                let next_pid = scheduler.ready_queue[0];
                remove_from_ready_queue(next_pid);
                
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
}

// 切换到指定进程
fn switch_to_process(process: &mut crate::task::process::ProcessControlBlock) {
    // 激活进程的地址空间
    process.address_space.activate();
    
    // 这里需要实现上下文切换的汇编代码
    // 暂时使用简单的实现
    unsafe {
        core::arch::asm!(
            "mov rsp, {0}",
            "jmp {1}",
            in(reg) process.stack_pointer,
            in(reg) process.program_counter,
            options(nostack, noreturn)
        );
    }
}

// 上下文切换汇编实现
#[no_mangle]
extern "C" fn switch_context(old_sp: *mut u64, old_pc: *mut u64, new_sp: u64, new_pc: u64) {
    unsafe {
        // 保存当前上下文
        core::arch::asm!(
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
            "mov [rdi], rsp",
            "lea rax, [rip]",
            "mov [rsi], rax",
            "jmp 2f",
            "2:",
            "mov rsp, rdx",
            "jmp rcx",
            in("rdi") old_sp,
            in("rsi") old_pc,
            in("rdx") new_sp,
            in("rcx") new_pc,
            options(nostack, noreturn)
        );
    }
}
