use crate::mm::virtual::AddressSpace;

// 进程状态
enum ProcessState {
    Running,
    Ready,
    Blocked,
    Terminated,
}

// 进程控制块
pub struct ProcessControlBlock {
    // 进程ID
    pid: usize,
    // 进程状态
    state: ProcessState,
    // 进程名称
    name: &'static str,
    // 虚拟地址空间
    address_space: AddressSpace,
    // 进程优先级
    priority: u32,
    // 栈指针
    stack_pointer: u64,
    // 指令指针
    instruction_pointer: u64,
    // 通用寄存器
    registers: [u64; 16], // RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP, R8-R15
    // 标志寄存器
    rflags: u64,
}

impl ProcessControlBlock {
    // 创建新进程
    pub fn new(pid: usize, name: &'static str, priority: u32) -> Self {
        ProcessControlBlock {
            pid,
            state: ProcessState::Ready,
            name,
            address_space: AddressSpace::new(),
            priority,
            stack_pointer: 0,
            instruction_pointer: 0,
            registers: [0; 16],
            rflags: 0,
        }
    }
    
    // 获取进程ID
    pub fn pid(&self) -> usize {
        self.pid
    }
    
    // 获取进程状态
    pub fn state(&self) -> &ProcessState {
        &self.state
    }
    
    // 设置进程状态
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
    
    // 获取进程名称
    pub fn name(&self) -> &'static str {
        self.name
    }
    
    // 获取地址空间
    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
    }
    
    // 获取地址空间可变引用
    pub fn address_space_mut(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }
    
    // 获取优先级
    pub fn priority(&self) -> u32 {
        self.priority
    }
    
    // 设置栈指针
    pub fn set_stack_pointer(&mut self, sp: u64) {
        self.stack_pointer = sp;
    }
    
    // 获取栈指针
    pub fn stack_pointer(&self) -> u64 {
        self.stack_pointer
    }
    
    // 设置指令指针
    pub fn set_instruction_pointer(&mut self, ip: u64) {
        self.instruction_pointer = ip;
    }
    
    // 获取指令指针
    pub fn instruction_pointer(&self) -> u64 {
        self.instruction_pointer
    }
    
    // 设置寄存器
    pub fn set_register(&mut self, index: usize, value: u64) {
        if index < 16 {
            self.registers[index] = value;
        }
    }
    
    // 获取寄存器
    pub fn register(&self, index: usize) -> u64 {
        if index < 16 {
            self.registers[index]
        } else {
            0
        }
    }
    
    // 设置标志寄存器
    pub fn set_rflags(&mut self, flags: u64) {
        self.rflags = flags;
    }
    
    // 获取标志寄存器
    pub fn rflags(&self) -> u64 {
        self.rflags
    }
}
