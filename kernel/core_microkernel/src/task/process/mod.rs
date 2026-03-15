pub mod process;
pub use process::*;

// 初始化进程管理
pub fn init() {
    process::init();
}
