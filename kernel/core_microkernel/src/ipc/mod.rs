// IPC模块

pub mod pipe;
pub mod shared_memory;
pub mod semaphore;

// IPC错误类型
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum IpcError {
    InvalidArgument,
    ResourceBusy,
    NotFound,
    PermissionDenied,
    NotSupported,
    InternalError,
}

// IPC操作结果类型
type IpcResult<T> = Result<T, IpcError>;