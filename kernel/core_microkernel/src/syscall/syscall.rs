// 系统调用号
enum SyscallNumber {
    Exit = 0,
    Fork = 1,
    Exec = 2,
    Open = 3,
    Read = 4,
    Write = 5,
    Close = 6,
    Waitpid = 7,
    Brk = 8,
    Mmap = 9,
    Munmap = 10,
}

// 系统调用处理函数类型
type SyscallHandler = fn(usize, usize, usize, usize, usize, usize) -> isize;

// 系统调用表
static mut SYSCALL_TABLE: [Option<SyscallHandler>; 11] = [
    Some(sys_exit),        // 0
    Some(sys_fork),        // 1
    Some(sys_exec),        // 2
    Some(sys_open),        // 3
    Some(sys_read),        // 4
    Some(sys_write),       // 5
    Some(sys_close),       // 6
    Some(sys_waitpid),     // 7
    Some(sys_brk),         // 8
    Some(sys_mmap),        // 9
    Some(sys_munmap),      // 10
];

// 系统调用处理函数
pub fn handle_syscall(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> isize {
    unsafe {
        if syscall_num < SYSCALL_TABLE.len() {
            if let Some(handler) = SYSCALL_TABLE[syscall_num] {
                return handler(arg1, arg2, arg3, arg4, arg5, arg6);
            }
        }
        -1
    }
}

// 退出系统调用
fn sys_exit(status: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 终止当前进程
    if let Some(current_process) = crate::task::process::get_current_process() {
        let pid = current_process.pid;
        crate::task::process::process_exit(pid);
    }
    0
}

// 创建进程系统调用
fn sys_fork(_arg1: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 创建子进程
    if let Some(current_process) = crate::task::process::get_current_process() {
        // 复制父进程的入口点和堆栈大小
        let entry_point = current_process.program_counter;
        let stack_size = 4096; // 简化处理
        
        // 创建子进程
        if let Some(child_pid) = crate::task::process::process_create(entry_point, stack_size) {
            // 将子进程添加到就绪队列
            crate::task::scheduler::add_to_ready_queue(child_pid);
            
            // 父进程返回子进程PID
            return child_pid as isize;
        }
    }
    -1
}

// 执行程序系统调用
fn sys_exec(path: usize, argv: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 这里实现执行程序系统调用
    0
}

// 打开文件系统调用
fn sys_open(path: usize, flags: usize, mode: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 这里实现打开文件系统调用
    0
}

// 读取文件系统调用
fn sys_read(fd: usize, buf: usize, count: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 这里实现读取文件系统调用
    0
}

// 写入文件系统调用
fn sys_write(fd: usize, buf: usize, count: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 向串口/VGA输出字符串
    if fd == 1 || fd == 2 { // stdout or stderr
        unsafe {
            let buf_ptr = buf as *const u8;
            let output = core::str::from_utf8_unchecked(core::slice::from_raw_parts(buf_ptr, count));
            crate::console::print(core::format_args!("{}", output));
        }
        return count as isize;
    }
    -1
}

// 关闭文件系统调用
fn sys_close(fd: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 这里实现关闭文件系统调用
    0
}

// 等待进程系统调用
fn sys_waitpid(pid: usize, status: usize, options: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 这里实现等待进程系统调用
    0
}

// 内存分配系统调用
fn sys_brk(addr: usize, _arg2: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 简化的内存分配实现
    // 实际需要管理进程的堆空间
    addr as isize
}

// 内存映射系统调用
fn sys_mmap(addr: usize, length: usize, prot: usize, flags: usize, fd: usize, offset: usize) -> isize {
    // 这里实现内存映射系统调用
    0
}

// 解除内存映射系统调用
fn sys_munmap(addr: usize, length: usize, _arg3: usize, _arg4: usize, _arg5: usize, _arg6: usize) -> isize {
    // 这里实现解除内存映射系统调用
    0
}
