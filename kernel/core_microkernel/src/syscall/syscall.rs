extern crate alloc;

// 系统调用枚举
#[derive(Debug, Clone, Copy)]
pub enum Syscall {
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
    Unlink = 11,
}

// 全局文件系统服务
use spin::Once;
use moyuan_fs_service::FSService;
use moyuan_fs_service::fs_impl::ramfs::Ramfs;
use alloc::vec;
use alloc::boxed::Box;
use alloc::string::String;

static FS_SERVICE: Once<spin::Mutex<FSService>> = Once::new();

// 获取文件系统服务
fn fs_service() -> &'static spin::Mutex<FSService> {
    FS_SERVICE.call_once(|| {
        spin::Mutex::new(FSService::new())
    })
}

// 初始化文件系统
pub fn init_fs() {
    let mut fs_service = fs_service().lock();
    let ramfs = Ramfs::new();
    if let Err(e) = fs_service.vfs().mount(Box::new(ramfs), "/") {
        crate::console::print(core::format_args!("警告: 挂载Ramfs失败: {}\n", e));
    } else {
        crate::console::print(core::format_args!("Ramfs挂载成功\n"));
    }
}

// 系统调用错误枚举
#[derive(Debug, Clone, Copy)]
pub enum SyscallError {
    InvalidSyscall,
    PermissionDenied,
    BadAddress,
    InvalidArgument,
    NotFound,
    ResourceBusy,
    NotSupported,
}

// 系统调用处理函数类型
type SyscallHandler = fn(&[usize]) -> Result<usize, SyscallError>;

// 系统调用表
static mut SYSCALL_TABLE: [Option<SyscallHandler>; 12] = [
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
    Some(sys_unlink),      // 11
];

// 系统调用处理函数
pub fn handle_syscall(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> isize {
    let args = &[arg1, arg2, arg3, arg4, arg5, arg6];
    
    match Syscall::try_from(syscall_num) {
        Ok(syscall) => {
            let result = handle_syscall_enum(syscall, args);
            match result {
                Ok(value) => value as isize,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

// 从usize转换为Syscall枚举
impl TryFrom<usize> for Syscall {
    type Error = ();
    
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Syscall::Exit),
            1 => Ok(Syscall::Fork),
            2 => Ok(Syscall::Exec),
            3 => Ok(Syscall::Open),
            4 => Ok(Syscall::Read),
            5 => Ok(Syscall::Write),
            6 => Ok(Syscall::Close),
            7 => Ok(Syscall::Waitpid),
            8 => Ok(Syscall::Brk),
            9 => Ok(Syscall::Mmap),
            10 => Ok(Syscall::Munmap),
            11 => Ok(Syscall::Unlink),
            _ => Err(()),
        }
    }
}

// 使用枚举的系统调用处理函数
pub fn handle_syscall_enum(syscall: Syscall, args: &[usize]) -> Result<usize, SyscallError> {
    unsafe {
        let syscall_num = syscall as usize;
        let table_len = core::ptr::addr_of!(SYSCALL_TABLE).read().len();
        if syscall_num < table_len {
            if let Some(handler) = SYSCALL_TABLE[syscall_num] {
                return handler(args);
            }
        }
        Err(SyscallError::InvalidSyscall)
    }
}

// 退出系统调用
fn sys_exit(_args: &[usize]) -> Result<usize, SyscallError> {
    // 终止当前进程
    if let Some(current_process) = crate::task::process::get_current_process() {
        let pid = current_process.pid;
        crate::task::process::process_exit(pid);
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 创建进程系统调用
fn sys_fork(_args: &[usize]) -> Result<usize, SyscallError> {
    // 创建子进程
    if let Some(current_process) = crate::task::process::get_current_process() {
        // 复制父进程的入口点和堆栈大小
        let entry_point = current_process.program_counter;
        let stack_size = 4096; // 简化处理
        
        // 创建子进程
        match crate::task::process::process_create(entry_point, stack_size) {
            Ok(child_pid) => {
                // 将子进程添加到就绪队列
                crate::task::scheduler::add_to_ready_queue(child_pid);
                
                // 父进程返回子进程PID
                Ok(child_pid)
            }
            Err(_) => Err(SyscallError::ResourceBusy),
        }
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 执行程序系统调用
fn sys_exec(_args: &[usize]) -> Result<usize, SyscallError> {
    // 这里实现执行程序系统调用
    Err(SyscallError::NotSupported)
}

// 打开文件系统调用
fn sys_open(args: &[usize]) -> Result<usize, SyscallError> {
    let path_ptr = args[0];
    let flags = args[1];
    
    // 从用户空间读取路径
    let path = unsafe {
        let mut path = String::new();
        let mut ptr = path_ptr as *const u8;
        while *ptr != 0 {
            path.push(*ptr as char);
            ptr = ptr.add(1);
        }
        path
    };
    
    // 获取当前进程
    let current_process = crate::task::process::get_current_process().ok_or(SyscallError::InvalidArgument)?;
    
    // 打开文件
    let mut fs_service = fs_service().lock();
    let inode = fs_service.vfs().open(&path).map_err(|_| SyscallError::NotFound)?;
    
    // 分配文件描述符
    let fd = current_process.next_fd;
    if fd >= 64 {
        return Err(SyscallError::ResourceBusy);
    }
    
    // 创建文件描述符
    current_process.file_descriptors[fd] = Some(crate::task::process::FileDescriptor {
        inode,
        offset: 0,
        flags: flags as u32,
    });
    current_process.next_fd += 1;
    
    Ok(fd)
}

// 读取文件系统调用
fn sys_read(args: &[usize]) -> Result<usize, SyscallError> {
    let fd = args[0];
    let buf = args[1];
    let count = args[2];
    
    // 检查文件描述符
    let current_process = crate::task::process::get_current_process().ok_or(SyscallError::InvalidArgument)?;
    let file_desc = current_process.file_descriptors[fd].as_mut().ok_or(SyscallError::InvalidArgument)?;
    
    // 获取inode
    let mut fs_service = fs_service().lock();
    let mut inode = fs_service.vfs().get_inode(file_desc.inode).ok_or(SyscallError::NotFound)?;
    
    // 读取文件
    let mut buffer = vec![0u8; count];
    let bytes_read = inode.read(file_desc.offset, &mut buffer).map_err(|_| SyscallError::InvalidArgument)?;
    
    // 复制到用户空间
    unsafe {
        let buf_ptr = buf as *mut u8;
        core::ptr::copy_nonoverlapping(buffer.as_ptr(), buf_ptr, bytes_read);
    }
    
    // 更新文件偏移
    file_desc.offset += bytes_read;
    
    Ok(bytes_read)
}

// 写入文件系统调用
fn sys_write(args: &[usize]) -> Result<usize, SyscallError> {
    let fd = args[0];
    let buf = args[1];
    let count = args[2];
    
    // 向串口/VGA输出字符串
    if fd == 1 || fd == 2 { // stdout or stderr
        unsafe {
            let buf_ptr = buf as *const u8;
            let output = core::str::from_utf8_unchecked(core::slice::from_raw_parts(buf_ptr, count));
            crate::console::print(core::format_args!("{}", output));
        }
        return Ok(count);
    }
    
    // 检查文件描述符
    let current_process = crate::task::process::get_current_process().ok_or(SyscallError::InvalidArgument)?;
    let file_desc = current_process.file_descriptors[fd].as_mut().ok_or(SyscallError::InvalidArgument)?;
    
    // 获取inode
    let mut fs_service = fs_service().lock();
    let mut inode = fs_service.vfs().get_inode(file_desc.inode).ok_or(SyscallError::NotFound)?;
    
    // 从用户空间读取数据
    let buffer = unsafe {
        core::slice::from_raw_parts(buf as *const u8, count)
    };
    
    // 写入文件
    let bytes_written = inode.write(file_desc.offset, buffer).map_err(|_| SyscallError::InvalidArgument)?;
    
    // 更新文件偏移
    file_desc.offset += bytes_written;
    
    Ok(bytes_written)
}

// 关闭文件系统调用
fn sys_close(args: &[usize]) -> Result<usize, SyscallError> {
    let fd = args[0];
    
    // 检查文件描述符
    let current_process = crate::task::process::get_current_process().ok_or(SyscallError::InvalidArgument)?;
    if fd < 3 || fd >= 64 {
        return Err(SyscallError::InvalidArgument);
    }
    
    // 关闭文件
    current_process.file_descriptors[fd] = None;
    
    Ok(0)
}

// 删除文件系统调用
fn sys_unlink(args: &[usize]) -> Result<usize, SyscallError> {
    let path_ptr = args[0];
    
    // 从用户空间读取路径
    let path = unsafe {
        let mut path = String::new();
        let mut ptr = path_ptr as *const u8;
        while *ptr != 0 {
            path.push(*ptr as char);
            ptr = ptr.add(1);
        }
        path
    };
    
    // 删除文件
    let mut fs_service = fs_service().lock();
    fs_service.vfs().unlink(&path).map_err(|_| SyscallError::NotFound)?;
    
    Ok(0)
}

// 等待进程系统调用
fn sys_waitpid(_args: &[usize]) -> Result<usize, SyscallError> {
    // 这里实现等待进程系统调用
    Err(SyscallError::NotSupported)
}

// 内存分配系统调用
fn sys_brk(args: &[usize]) -> Result<usize, SyscallError> {
    let addr = args[0];
    // 简化的内存分配实现
    // 实际需要管理进程的堆空间
    Ok(addr)
}

// 内存映射系统调用
fn sys_mmap(_args: &[usize]) -> Result<usize, SyscallError> {
    // 这里实现内存映射系统调用
    Err(SyscallError::NotSupported)
}

// 解除内存映射系统调用
fn sys_munmap(_args: &[usize]) -> Result<usize, SyscallError> {
    // 这里实现解除内存映射系统调用
    Err(SyscallError::NotSupported)
}
