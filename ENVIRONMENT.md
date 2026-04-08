# 墨渊操作系统开发环境配置指南

## 1. 环境概述

墨渊操作系统（Moyuan OS）是一个基于 Rust 语言开发的现代操作系统。本指南将帮助您设置完整的开发环境。

## 2. 系统要求

### 2.1 硬件要求
- 处理器：x86_64 架构（推荐 2 核以上）
- 内存：至少 4GB RAM（推荐 8GB 以上）
- 存储：至少 10GB 可用空间

### 2.2 软件要求
- 操作系统：Linux（推荐 Ubuntu 20.04 或更高版本）
- Rust 工具链：nightly 版本
- NASM 汇编器
- QEMU 模拟器
- Make 构建工具
- Git 版本控制

## 3. 环境安装步骤

### 3.1 安装系统依赖

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y build-essential git nasm qemu-system-x86 qemu-utils ovmf

# Arch Linux
sudo pacman -S base-devel git nasm qemu ovmf
```

### 3.2 安装 Rust 工具链

项目使用 `rust-toolchain.toml` 自动管理 Rust 工具链版本。只需确保已安装 Rustup：

```bash
# 安装 Rustup（如未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

项目会自动安装以下组件：
- rust-src
- llvm-tools-preview
- x86_64-unknown-none 目标
- aarch64-unknown-none 目标
- riscv64gc-unknown-none-elf 目标

### 3.3 验证安装

```bash
# 检查 Rust 版本
rustc --version
cargo --version

# 检查其他工具
nasm -v
qemu-system-x86_64 --version
git --version
```

## 4. 项目配置

### 4.1 环境变量（可选）

可以设置以下环境变量来优化开发体验：

```bash
# 添加到 ~/.bashrc 或 ~/.zshrc
export RUST_LOG=debug
export CARGO_TARGET_DIR=./target
```

### 4.2 项目配置文件

项目已包含以下配置文件：
- `rust-toolchain.toml` - Rust 工具链配置
- `.cargo/config.toml` - Cargo 构建配置
- `.editorconfig` - 代码格式规范
- `.vscode/settings.json` - VS Code 编辑器配置
- `CODE_STYLE.md` - 项目代码风格规范

## 5. 构建和运行

### 5.1 构建项目

```bash
# 构建整个项目（内核 + UEFI 引导加载器）
make all

# 仅构建内核
make kernel

# 仅构建 UEFI 引导加载器
make uefi_bootloader

# 构建调试版本
make debug_build
```

### 5.2 运行项目

```bash
# 运行内核（在 QEMU 中）
make run

# 运行测试内核
make run_test

# 运行 UEFI 模拟器
make run_uefi
```

### 5.3 调试项目

```bash
# 启动调试环境（GDB + QEMU）
make debug

# 启动 UEFI 调试环境
make debug_uefi

# 启动 GRUB 调试环境
make debug_grub
```

### 5.4 清理构建产物

```bash
make clean
```

## 6. 开发工具

### 6.1 代码格式化

```bash
# 格式化 Rust 代码
cargo fmt

# 检查代码格式（不修改）
cargo fmt -- --check
```

### 6.2 代码检查

```bash
# 使用 Clippy 进行代码检查
cargo clippy

# 自动修复某些 Clippy 警告
cargo clippy --fix
```

### 6.3 代码分析

```bash
# 使用 Rust Analyzer（VS Code 推荐）
# 安装 VS Code 扩展：rust-lang.rust-analyzer
```

## 7. Git 配置

### 7.1 基本配置

```bash
# 设置用户名和邮箱
git config user.name "Your Name"
git config user.email "your.email@example.com"

# （可选）设置默认分支名
git config init.defaultBranch main
```

### 7.2 提交规范

项目使用以下提交消息格式：

```
<类型>(<范围>): <主题>

<详细描述>

<页脚>
```

类型包括：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

## 8. 开发流程

### 8.1 创建功能分支

```bash
# 从 main 分支创建新分支
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

### 8.2 开发和提交

```bash
# 编写代码
# 运行测试
make test_kernel
make run_test

# 格式化和检查代码
cargo fmt
cargo clippy

# 提交更改
git add .
git commit -m "feat: 描述你的更改"
```

### 8.3 推送和 PR

```bash
# 推送到远程仓库
git push origin feature/your-feature-name

# 在 GitHub/GitLab 上创建 Pull Request
```

## 9. 常见问题

### 9.1 Rust 工具链问题

```bash
# 更新 Rust 工具链
rustup update

# 重新安装工具链
rustup toolchain uninstall nightly
rustup toolchain install nightly
```

### 9.2 构建错误

```bash
# 清理并重新构建
make clean
cargo clean
make all
```

### 9.3 QEMU 相关问题

确保已安装正确的 QEMU 版本和 OVMF BIOS：
```bash
# Ubuntu/Debian
sudo apt-get install ovmf

# 检查 OVMF 位置
ls -la /usr/share/qemu/OVMF.fd
```

## 10. 环境一致性

为了确保开发环境与生产环境一致，请遵循以下原则：

1. **使用相同的 Rust 工具链版本**：项目通过 `rust-toolchain.toml` 统一管理
2. **使用相同的依赖版本**：确保 `Cargo.lock` 被提交到版本控制
3. **使用相同的构建工具**：通过 `Makefile` 统一构建流程
4. **文档化所有配置**：所有环境配置都应有相应的文档说明

## 11. 联系方式

如有问题，请参考：
- 项目主页：https://github.com/sheng-fs/moyuan-os
- 问题反馈：提交 GitHub Issue

---

**最后更新**：2026-04-08
