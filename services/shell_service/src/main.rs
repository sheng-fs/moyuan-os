#![no_std]
#![cfg_attr(not(test), no_main)]
#![allow(unexpected_cfgs)]

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

// 导入libc函数
extern "C" {
    fn exit(status: i32) -> !;
    fn printf(format: *const u8, ...) -> i32;
    fn open(path: *const u8, flags: i32, mode: u32) -> i32;
    fn close(fd: i32) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

// 定义标准文件描述符
const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;

// 文件操作标志
const O_RDONLY: i32 = 0;
const O_WRONLY: i32 = 1;
const O_CREAT: i32 = 64;
const O_TRUNC: i32 = 512;

// 权限标志
const S_IRUSR: u32 = 0o400;
const S_IWUSR: u32 = 0o200;
const S_IRGRP: u32 = 0o040;
const S_IROTH: u32 = 0o004;

// 打印字符串
fn print_str(s: &str) {
    unsafe {
        write(STDOUT_FILENO, s.as_ptr(), s.len());
    }
}

// 打印格式化字符串
macro_rules! println {
    ($fmt:expr) => {
        unsafe {
            printf(concat!($fmt, "\n").as_ptr() as *const u8);
        }
    };
    ($fmt:expr, $($arg:expr),*) => {
        unsafe {
            printf(concat!($fmt, "\n").as_ptr() as *const u8, $($arg),*);
        }
    };
}

// 读取一行输入
fn read_line() -> String {
    let mut buf = [0u8; 256];
    let mut len = 0;
    
    unsafe {
        while len < buf.len() - 1 {
            let n = read(STDIN_FILENO, buf.as_mut_ptr().offset(len as isize), 1);
            if n <= 0 {
                break;
            }
            
            if buf[len] == b'\n' {
                break;
            }
            
            len += 1;
        }
    }
    
    String::from_utf8_lossy(&buf[..len]).to_string()
}

// 命令执行结果
enum CommandResult {
    Success,
    Failure(i32),
    Exit,
}

// 命令执行器 trait
trait CommandExecutor {
    fn execute(&self, args: &[&str]) -> CommandResult;
}

// 内置命令实现
struct LsCommand;
impl CommandExecutor for LsCommand {
    fn execute(&self, _args: &[&str]) -> CommandResult {
        println!("Directory contents:");
        println!("  .");
        println!("  ..");
        println!("  bin");
        println!("  dev");
        println!("  etc");
        println!("  home");
        println!("  lib");
        println!("  proc");
        println!("  sys");
        println!("  tmp");
        CommandResult::Success
    }
}

struct EchoCommand;
impl CommandExecutor for EchoCommand {
    fn execute(&self, args: &[&str]) -> CommandResult {
        let output = args.join(" ");
        print_str(&output);
        print_str("\n");
        CommandResult::Success
    }
}

struct CatCommand;
impl CommandExecutor for CatCommand {
    fn execute(&self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            println!("Usage: cat <file>");
            return CommandResult::Failure(1);
        }
        
        for file in args {
            unsafe {
                let fd = open(file.as_ptr(), O_RDONLY as i32, 0);
                if fd < 0 {
                    printf(concat!("cat: {}: No such file or directory", "\n").as_ptr() as *const u8, file.as_ptr());
                    continue;
                }
                
                let mut buf = [0u8; 1024];
                loop {
                    let n = read(fd, buf.as_mut_ptr(), buf.len());
                    if n <= 0 {
                        break;
                    }
                    write(STDOUT_FILENO, buf.as_ptr(), n as usize);
                }
                
                close(fd);
            }
        }
        
        CommandResult::Success
    }
}

struct ExitCommand;
impl CommandExecutor for ExitCommand {
    fn execute(&self, _args: &[&str]) -> CommandResult {
        CommandResult::Exit
    }
}

// Shell结构体
struct Shell {
    commands: BTreeMap<String, Box<dyn CommandExecutor>>,
    history: Vec<String>,
}

impl Shell {
    fn new() -> Self {
        let mut commands: BTreeMap<String, Box<dyn CommandExecutor>> = BTreeMap::new();
        
        // 注册内置命令
        commands.insert("ls".to_string(), Box::new(LsCommand) as Box<dyn CommandExecutor>);
        commands.insert("echo".to_string(), Box::new(EchoCommand) as Box<dyn CommandExecutor>);
        commands.insert("cat".to_string(), Box::new(CatCommand) as Box<dyn CommandExecutor>);
        commands.insert("exit".to_string(), Box::new(ExitCommand) as Box<dyn CommandExecutor>);
        
        Self {
            commands,
            history: Vec::new(),
        }
    }
    
    // 解析命令行
    fn parse_command<'a>(&self, input: &'a str) -> Option<(String, Vec<&'a str>, Option<String>)> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }
        
        // 检查是否有输出重定向
        let mut parts: Vec<&'a str> = input.split('>').collect();
        let redirect = if parts.len() > 1 {
            Some(parts.pop().unwrap().trim().to_string())
        } else {
            None
        };
        
        let command_part = parts[0].trim();
        let tokens: Vec<&'a str> = command_part.split_whitespace().collect();
        
        if tokens.is_empty() {
            return None;
        }
        
        let command = tokens[0].to_string();
        let args = tokens[1..].to_vec();
        
        Some((command, args, redirect))
    }
    
    // 执行命令
    fn execute_command(&mut self, command: &str, args: &[&str], redirect: &Option<String>) -> CommandResult {
        // 保存到历史记录
        // 简单的字符串拼接
        let mut history_entry = String::new();
        history_entry.push_str(command);
        history_entry.push(' ');
        for arg in args {
            history_entry.push_str(arg);
            history_entry.push(' ');
        }
        self.history.push(history_entry);
        
        // 处理输出重定向
        let original_stdout = if let Some(ref file) = redirect {
            unsafe {
                let fd = open(file.as_ptr(), (O_WRONLY | O_CREAT | O_TRUNC) as i32, 
                    S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);
                if fd < 0 {
                    printf(concat!("Error: Cannot open file {}", "\n").as_ptr() as *const u8, file.as_ptr());
                    return CommandResult::Failure(1);
                }
                // 这里应该保存原始stdout并重定向，简化处理
                fd
            }
        } else {
            -1
        };
        
        // 执行命令
        let result = if let Some(executor) = self.commands.get(command) {
            executor.execute(args)
        } else {
            // 尝试执行外部命令
            println!("Command not found: {}", command);
            CommandResult::Failure(127)
        };
        
        // 恢复stdout
        if original_stdout != -1 {
            unsafe {
                close(original_stdout);
            }
        }
        
        result
    }
    
    // 主循环
    fn run(&mut self) {
        print_str("MOYUAN OS Shell\n");
        print_str("Type 'exit' to quit\n");
        print_str("Available commands: ls, echo, cat, exit\n\n");
        
        loop {
            // 显示提示符
            print_str("moyuan$ ");
            
            // 读取输入
            let input = read_line();
            
            // 解析命令
            if let Some((command, args, redirect)) = self.parse_command(&input) {
                // 复制值以避免借用冲突
                let command_copy = command.clone();
                let args_copy = args.iter().map(|&s| s).collect::<Vec<&str>>();
                let redirect_copy = redirect.clone();
                
                // 执行命令
                match self.execute_command(&command_copy, &args_copy, &redirect_copy) {
                    CommandResult::Exit => {
                        print_str("Exiting shell...\n");
                        break;
                    }
                    CommandResult::Failure(code) => {
                        print_str("Command failed with exit code: ");
                        // 简单的数字转字符串
                        let mut code_str = [0u8; 10];
                        let mut i = 0;
                        let mut num = code;
                        let is_negative = num < 0;
                        if is_negative {
                            num = -num;
                        }
                        if num == 0 {
                            code_str[0] = b'0';
                            i = 1;
                        } else {
                            while num > 0 && i < code_str.len() {
                                code_str[code_str.len() - 1 - i] = (num % 10) as u8 + b'0';
                                num /= 10;
                                i += 1;
                            }
                        }
                        if is_negative && i < code_str.len() {
                            code_str[code_str.len() - 1 - i] = b'-';
                            i += 1;
                        }
                        print_str(core::str::from_utf8(&code_str[code_str.len() - i..]).unwrap());
                        print_str("\n");
                    }
                    _ => {}
                }
            }
        }
    }
}

// 主函数
#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut shell = Shell::new();
    shell.run();
    
    unsafe {
        exit(0);
    }
}



// 分配器
use alloc::alloc::GlobalAlloc;
use core::ptr::null_mut;

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
    }
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

// panic 处理函数
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
