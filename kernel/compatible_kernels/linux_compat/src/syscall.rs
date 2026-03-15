// Linux系统调用兼容层

use core::arch::asm;

// Linux系统调用号
pub const SYS_EXIT: u64 = 60;
pub const SYS_FORK: u64 = 57;
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_WAIT4: u64 = 61;
pub const SYS_BRK: u64 = 12;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_MPROTECT: u64 = 10;
pub const SYS_GETPID: u64 = 39;
pub const SYS_GETPPID: u64 = 64;
pub const SYS_GETUID: u64 = 102;
pub const SYS_GETEUID: u64 = 107;
pub const SYS_GETGID: u64 = 104;
pub const SYS_GETEGID: u64 = 108;
pub const SYS_SETUID: u64 = 105;
pub const SYS_SETGID: u64 = 106;
pub const SYS_SETEUID: u64 = 113;
pub const SYS_SETEGID: u64 = 114;
pub const SYS_IOCTL: u64 = 16;
pub const SYS_STAT: u64 = 4;
pub const SYS_LSTAT: u64 = 6;
pub const SYS_FSTAT: u64 = 5;
pub const SYS_ACCESS: u64 = 21;
pub const SYS_CHMOD: u64 = 90;
pub const SYS_CHOWN: u64 = 92;
pub const SYS_DUP: u64 = 32;
pub const SYS_DUP2: u64 = 33;
pub const SYS_PIPE: u64 = 22;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_CHDIR: u64 = 80;
pub const SYS_GETCWD: u64 = 79;
pub const SYS_MKDIR: u64 = 83;
pub const SYS_RMDIR: u64 = 84;
pub const SYS_UNLINK: u64 = 87;
pub const SYS_RENAME: u64 = 82;
pub const SYS_LINK: u64 = 86;
pub const SYS_SYMLINK: u64 = 88;
pub const SYS_READLINK: u64 = 89;
pub const SYS_POLL: u64 = 7;
pub const SYS_SELECT: u64 = 23;
pub const SYS_GETTIMEOFDAY: u64 = 96;
pub const SYS_SETTIMEOFDAY: u64 = 95;
pub const SYS_TIME: u64 = 201;
pub const SYS_CLOCK_GETTIME: u64 = 228;
pub const SYS_CLOCK_SETTIME: u64 = 229;
pub const SYS_SIGACTION: u64 = 13;
pub const SYS_SIGPROCMASK: u64 = 14;
pub const SYS_SIGPENDING: u64 = 30;
pub const SYS_SIGRETURN: u64 = 15;
pub const SYS_KILL: u64 = 62;
pub const SYS_GETPRIORITY: u64 = 127;
pub const SYS_SETPRIORITY: u64 = 129;
pub const SYS_SCHED_YIELD: u64 = 24;
pub const SYS_SCHED_GETAFFINITY: u64 = 204;
pub const SYS_SCHED_SETAFFINITY: u64 = 203;
pub const SYS_SCHED_GETPARAM: u64 = 201;
pub const SYS_SCHED_SETPARAM: u64 = 202;
pub const SYS_SCHED_GETSCHEDULER: u64 = 144;
pub const SYS_SCHED_SETSCHEDULER: u64 = 145;
pub const SYS_MADVISE: u64 = 28;
pub const SYS_MINCORE: u64 = 29;
pub const SYS_MREMAP: u64 = 25;
pub const SYS_MSYNC: u64 = 26;
pub const SYS_MLOCK: u64 = 152;
pub const SYS_MUNLOCK: u64 = 153;
pub const SYS_MLOCKALL: u64 = 154;
pub const SYS_MUNLOCKALL: u64 = 155;
pub const SYS_GETRLIMIT: u64 = 163;
pub const SYS_SETRESOURCE: u64 = 164;
pub const SYS_GETRUSAGE: u64 = 165;
pub const SYS_TIMES: u64 = 153;
pub const SYS_PTRACE: u64 = 101;
pub const SYS_GETUID32: u64 = 24;
pub const SYS_GETGID32: u64 = 25;
pub const SYS_GETEUID32: u64 = 26;
pub const SYS_GETEGID32: u64 = 27;
pub const SYS_SETUID32: u64 = 28;
pub const SYS_SETGID32: u64 = 29;
pub const SYS_SETEUID32: u64 = 30;
pub const SYS_SETEGID32: u64 = 31;
pub const SYS_GETRESUID: u64 = 117;
pub const SYS_SETRESUID: u64 = 118;
pub const SYS_GETRESGID: u64 = 119;
pub const SYS_SETRESGID: u64 = 120;
pub const SYS_GETGROUPS: u64 = 115;
pub const SYS_SETGROUPS: u64 = 116;
pub const SYS_NEWUNAME: u64 = 63;
pub const SYS_UNAME: u64 = 122;
pub const SYS_GETHOSTNAME: u64 = 160;
pub const SYS_SETHOSTNAME: u64 = 161;
pub const SYS_GETDOMAINNAME: u64 = 166;
pub const SYS_SETDOMAINNAME: u64 = 167;
pub const SYS_GETTID: u64 = 186;
pub const SYS_TGKILL: u64 = 234;
pub const SYS_TIMER_CREATE: u64 = 222;
pub const SYS_TIMER_SETTIME: u64 = 223;
pub const SYS_TIMER_GETTIME: u64 = 224;
pub const SYS_TIMER_DELETE: u64 = 225;
pub const SYS_CLOCK_GETRES: u64 = 227;

// 系统调用处理函数
type SyscallHandler = fn(u64, u64, u64, u64, u64, u64) -> isize;

// 系统调用表
static mut SYSCALL_TABLE: [Option<SyscallHandler>; 256] = [None; 256];

// 初始化系统调用表
pub fn init() {
    unsafe {
        // 注册系统调用处理函数
        SYSCALL_TABLE[SYS_EXIT as usize] = Some(handle_exit);
        SYSCALL_TABLE[SYS_READ as usize] = Some(handle_read);
        SYSCALL_TABLE[SYS_WRITE as usize] = Some(handle_write);
        SYSCALL_TABLE[SYS_OPEN as usize] = Some(handle_open);
        SYSCALL_TABLE[SYS_CLOSE as usize] = Some(handle_close);
        SYSCALL_TABLE[SYS_BRK as usize] = Some(handle_brk);
        SYSCALL_TABLE[SYS_GETPID as usize] = Some(handle_getpid);
        SYSCALL_TABLE[SYS_GETUID as usize] = Some(handle_getuid);
        SYSCALL_TABLE[SYS_GETGID as usize] = Some(handle_getgid);
        // 可以继续添加其他系统调用处理函数
    }
}

// 处理系统调用
pub fn handle_syscall(syscall_num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64, arg6: u64) -> isize {
    unsafe {
        let syscall_table = &raw const SYSCALL_TABLE;
        if (syscall_num as usize) < (*syscall_table).len() {
            let handler = &raw const (*syscall_table)[syscall_num as usize];
            if let Some(handler) = *handler {
                handler(arg1, arg2, arg3, arg4, arg5, arg6)
            } else {
                -1 // 系统调用未实现
            }
        } else {
            -1 // 系统调用号无效
        }
    }
}

// 系统调用处理函数实现
fn handle_exit(arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的exit系统调用
    unsafe {
        asm!(
            "syscall",
            in("rax") 0, // exit系统调用号
            in("rdi") arg1,
        );
    }
    0
}

fn handle_read(arg1: u64, arg2: u64, arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的read系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 4, // read系统调用号
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") result,
        );
    }
    result
}

fn handle_write(arg1: u64, arg2: u64, arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的write系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 5, // write系统调用号
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") result,
        );
    }
    result
}

fn handle_open(arg1: u64, arg2: u64, arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的open系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 2, // open系统调用号
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") result,
        );
    }
    result
}

fn handle_close(arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的close系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 3, // close系统调用号
            in("rdi") arg1,
            lateout("rax") result,
        );
    }
    result
}

fn handle_brk(arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的brk系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 8, // brk系统调用号
            in("rdi") arg1,
            lateout("rax") result,
        );
    }
    result
}

fn handle_getpid(_arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的getpid系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 6, // getpid系统调用号
            lateout("rax") result,
        );
    }
    result
}

fn handle_getuid(_arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的getuid系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 7, // getuid系统调用号
            lateout("rax") result,
        );
    }
    result
}

fn handle_getgid(_arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64, _arg5: u64, _arg6: u64) -> isize {
    // 调用墨渊OS的getgid系统调用
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 7, // getgid系统调用号
            lateout("rax") result,
        );
    }
    result
}
