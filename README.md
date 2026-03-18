# 墨渊操作系统 (Moyuan OS)

墨渊操作系统是一个基于 Rust 语言开发的现代操作系统，旨在提供安全、高效、可靠的计算环境。

## 项目结构

```
├── assets/           # 资源文件
├── bootloader/       # 引导加载器
├── build/            # 构建输出目录
├── docs/             # 文档
├── iso/              # ISO 镜像
├── kernel/           # 内核源码
│   ├── arch/         # 架构相关代码
│   ├── compatible_kernels/ # 兼容层
│   ├── core_microkernel/ # 核心微内核
│   ├── modules/      # 内核模块
│   └── security/     # 安全相关代码
├── resources/        # 资源
├── security_policy/  # 安全策略
├── services/         # 系统服务
│   ├── config_service/ # 配置服务
│   ├── container_service/ # 容器服务
│   ├── device_service/ # 设备服务
│   ├── fs_service/   # 文件系统服务
│   ├── gui_service/  # GUI 服务
│   ├── init/         # 初始化服务
│   ├── log_service/  # 日志服务
│   ├── network_service/ # 网络服务
│   ├── power_service/ # 电源服务
│   ├── security_service/ # 安全服务
│   ├── shell_service/ # Shell 服务
│   └── time_service/ # 时间服务
├── target/           # Rust 构建目标
├── third_party/      # 第三方库
├── toolchain/        # 工具链
├── userland/         # 用户空间
│   ├── apps/         # 应用程序
│   ├── bin/          # 二进制文件
│   ├── libs/         # 库文件
│   ├── loader/       # 加载器
│   └── tests/        # 测试
├── virtualization/   # 虚拟化
├── Cargo.toml        # Rust 项目配置
├── Makefile          # 构建脚本
└── README.md         # 项目说明
```

## 主要特性

- **基于 Rust 语言**：利用 Rust 的内存安全特性，提供更安全的系统环境
- **微内核设计**：核心功能最小化，其他功能通过服务实现
- **多架构支持**：支持 x86\_64、AArch64 和 RISC-V 架构
- **模块化设计**：系统组件高度模块化，便于扩展和维护
- **安全优先**：内置安全机制，保护系统和用户数据
- **现代化服务**：提供容器、网络、GUI 等现代操作系统服务

## 系统要求

- Rust 工具链（nightly 版本）
- NASM 汇编器
- QEMU 模拟器（用于测试）
- GCC 编译器（用于某些组件）

## 构建和运行

### 构建项目

```bash
# 构建整个项目
make all

# 仅构建内核
make kernel

# 仅构建 UEFI 引导加载器
make uefi_bootloader
```

### 运行项目

```bash
# 运行内核（在 QEMU 中）
make run

# 运行测试内核
make run_test

# 运行 UEFI 模拟器
make run_uefi
```

### 清理构建产物

```bash
make clean
```

## 核心组件

### 内核

- **核心微内核**：提供进程管理、内存管理、中断处理等基础功能
- **架构支持**：支持 x86\_64、AArch64 和 RISC-V 架构
- **兼容层**：提供 POSIX、Linux 和 RTOS 兼容层

### 系统服务

- **初始化服务**：系统启动和初始化
- **设备服务**：设备管理和驱动
- **文件系统服务**：文件系统管理
- **网络服务**：网络协议栈和管理
- **安全服务**：安全策略和访问控制
- **Shell 服务**：命令行界面
- **GUI 服务**：图形用户界面
- **容器服务**：容器管理
- **时间服务**：时间管理
- **电源服务**：电源管理
- **日志服务**：系统日志
- **配置服务**：系统配置

### 用户空间

- **标准库**：提供用户程序所需的库函数
- **基本工具**：包括 ls、cat、mkdir、rm 等基本命令
- **应用程序**：用户应用程序

## 开发指南

### 代码风格

- 遵循 Rust 官方代码风格
- 使用 4 空格缩进
- 文件名使用 snake\_case
- 模块名使用 snake\_case
- 结构体和枚举使用 PascalCase
- 函数和变量使用 snake\_case

### 贡献流程

1. Fork 项目仓库
2. 创建功能分支
3. 提交代码
4. 推送分支
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 联系方式

- 项目主页：[https://github.com/sheng-fs/moyuan-os](https://github.com/sheng-fs/moyuan-os)
- 邮件：<3555679134@qq.com>

## 致谢

感谢所有为墨渊操作系统做出贡献的开发者和支持者！
