use super::super::process::process::ProcessControlBlock;
use core::cell::RefCell;
use core::ptr::NonNull;

// 进程队列
struct ProcessQueue {
    head: Option<NonNull<ProcessControlBlock>>,
    tail: Option<NonNull<ProcessControlBlock>>,
}

impl ProcessQueue {
    // 创建新的进程队列
    pub fn new() -> Self {
        ProcessQueue {
            head: None,
            tail: None,
        }
    }
    
    // 添加进程到队列
    pub fn push(&mut self, process: NonNull<ProcessControlBlock>) {
        unsafe {
            if let Some(tail) = self.tail {
                // 这里需要在ProcessControlBlock中添加next指针
                // 暂时简化处理
            } else {
                self.head = Some(process);
                self.tail = Some(process);
            }
        }
    }
    
    // 从队列中取出进程
    pub fn pop(&mut self) -> Option<NonNull<ProcessControlBlock>> {
        let head = self.head;
        if let Some(head) = head {
            unsafe {
                // 这里需要在ProcessControlBlock中添加next指针
                // 暂时简化处理
                self.head = None;
                self.tail = None;
            }
        }
        head
    }
}

// 调度器
pub struct Scheduler {
    // 就绪队列
    ready_queue: ProcessQueue,
    // 当前运行的进程
    current_process: Option<NonNull<ProcessControlBlock>>,
}

impl Scheduler {
    // 创建新的调度器
    pub fn new() -> Self {
        Scheduler {
            ready_queue: ProcessQueue::new(),
            current_process: None,
        }
    }
    
    // 添加进程到就绪队列
    pub fn add_process(&mut self, process: NonNull<ProcessControlBlock>) {
        self.ready_queue.push(process);
    }
    
    // 选择下一个要运行的进程
    pub fn select_next_process(&mut self) -> Option<NonNull<ProcessControlBlock>> {
        self.ready_queue.pop()
    }
    
    // 切换进程
    pub fn switch_process(&mut self, next_process: NonNull<ProcessControlBlock>) {
        // 这里实现进程切换逻辑
        self.current_process = Some(next_process);
    }
    
    // 获取当前运行的进程
    pub fn current_process(&self) -> Option<NonNull<ProcessControlBlock>> {
        self.current_process
    }
}
