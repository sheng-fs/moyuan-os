#!/bin/bash

# 修复Cargo.toml文件并创建src/lib.rs文件

# 兼容层目录
compat_dirs=("kernel/compatible_kernels/posix_compat"
             "kernel/compatible_kernels/linux_compat"
             "kernel/compatible_kernels/rtos_compat")

# 服务目录
service_dirs=("services/init"
              "services/device_service"
              "services/fs_service"
              "services/network_service"
              "services/security_service"
              "services/time_service"
              "services/power_service"
              "services/log_service"
              "services/config_service"
              "services/gui_service"
              "services/shell_service"
              "services/container_service")

# 修复兼容层
for dir in "${compat_dirs[@]}"; do
    echo "处理 $dir..."
    
    # 修复Cargo.toml
    if [ -f "$dir/Cargo.toml" ]; then
        if ! grep -q "\[lib\]" "$dir/Cargo.toml"; then
            sed -i '/\[dependencies\]/i \[lib\]\npath = "src/lib.rs"\n' "$dir/Cargo.toml"
        fi
    fi
    
    # 创建src/lib.rs
    mkdir -p "$dir/src"
    echo '#![no_std]

// 兼容层' > "$dir/src/lib.rs"
done

# 修复服务
for dir in "${service_dirs[@]}"; do
    echo "处理 $dir..."
    
    # 修复Cargo.toml
    if [ -f "$dir/Cargo.toml" ]; then
        if ! grep -q "\[lib\]" "$dir/Cargo.toml"; then
            sed -i '/\[dependencies\]/i \[lib\]\npath = "src/lib.rs"\n' "$dir/Cargo.toml"
        fi
    fi
    
    # 创建src/lib.rs
    mkdir -p "$dir/src"
    echo '#![no_std]

// 服务' > "$dir/src/lib.rs"
done

echo "所有文件已修复完成！"
