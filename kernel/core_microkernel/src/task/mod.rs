pub mod process;
pub mod scheduler;

// 初始化进程管理
pub fn init() {
    // 初始化进程管理
    process::init();
    
    // 初始化调度器
    scheduler::init();
}
