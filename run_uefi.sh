#!/bin/bash

# 墨渊操作系统UEFI启动脚本

echo "=== 墨渊操作系统 UEFI 启动脚本 ==="

# 1. 清理旧文件
echo "清理旧文件..."
rm -f /tmp/moyuan_os.img
rm -rf /tmp/efi

# 2. 创建100MB空白镜像
echo "创建空白镜像..."
dd if=/dev/zero of=/tmp/moyuan_os.img bs=1M count=100

# 3. 格式化为FAT32
echo "格式化为FAT32..."
mkfs.fat -F32 /tmp/moyuan_os.img

# 4. 创建标准UEFI目录结构
echo "创建UEFI目录结构..."
mkdir -p /tmp/efi/EFI/BOOT

# 5. 复制内核文件并命名为标准名称
echo "复制内核文件..."
cp /home/jinju/MOYUAN_OS/build/kernel.elf /tmp/efi/EFI/BOOT/BOOTX64.EFI

# 6. 复制到镜像中
echo "复制到镜像中..."
mcopy -i /tmp/moyuan_os.img -s /tmp/efi/EFI ::/

# 7. 验证文件路径
echo "验证文件路径..."
mdir -i /tmp/moyuan_os.img ::/EFI/BOOT/

# 8. 启动QEMU
echo "启动QEMU..."
qemu-system-x86_64 \
  -drive format=raw,file=/tmp/moyuan_os.img \
  -bios /usr/share/ovmf/OVMF.fd \
  -net none \
  -accel tcg \
  -vga std \
  -serial stdio
