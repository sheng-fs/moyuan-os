#![no_std]
#![feature(c_variadic)]
#![allow(unexpected_cfgs)]

// 系统调用号定义
pub const SYS_EXIT: usize = 0;
pub const SYS_FORK: usize = 1;
pub const SYS_EXEC: usize = 2;
pub const SYS_OPEN: usize = 3;
pub const SYS_READ: usize = 4;
pub const SYS_WRITE: usize = 5;
pub const SYS_CLOSE: usize = 6;
pub const SYS_WAITPID: usize = 7;
pub const SYS_BRK: usize = 8;
pub const SYS_MMAP: usize = 9;
pub const SYS_MUNMAP: usize = 10;
pub const SYS_UNLINK: usize = 11;

// 文件操作标志
pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32 = 2;
pub const O_CREAT: u32 = 64;
pub const O_TRUNC: u32 = 512;
pub const O_APPEND: u32 = 1024;

// 标准文件描述符
pub const STDIN_FILENO: i32 = 0;
pub const STDOUT_FILENO: i32 = 1;
pub const STDERR_FILENO: i32 = 2;

// 系统调用函数
/// # Safety
///
/// 调用者必须确保系统调用号和参数是有效的
#[inline(always)]
pub unsafe fn syscall0(number: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        in("rax") number,
        out("rcx") _, // syscall 会修改 rcx
        out("r11") _, // syscall 会修改 r11
        lateout("rax") ret,
        options(nostack, preserves_flags),
    );
    ret
}

/// # Safety
///
/// 调用者必须确保系统调用号和参数是有效的
#[inline(always)]
pub unsafe fn syscall1(number: usize, arg1: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        in("rax") number,
        in("rdi") arg1,
        out("rcx") _, // syscall 会修改 rcx
        out("r11") _, // syscall 会修改 r11
        lateout("rax") ret,
        options(nostack, preserves_flags),
    );
    ret
}

/// # Safety
///
/// 调用者必须确保系统调用号和参数是有效的
#[inline(always)]
pub unsafe fn syscall2(number: usize, arg1: usize, arg2: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        out("rcx") _, // syscall 会修改 rcx
        out("r11") _, // syscall 会修改 r11
        lateout("rax") ret,
        options(nostack, preserves_flags),
    );
    ret
}

/// # Safety
///
/// 调用者必须确保系统调用号和参数是有效的
#[inline(always)]
pub unsafe fn syscall3(number: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rcx") _, // syscall 会修改 rcx
        out("r11") _, // syscall 会修改 r11
        lateout("rax") ret,
        options(nostack, preserves_flags),
    );
    ret
}

// 标准库函数封装

// 退出进程
#[no_mangle]
pub extern "C" fn exit(status: i32) -> ! {
    unsafe {
        syscall1(SYS_EXIT, status as usize);
        panic!("syscall exit failed");
    }
}

// 打开文件
/// # Safety
///
/// 调用者必须确保 path 是有效的以零结尾的字符串指针
#[no_mangle]
pub extern "C" fn open(path: *const u8, flags: i32, mode: u32) -> i32 {
    unsafe {
        syscall3(SYS_OPEN, path as usize, flags as usize, mode as usize) as i32
    }
}

// 读取文件
/// # Safety
///
/// 调用者必须确保 buf 是有效的指针，并且有足够的空间存储 count 字节
#[no_mangle]
pub extern "C" fn read(fd: i32, buf: *mut u8, count: usize) -> isize {
    unsafe {
        syscall3(SYS_READ, fd as usize, buf as usize, count)
    }
}

// 写入文件
/// # Safety
///
/// 调用者必须确保 buf 是有效的指针，指向 count 字节的有效数据
#[no_mangle]
pub extern "C" fn write(fd: i32, buf: *const u8, count: usize) -> isize {
    unsafe {
        syscall3(SYS_WRITE, fd as usize, buf as usize, count)
    }
}

// 关闭文件
#[no_mangle]
pub extern "C" fn close(fd: i32) -> i32 {
    unsafe {
        syscall1(SYS_CLOSE, fd as usize) as i32
    }
}

// 删除文件
/// # Safety
///
/// 调用者必须确保 path 是有效的以零结尾的字符串指针
#[no_mangle]
pub extern "C" fn unlink(path: *const u8) -> i32 {
    unsafe {
        syscall1(SYS_UNLINK, path as usize) as i32
    }
}

// 内存分配
/// # Safety
///
/// 调用者必须确保 addr 是有效的指针或 null
#[no_mangle]
pub extern "C" fn brk(addr: *mut u8) -> *mut u8 {
    unsafe {
        syscall1(SYS_BRK, addr as usize) as *mut u8
    }
}

// 字符串函数
/// # Safety
///
/// 调用者必须确保 s 是有效的以零结尾的字符串指针
#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// # Safety
///
/// 调用者必须确保 s1 和 s2 是有效的以零结尾的字符串指针
#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as i32) - (b as i32);
        }
        if a == 0 {
            return 0;
        }
        i += 1;
    }
}

/// # Safety
///
/// 调用者必须确保 dest 有足够的空间，且 src 是有效的以零结尾的字符串指针
#[no_mangle]
pub unsafe extern "C" fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    dest
}

// 内存函数
/// # Safety
///
/// 调用者必须确保 dest 和 src 是有效的指针，且不重叠
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    core::ptr::copy(src, dest, n);
    dest
}

/// # Safety
///
/// 调用者必须确保 s 是有效的指针，指向 n 字节的可写内存
#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    core::ptr::write_bytes(s, c as u8, n);
    s
}

// 标准输入输出函数
/// # Safety
///
/// 调用者必须确保 s 是有效的以零结尾的字符串指针
#[no_mangle]
pub unsafe extern "C" fn puts(s: *const u8) -> i32 {
    let len = strlen(s);
    let mut buf = [0u8; 1024];
    strcpy(buf.as_mut_ptr(), s);
    buf[len] = b'\n';
    write(STDOUT_FILENO, buf.as_ptr(), len + 1) as i32
}

/// # Safety
///
/// 调用者必须确保 format 是有效的以零结尾的字符串指针
#[no_mangle]
pub unsafe extern "C" fn printf(format: *const u8, _: ...) -> i32 {
    // 简化实现，仅支持 %s 和 %d
    let mut args = core::ptr::addr_of!(format).add(1) as *const usize;
    let mut fmt = format;
    let mut buf = [0u8; 1024];
    let mut pos = 0;
    
    while *fmt != 0 {
        if *fmt == b'%' {
            fmt = fmt.add(1);
            match *fmt {
                b's' => {
                    let s = *args as *const u8;
                    let len = strlen(s);
                    if pos + len < buf.len() {
                        memcpy(buf.as_mut_ptr().add(pos), s, len);
                        pos += len;
                    }
                    args = args.add(1);
                }
                b'd' => {
                    let num = *args as i32;
                    let mut num_str = [0u8; 20];
                    let mut num_pos = 0;
                    let mut n = num;
                    if n < 0 {
                        num_str[num_pos] = b'-';
                        num_pos += 1;
                        n = -n;
                    }
                    let mut digits = 0;
                    let mut temp = n;
                    while temp > 0 {
                        temp /= 10;
                        digits += 1;
                    }
                    if digits == 0 {
                        num_str[num_pos] = b'0';
                        num_pos += 1;
                    } else {
                        temp = n;
                        for i in (0..digits).rev() {
                            num_str[num_pos + i] = (temp % 10) as u8 + b'0';
                            temp /= 10;
                        }
                        num_pos += digits;
                    }
                    if pos + num_pos < buf.len() {
                        memcpy(buf.as_mut_ptr().add(pos), num_str.as_ptr(), num_pos);
                        pos += num_pos;
                    }
                    args = args.add(1);
                }
                _ => {}
            }
        } else {
            buf[pos] = *fmt;
            pos += 1;
        }
        fmt = fmt.add(1);
    }
    write(STDOUT_FILENO, buf.as_ptr(), pos) as i32
}
