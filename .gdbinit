# GDB 配置文件
set architecture i386:x86-64
set gnutarget elf64-x86-64

# 连接到 QEMU
target remote localhost:1234

# 加载符号文件
file build/kernel.elf

# 设置断点
b _start
b kmain

# 继续执行
continue