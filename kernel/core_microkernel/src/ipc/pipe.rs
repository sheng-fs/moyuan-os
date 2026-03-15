// 管道通信模块

extern crate alloc;

use alloc::{vec::Vec, sync::Arc, string::String};
use spin::Mutex;

// 导入IPC错误类型和结果类型
use super::IpcResult;

const PIPE_BUFFER_SIZE: usize = 4096; // 4KB缓冲区

// 管道方向
#[derive(Debug, Clone, Copy, PartialEq)]
enum PipeDirection {
    Read,
    Write,
}

// 管道状态
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum PipeState {
    Open,
    Closed,
}

// 管道缓冲区
#[allow(dead_code)]
struct PipeBuffer {
    data: [u8; PIPE_BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
    size: usize,
    state: PipeState,
}

#[allow(dead_code)]
impl PipeBuffer {
    fn new() -> Self {
        Self {
            data: [0; PIPE_BUFFER_SIZE],
            read_pos: 0,
            write_pos: 0,
            size: 0,
            state: PipeState::Open,
        }
    }

    fn is_full(&self) -> bool {
        self.size == PIPE_BUFFER_SIZE
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn write(&mut self, data: &[u8]) -> usize {
        if self.is_full() || self.state == PipeState::Closed {
            return 0;
        }

        let mut written = 0;
        let remaining = PIPE_BUFFER_SIZE - self.size;
        let write_size = data.len().min(remaining);

        // 分两部分写入
        let first_part = (PIPE_BUFFER_SIZE - self.write_pos).min(write_size);
        let second_part = write_size - first_part;

        // 写入第一部分
        self.data[self.write_pos..self.write_pos + first_part].copy_from_slice(&data[0..first_part]);
        self.write_pos = (self.write_pos + first_part) % PIPE_BUFFER_SIZE;
        written += first_part;

        // 写入第二部分（如果需要）
        if second_part > 0 {
            self.data[0..second_part].copy_from_slice(&data[first_part..first_part + second_part]);
            self.write_pos = second_part;
            written += second_part;
        }

        self.size += written;
        written
    }

    fn read(&mut self, buf: &mut [u8]) -> usize {
        if self.is_empty() || self.state == PipeState::Closed {
            return 0;
        }

        let mut read = 0;
        let read_size = buf.len().min(self.size);

        // 分两部分读取
        let first_part = (PIPE_BUFFER_SIZE - self.read_pos).min(read_size);
        let second_part = read_size - first_part;

        // 读取第一部分
        buf[0..first_part].copy_from_slice(&self.data[self.read_pos..self.read_pos + first_part]);
        self.read_pos = (self.read_pos + first_part) % PIPE_BUFFER_SIZE;
        read += first_part;

        // 读取第二部分（如果需要）
        if second_part > 0 {
            buf[first_part..first_part + second_part].copy_from_slice(&self.data[0..second_part]);
            self.read_pos = second_part;
            read += second_part;
        }

        self.size -= read;
        read
    }

    fn close(&mut self) {
        self.state = PipeState::Closed;
    }

    fn is_closed(&self) -> bool {
        self.state == PipeState::Closed
    }
}

// 管道结构
#[allow(dead_code)]
struct Pipe {
    buffer: Arc<Mutex<PipeBuffer>>,
    direction: PipeDirection,
    name: Option<String>,
}

#[allow(dead_code)]
impl Pipe {
    fn new(direction: PipeDirection, name: Option<String>) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(PipeBuffer::new())),
            direction,
            name,
        }
    }

    fn write(&self, data: &[u8]) -> usize {
        if self.direction != PipeDirection::Write {
            return 0;
        }
        self.buffer.lock().write(data)
    }

    fn read(&self, buf: &mut [u8]) -> usize {
        if self.direction != PipeDirection::Read {
            return 0;
        }
        self.buffer.lock().read(buf)
    }

    fn close(&self) {
        self.buffer.lock().close();
    }
}

// 管道表
static mut PIPES: Vec<Option<Pipe>> = Vec::new();
static mut NEXT_PIPE_ID: usize = 0;

// 匿名管道表
static mut ANONYMOUS_PIPES: Vec<Option<(Arc<Mutex<PipeBuffer>>, usize, usize)>> = Vec::new();
static mut NEXT_ANONYMOUS_PIPE_ID: usize = 0;

// 初始化管道模块
pub fn init() {
    unsafe {
        let pipes = &raw mut PIPES;
        let anonymous_pipes = &raw mut ANONYMOUS_PIPES;
        (*pipes).reserve(64);
        (*anonymous_pipes).reserve(64);
    }
}

// 创建匿名管道
pub fn create_anonymous_pipe() -> IpcResult<(usize, usize)> {
    unsafe {
        let next_pipe_id = &raw mut NEXT_ANONYMOUS_PIPE_ID;
        let pipe_id = *next_pipe_id;
        if pipe_id >= 64 {
            return Err(super::IpcError::ResourceBusy);
        }
        *next_pipe_id += 1;

        let buffer = Arc::new(Mutex::new(PipeBuffer::new()));
        let read_fd = allocate_file_descriptor(PipeDirection::Read, None, Some(buffer.clone()))?;
        let write_fd = allocate_file_descriptor(PipeDirection::Write, None, Some(buffer.clone()))?;

        let anonymous_pipes = &raw mut ANONYMOUS_PIPES;
        (*anonymous_pipes).push(Some((buffer, read_fd, write_fd)));
        Ok((read_fd, write_fd))
    }
}

// 创建命名管道
#[allow(dead_code)]
pub fn create_named_pipe(name: &str) -> IpcResult<usize> {
    unsafe {
        // 检查管道名是否已存在
        let pipes = &raw const PIPES;
        for pipe in &(*pipes) {
            if let Some(pipe) = pipe {
                if let Some(pipe_name) = &pipe.name {
                    if pipe_name == name {
                        return Err(super::IpcError::ResourceBusy);
                    }
                }
            }
        }

        allocate_file_descriptor(PipeDirection::Write, Some(String::from(name)), None)
    }
}

// 打开命名管道
#[allow(dead_code)]
pub fn open_named_pipe(name: &str) -> IpcResult<usize> {
    unsafe {
        let pipes = &raw const PIPES;
        for (_i, pipe) in (*pipes).iter().enumerate() {
            if let Some(pipe) = pipe {
                if let Some(pipe_name) = &pipe.name {
                    if pipe_name == name {
                        return allocate_file_descriptor(PipeDirection::Read, Some(String::from(name)), Some(pipe.buffer.clone()));
                    }
                }
            }
        }
        Err(super::IpcError::NotFound)
    }
}

// 分配文件描述符
fn allocate_file_descriptor(direction: PipeDirection, name: Option<String>, buffer: Option<Arc<Mutex<PipeBuffer>>>) -> IpcResult<usize> {
    unsafe {
        let next_pipe_id = &raw mut NEXT_PIPE_ID;
        let pipe_id = *next_pipe_id;
        if pipe_id >= 64 {
            return Err(super::IpcError::ResourceBusy);
        }
        *next_pipe_id += 1;

        let pipe = if let Some(buffer) = buffer {
            Pipe {
                buffer,
                direction,
                name,
            }
        } else {
            Pipe::new(direction, name)
        };

        let pipes = &raw mut PIPES;
        (*pipes).push(Some(pipe));
        Ok(pipe_id)
    }
}

// 写入管道
#[allow(dead_code)]
pub fn write_pipe(pipe_id: usize, data: &[u8]) -> IpcResult<usize> {
    unsafe {
        let pipes = &raw const PIPES;
        if pipe_id >= (*pipes).len() {
            return Err(super::IpcError::InvalidArgument);
        }
        if let Some(pipe) = &(&(*pipes))[pipe_id] {
            let written = pipe.write(data);
            if written == 0 && !pipe.buffer.lock().is_full() {
                Err(super::IpcError::InternalError)
            } else {
                Ok(written)
            }
        } else {
            Err(super::IpcError::InvalidArgument)
        }
    }
}

// 读取管道
#[allow(dead_code)]
pub fn read_pipe(pipe_id: usize, buf: &mut [u8]) -> IpcResult<usize> {
    unsafe {
        let pipes = &raw const PIPES;
        if pipe_id >= (*pipes).len() {
            return Err(super::IpcError::InvalidArgument);
        }
        if let Some(pipe) = &(&(*pipes))[pipe_id] {
            let read = pipe.read(buf);
            if read == 0 && !pipe.buffer.lock().is_empty() {
                Err(super::IpcError::InternalError)
            } else {
                Ok(read)
            }
        } else {
            Err(super::IpcError::InvalidArgument)
        }
    }
}

// 关闭管道
#[allow(dead_code)]
pub fn close_pipe(pipe_id: usize) -> IpcResult<()> {
    unsafe {
        let pipes = &raw mut PIPES;
        if pipe_id >= (*pipes).len() {
            return Err(super::IpcError::InvalidArgument);
        }
        if let Some(pipe) = &(&(*pipes))[pipe_id] {
            pipe.close();
            (&mut (*pipes))[pipe_id] = None;
            Ok(())
        } else {
            Err(super::IpcError::InvalidArgument)
        }
    }
}