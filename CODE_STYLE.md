# 墨渊操作系统 Rust 代码规范

## 1. 命名规范

### 1.1 系统调用
- **前缀**：`sys_`
- **示例**：`sys_open`, `sys_read`, `sys_write`
- **使用场景**：内核与用户空间之间的系统调用接口

### 1.2 飞地模块
- **前缀**：`enclave_`
- **示例**：`enclave_init`, `enclave_secure_memory`, `enclave_exit`
- **使用场景**：外核飞地相关的模块和函数

### 1.3 多语言接口
- **后缀**：`_l10n`
- **示例**：`get_message_l10n`, `format_error_l10n`
- **使用场景**：支持多语言的接口函数

## 2. 通用命名规范

### 2.1 变量和函数
- 使用蛇形命名法：`snake_case`
- 常量使用全大写：`UPPER_SNAKE_CASE`
- 类型和特质使用驼峰命名法：`CamelCase`

### 2.2 模块和文件
- 模块名使用蛇形命名法：`snake_case`
- 文件和目录名使用蛇形命名法：`snake_case`

## 3. 代码风格

### 3.1 缩进
- 使用 4 个空格进行缩进
- 不使用制表符

### 3.2 行宽
- 每行代码不超过 100 个字符
- 长行应适当换行

### 3.3 注释
- 模块级注释使用 `//!`
- 函数和结构体注释使用 `///`
- 行内注释使用 `//`

## 4. 规范执行

- 所有新增代码必须遵循本规范
- 现有代码逐步迁移到本规范
- 代码审查时应检查命名规范的遵守情况

## 5. 示例

```rust
// 系统调用示例
pub fn sys_open(path: *const u8, flags: usize) -> isize {
    // 实现
}

// 飞地模块示例
pub fn enclave_init(secure_memory: usize) -> Result<(), EnclaveError> {
    // 实现
}

// 多语言接口示例
pub fn get_greeting_l10n(lang: &str) -> &'static str {
    // 实现
}
```