extern crate alloc;

// 导入IPC模块


use crate::power;

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
    // IPC系统调用
    Pipe = 12,           // 创建匿名管道
    ShmCreate = 13,      // 创建共享内存
    ShmOpen = 14,        // 打开共享内存
    ShmMap = 15,         // 映射共享内存
    ShmUnmap = 16,       // 解除共享内存映射
    ShmClose = 17,       // 关闭共享内存
    SemCreate = 18,      // 创建信号量
    SemOpen = 19,        // 打开信号量
    SemP = 20,           // 信号量P操作
    SemV = 21,           // 信号量V操作
    SemClose = 22,       // 关闭信号量
    // 安全相关系统调用
    GetUid = 23,         // 获取当前进程的UID
    GetGid = 24,         // 获取当前进程的GID
    SetUid = 25,         // 设置当前进程的UID
    SetGid = 26,         // 设置当前进程的GID
    GetEuid = 27,        // 获取当前进程的有效UID
    GetEgid = 28,        // 获取当前进程的有效GID
    SetEuid = 29,        // 设置当前进程的有效UID
    SetEgid = 30,        // 设置当前进程的有效GID
    
    // 电源管理相关系统调用
    PowerManagement = 31,    // 电源管理
    CpuFrequency = 32,       // CPU频率调节
    DevicePowerManagement = 33, // 设备电源管理
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
#[allow(dead_code)]
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
static mut SYSCALL_TABLE: [Option<SyscallHandler>; 34] = [
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
    // IPC系统调用
    Some(sys_pipe),        // 12
    Some(sys_shm_create),  // 13
    Some(sys_shm_open),    // 14
    Some(sys_shm_map),     // 15
    Some(sys_shm_unmap),   // 16
    Some(sys_shm_close),   // 17
    Some(sys_sem_create),  // 18
    Some(sys_sem_open),    // 19
    Some(sys_sem_p),       // 20
    Some(sys_sem_v),       // 21
    Some(sys_sem_close),   // 22
    // 安全相关系统调用
    Some(sys_getuid),      // 23
    Some(sys_getgid),      // 24
    Some(sys_setuid),      // 25
    Some(sys_setgid),      // 26
    Some(sys_geteuid),     // 27
    Some(sys_getegid),     // 28
    Some(sys_seteuid),     // 29
    Some(sys_setegid),     // 30
    // 电源管理相关系统调用
    Some(sys_power_management),  // 31
    Some(sys_cpu_frequency),     // 32
    Some(sys_device_power_management), // 33
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
            // IPC系统调用
            12 => Ok(Syscall::Pipe),
            13 => Ok(Syscall::ShmCreate),
            14 => Ok(Syscall::ShmOpen),
            15 => Ok(Syscall::ShmMap),
            16 => Ok(Syscall::ShmUnmap),
            17 => Ok(Syscall::ShmClose),
            18 => Ok(Syscall::SemCreate),
            19 => Ok(Syscall::SemOpen),
            20 => Ok(Syscall::SemP),
            21 => Ok(Syscall::SemV),
            22 => Ok(Syscall::SemClose),
            // 安全相关系统调用
            23 => Ok(Syscall::GetUid),
            24 => Ok(Syscall::GetGid),
            25 => Ok(Syscall::SetUid),
            26 => Ok(Syscall::SetGid),
            27 => Ok(Syscall::GetEuid),
            28 => Ok(Syscall::GetEgid),
            29 => Ok(Syscall::SetEuid),
            30 => Ok(Syscall::SetEgid),
            // 电源管理相关系统调用
            31 => Ok(Syscall::PowerManagement),
            32 => Ok(Syscall::CpuFrequency),
            33 => Ok(Syscall::DevicePowerManagement),
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
    if !(3..64).contains(&fd) {
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

// 创建匿名管道系统调用
fn sys_pipe(args: &[usize]) -> Result<usize, SyscallError> {
    let pipefd = args[0]; // 指向两个文件描述符的数组
    
    match crate::ipc::pipe::create_anonymous_pipe() {
        Ok((read_fd, write_fd)) => {
            // 将文件描述符写入用户空间
            unsafe {
                let pipefd_ptr = pipefd as *mut [usize; 2];
                (*pipefd_ptr)[0] = read_fd;
                (*pipefd_ptr)[1] = write_fd;
            }
            Ok(0)
        },
        Err(_) => Err(SyscallError::ResourceBusy),
    }
}

// 创建共享内存系统调用
fn sys_shm_create(args: &[usize]) -> Result<usize, SyscallError> {
    let size = args[0];
    let name_ptr = args[1];
    
    let name = if name_ptr != 0 {
        unsafe {
            let mut name_str = alloc::string::String::new();
            let mut ptr = name_ptr as *const u8;
            while *ptr != 0 {
                name_str.push(*ptr as char);
                ptr = ptr.add(1);
            }
            Some(name_str)
        }
    } else {
        None
    };
    
    match crate::ipc::shared_memory::create_shared_memory(size, name.as_deref()) {
        Ok(shm_id) => Ok(shm_id),
        Err(_) => Err(SyscallError::ResourceBusy),
    }
}

// 打开共享内存系统调用
fn sys_shm_open(args: &[usize]) -> Result<usize, SyscallError> {
    let name_ptr = args[0];
    
    let name = unsafe {
        let mut name = alloc::string::String::new();
        let mut ptr = name_ptr as *const u8;
        while *ptr != 0 {
            name.push(*ptr as char);
            ptr = ptr.add(1);
        }
        name
    };
    
    match crate::ipc::shared_memory::open_shared_memory(&name) {
        Ok(shm_id) => Ok(shm_id),
        Err(_) => Err(SyscallError::NotFound),
    }
}

// 映射共享内存系统调用
fn sys_shm_map(args: &[usize]) -> Result<usize, SyscallError> {
    let shm_id = args[0];
    
    // 获取当前进程PID
    let current_pid = crate::task::process::get_current_pid().ok_or(SyscallError::InvalidArgument)?;
    
    match crate::ipc::shared_memory::map_shared_memory(shm_id, current_pid) {
        Ok(virtual_address) => Ok(virtual_address),
        Err(_) => Err(SyscallError::ResourceBusy),
    }
}

// 解除共享内存映射系统调用
fn sys_shm_unmap(args: &[usize]) -> Result<usize, SyscallError> {
    let shm_id = args[0];
    let virtual_address = args[1];
    
    // 获取当前进程PID
    let current_pid = crate::task::process::get_current_pid().ok_or(SyscallError::InvalidArgument)?;
    
    match crate::ipc::shared_memory::unmap_shared_memory(shm_id, current_pid, virtual_address) {
        Ok(()) => Ok(0),
        Err(_) => Err(SyscallError::InvalidArgument),
    }
}

// 关闭共享内存系统调用
fn sys_shm_close(args: &[usize]) -> Result<usize, SyscallError> {
    let shm_id = args[0];
    
    match crate::ipc::shared_memory::close_shared_memory(shm_id) {
        Ok(()) => Ok(0),
        Err(_) => Err(SyscallError::InvalidArgument),
    }
}

// 创建信号量系统调用
fn sys_sem_create(args: &[usize]) -> Result<usize, SyscallError> {
    let value = args[0] as isize;
    let name_ptr = args[1];
    
    let name = if name_ptr != 0 {
        unsafe {
            let mut name_str = alloc::string::String::new();
            let mut ptr = name_ptr as *const u8;
            while *ptr != 0 {
                name_str.push(*ptr as char);
                ptr = ptr.add(1);
            }
            Some(name_str)
        }
    } else {
        None
    };
    
    match crate::ipc::semaphore::create_semaphore(value, name.as_deref()) {
        Ok(sem_id) => Ok(sem_id),
        Err(_) => Err(SyscallError::ResourceBusy),
    }
}

// 打开信号量系统调用
fn sys_sem_open(args: &[usize]) -> Result<usize, SyscallError> {
    let name_ptr = args[0];
    
    let name = unsafe {
        let mut name = alloc::string::String::new();
        let mut ptr = name_ptr as *const u8;
        while *ptr != 0 {
            name.push(*ptr as char);
            ptr = ptr.add(1);
        }
        name
    };
    
    match crate::ipc::semaphore::open_semaphore(&name) {
        Ok(sem_id) => Ok(sem_id),
        Err(_) => Err(SyscallError::NotFound),
    }
}

// 信号量P操作系统调用
fn sys_sem_p(args: &[usize]) -> Result<usize, SyscallError> {
    let sem_id = args[0];
    
    match crate::ipc::semaphore::semaphore_p(sem_id) {
        Ok(()) => Ok(0),
        Err(_) => Err(SyscallError::InvalidArgument),
    }
}

// 信号量V操作系统调用
fn sys_sem_v(args: &[usize]) -> Result<usize, SyscallError> {
    let sem_id = args[0];
    
    match crate::ipc::semaphore::semaphore_v(sem_id) {
        Ok(()) => Ok(0),
        Err(_) => Err(SyscallError::InvalidArgument),
    }
}

// 关闭信号量系统调用
fn sys_sem_close(args: &[usize]) -> Result<usize, SyscallError> {
    let sem_id = args[0];
    
    match crate::ipc::semaphore::close_semaphore(sem_id) {
        Ok(()) => Ok(0),
        Err(_) => Err(SyscallError::InvalidArgument),
    }
}

// 获取当前进程的UID系统调用
fn sys_getuid(_args: &[usize]) -> Result<usize, SyscallError> {
    if let Some(current_process) = crate::task::process::get_current_process() {
        Ok(current_process.uid)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 获取当前进程的GID系统调用
fn sys_getgid(_args: &[usize]) -> Result<usize, SyscallError> {
    if let Some(current_process) = crate::task::process::get_current_process() {
        Ok(current_process.gid)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 设置当前进程的UID系统调用
fn sys_setuid(args: &[usize]) -> Result<usize, SyscallError> {
    let uid = args[0];
    
    if let Some(current_process) = crate::task::process::get_current_process() {
        // 只有root用户可以设置UID
        if current_process.uid != 0 {
            return Err(SyscallError::PermissionDenied);
        }
        
        current_process.uid = uid;
        current_process.euid = uid;
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 设置当前进程的GID系统调用
fn sys_setgid(args: &[usize]) -> Result<usize, SyscallError> {
    let gid = args[0];
    
    if let Some(current_process) = crate::task::process::get_current_process() {
        // 只有root用户可以设置GID
        if current_process.uid != 0 {
            return Err(SyscallError::PermissionDenied);
        }
        
        current_process.gid = gid;
        current_process.egid = gid;
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 获取当前进程的有效UID系统调用
fn sys_geteuid(_args: &[usize]) -> Result<usize, SyscallError> {
    if let Some(current_process) = crate::task::process::get_current_process() {
        Ok(current_process.euid)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 获取当前进程的有效GID系统调用
fn sys_getegid(_args: &[usize]) -> Result<usize, SyscallError> {
    if let Some(current_process) = crate::task::process::get_current_process() {
        Ok(current_process.egid)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 设置当前进程的有效UID系统调用
fn sys_seteuid(args: &[usize]) -> Result<usize, SyscallError> {
    let euid = args[0];
    
    if let Some(current_process) = crate::task::process::get_current_process() {
        // 只有root用户或进程本身可以设置有效UID
        if current_process.uid != 0 && current_process.uid != euid {
            return Err(SyscallError::PermissionDenied);
        }
        
        current_process.euid = euid;
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 设置当前进程的有效GID系统调用
fn sys_setegid(args: &[usize]) -> Result<usize, SyscallError> {
    let egid = args[0];
    
    if let Some(current_process) = crate::task::process::get_current_process() {
        // 只有root用户或进程本身所属组可以设置有效GID
        if current_process.uid != 0 && !crate::security::user_group::is_user_in_group(current_process.uid as u32, egid as u32) {
            return Err(SyscallError::PermissionDenied);
        }
        
        current_process.egid = egid;
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}

// 电源管理系统调用
fn sys_power_management(args: &[usize]) -> Result<usize, SyscallError> {
    let cmd = args[0];
    let arg1 = args[1];
    
    let result = power::syscall_power_management(cmd as u64, arg1 as u64, 0, 0, 0, 0, 0);
    if result < 0 {
        Err(SyscallError::NotSupported)
    } else {
        Ok(result as usize)
    }
}

// CPU频率调节系统调用
fn sys_cpu_frequency(args: &[usize]) -> Result<usize, SyscallError> {
    let cmd = args[0];
    let arg1 = args[1];
    
    let result = power::cpu::syscall_cpu_frequency(cmd as u64, arg1 as u64, 0, 0, 0, 0, 0);
    if result < 0 {
        Err(SyscallError::NotSupported)
    } else {
        Ok(result as usize)
    }
}

// 设备电源管理系统调用
fn sys_device_power_management(args: &[usize]) -> Result<usize, SyscallError> {
    let cmd = args[0];
    let arg1 = args[1];
    let arg2 = args[2];
    
    let result = power::device::syscall_device_power_management(cmd as u64, arg1 as u64, arg2 as u64, 0, 0, 0, 0);
    if result < 0 {
        Err(SyscallError::NotSupported)
    } else {
        Ok(result as usize)
    }
}
