// C库兼容层

use core::arch::asm;

// 初始化C库兼容层
pub fn init() {
    // 初始化C库兼容层
    // 这里可以添加必要的初始化代码
}

// 字符串长度函数
#[no_mangle]
pub extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    let mut p = s;
    unsafe {
        while *p != 0 {
            len += 1;
            p = p.offset(1);
        }
    }
    len
}

// 字符串复制函数
#[no_mangle]
pub extern "C" fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    let mut d = dest;
    let mut s = src;
    unsafe {
        while *s != 0 {
            *d = *s;
            d = d.offset(1);
            s = s.offset(1);
        }
        *d = 0;
    }
    dest
}

// 字符串比较函数
#[no_mangle]
pub extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut p1 = s1;
    let mut p2 = s2;
    unsafe {
        while *p1 != 0 && *p2 != 0 && *p1 == *p2 {
            p1 = p1.offset(1);
            p2 = p2.offset(1);
        }
        (*p1 as i32) - (*p2 as i32)
    }
}

// 内存复制函数
#[no_mangle]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut d = dest;
    let mut s = src;
    let mut i = 0;
    unsafe {
        while i < n {
            *d = *s;
            d = d.offset(1);
            s = s.offset(1);
            i += 1;
        }
    }
    dest
}

// 内存设置函数
#[no_mangle]
pub extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut p = s;
    let mut i = 0;
    let byte = c as u8;
    unsafe {
        while i < n {
            *p = byte;
            p = p.offset(1);
            i += 1;
        }
    }
    s
}

// 内存比较函数
#[no_mangle]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut p1 = s1;
    let mut p2 = s2;
    let mut i = 0;
    unsafe {
        while i < n && *p1 == *p2 {
            p1 = p1.offset(1);
            p2 = p2.offset(1);
            i += 1;
        }
        if i < n {
            (*p1 as i32) - (*p2 as i32)
        } else {
            0
        }
    }
}

// 退出函数
#[no_mangle]
pub extern "C" fn exit(status: i32) -> ! {
    unsafe {
        asm!(
            "syscall",
            in("rax") 60, // Linux exit系统调用号
            in("rdi") status,
        );
    }
    loop {}
}

// 读取函数
#[no_mangle]
pub extern "C" fn read(fd: i32, buf: *mut u8, count: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 0, // Linux read系统调用号
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rax") result,
        );
    }
    result
}

// 写入函数
#[no_mangle]
pub extern "C" fn write(fd: i32, buf: *const u8, count: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 1, // Linux write系统调用号
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rax") result,
        );
    }
    result
}

// 打开文件函数
#[no_mangle]
pub extern "C" fn open(pathname: *const u8, flags: i32, mode: u32) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 2, // Linux open系统调用号
            in("rdi") pathname,
            in("rsi") flags,
            in("rdx") mode,
            lateout("rax") result,
        );
    }
    result
}

// 关闭文件函数
#[no_mangle]
pub extern "C" fn close(fd: i32) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") 3, // Linux close系统调用号
            in("rdi") fd,
            lateout("rax") result,
        );
    }
    result
}

// 分配内存函数
#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut u8 {
    // 调用墨渊OS的内存分配系统调用
    let result: *mut u8;
    unsafe {
        asm!(
            "syscall",
            in("rax") 9, // mmap系统调用号
            in("rdi") 0, // 地址
            in("rsi") size, // 大小
            in("rdx") 0x3, // PROT_READ | PROT_WRITE
            in("r10") 0x22, // MAP_PRIVATE | MAP_ANONYMOUS
            in("r8") -1, // 文件描述符
            in("r9") 0, // 偏移
            lateout("rax") result,
        );
    }
    result
}

// 释放内存函数
#[no_mangle]
pub extern "C" fn free(ptr: *mut u8) {
    // 调用墨渊OS的内存释放系统调用
    unsafe {
        asm!(
            "syscall",
            in("rax") 11, // munmap系统调用号
            in("rdi") ptr,
            in("rsi") 0, // 暂时使用0，实际应该使用分配的大小
        );
    }
}

// 获取进程ID函数
#[no_mangle]
pub extern "C" fn getpid() -> i32 {
    let result: i32;
    unsafe {
        asm!(
            "syscall",
            in("rax") 39, // Linux getpid系统调用号
            lateout("rax") result,
        );
    }
    result
}

// 获取用户ID函数
#[no_mangle]
pub extern "C" fn getuid() -> i32 {
    let result: i32;
    unsafe {
        asm!(
            "syscall",
            in("rax") 102, // Linux getuid系统调用号
            lateout("rax") result,
        );
    }
    result
}

// 获取组ID函数
#[no_mangle]
pub extern "C" fn getgid() -> i32 {
    let result: i32;
    unsafe {
        asm!(
            "syscall",
            in("rax") 104, // Linux getgid系统调用号
            lateout("rax") result,
        );
    }
    result
}

// 时间函数
#[no_mangle]
pub extern "C" fn time(t: *mut i64) -> i64 {
    let result: i64;
    unsafe {
        asm!(
            "syscall",
            in("rax") 201, // Linux time系统调用号
            in("rdi") t,
            lateout("rax") result,
        );
    }
    result
}

// 系统调用函数
#[no_mangle]
pub extern "C" fn syscall(num: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
            "syscall",
            in("rax") num,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            in("r8") arg5,
            in("r9") arg6,
            lateout("rax") result,
        );
    }
    result
}
