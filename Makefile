# 墨渊操作系统构建脚本

# 工具链设置
RUSTC = rustc
CARGO = cargo
NASM = nasm

# 构建目标
TARGET = x86_64-unknown-none

# 输出目录
OUTPUT_DIR = build
TEST_OUTPUT_DIR = build/test
ISO_DIR = iso

# 内核构建目标
KERNEL = $(OUTPUT_DIR)/kernel.elf
KERNEL_ISO = $(ISO_DIR)/boot/kernel.elf

# 测试内核构建目标
TEST_KERNEL = $(TEST_OUTPUT_DIR)/test_kernel.elf

# 引导加载器构建目标
UEFI_BOOTLOADER = $(OUTPUT_DIR)/bootloader.efi

# 构建目标
all: kernel uefi_bootloader iso

# 构建内核
kernel:
	@mkdir -p $(OUTPUT_DIR) $(ISO_DIR)/boot
	@echo "构建内核..."
	@RUSTFLAGS="-C link-arg=-Tkernel/core_microkernel/linker.ld -C relocation-model=static" $(CARGO) build --target $(TARGET) --release
	@cp target/$(TARGET)/release/kernel $(KERNEL)
	@cp target/$(TARGET)/release/kernel $(KERNEL_ISO)

# 构建测试内核
test_kernel:
	@mkdir -p $(TEST_OUTPUT_DIR)
	@echo "构建测试内核..."
	@RUSTFLAGS="-C link-arg=-Tkernel/core_microkernel/linker.ld -C relocation-model=static" $(CARGO) build --target $(TARGET) --features test --release
	@cp target/$(TARGET)/release/kernel $(TEST_KERNEL)

# 构建UEFI引导加载器
uefi_bootloader:
	@mkdir -p $(OUTPUT_DIR)
	@echo "构建UEFI引导加载器..."
	@cd bootloader/uefi && $(CARGO) build --target x86_64-unknown-uefi --release
	@cp target/x86_64-unknown-uefi/release/moyuan-uefi-bootloader.efi $(UEFI_BOOTLOADER)
	@mkdir -p $(OUTPUT_DIR)/EFI/BOOT
	@cp target/x86_64-unknown-uefi/release/moyuan-uefi-bootloader.efi $(OUTPUT_DIR)/EFI/BOOT/BOOTX64.EFI

# 构建ISO镜像
iso: kernel
	@echo "构建ISO镜像..."
	@mkdir -p $(ISO_DIR)/boot/grub
	@cp bootloader/grub/grub.cfg $(ISO_DIR)/boot/grub/grub.cfg
	@echo "ISO镜像准备完成"

# 清理构建产物
clean:
	@echo "清理构建产物..."
	@$(CARGO) clean
	@cd bootloader/uefi && $(CARGO) clean
	@rm -rf $(OUTPUT_DIR)
	@rm -f $(ISO_DIR)/boot/kernel.elf

# 运行QEMU模拟器（直接内核启动）
run: kernel
	@echo "运行模拟器（直接内核启动）..."
	@qemu-system-x86_64 -kernel $(KERNEL) -m 512M -serial stdio

# 运行测试
run_test: test_kernel
	@echo "运行测试..."
	@qemu-system-x86_64 -kernel $(TEST_KERNEL) -m 512M -serial stdio -cpu qemu64 -machine pc -accel tcg

# 运行UEFI模拟器
run_uefi: uefi_bootloader kernel
	@echo "运行UEFI模拟器..."
	@mkdir -p $(OUTPUT_DIR)
	@cp $(KERNEL) $(OUTPUT_DIR)/kernel.elf
	@qemu-system-x86_64 -bios /usr/share/qemu/OVMF.fd -hda fat:rw:$(OUTPUT_DIR) -m 2G -serial stdio -vga std

# 运行GRUB模拟器
run_grub: iso
	@echo "运行GRUB模拟器..."
	@grub-mkrescue -o $(OUTPUT_DIR)/moyuan_os.iso $(ISO_DIR) 2>/dev/null || \
		(echo "警告: grub-mkrescue未找到，无法创建ISO" && exit 1)
	@qemu-system-x86_64 -cdrom $(OUTPUT_DIR)/moyuan_os.iso -m 512M -serial stdio

# 调试构建（包含调试信息）
debug_build:
	@mkdir -p $(OUTPUT_DIR)
	@echo "构建调试内核..."
	@RUSTFLAGS="-C link-arg=-Tkernel/core_microkernel/linker.ld -C relocation-model=pic" $(CARGO) build --target $(TARGET)
	@cp target/$(TARGET)/debug/kernel $(KERNEL)

# 运行UEFI调试环境（GDB + QEMU）
debug_uefi: uefi_bootloader kernel
	@echo "运行UEFI调试环境..."
	@mkdir -p $(OUTPUT_DIR)
	@cp $(KERNEL) $(OUTPUT_DIR)/kernel.elf
	@qemu-system-x86_64 \
		-bios /usr/share/qemu/OVMF.fd \
		-hda fat:rw:$(OUTPUT_DIR) \
		-m 2G \
		-net none \
		-accel tcg \
		-vga std \
		-serial stdio \
		-s \
		-S

# 运行GRUB调试环境（GDB + QEMU）
debug_grub: iso
	@echo "运行GRUB调试环境..."
	@grub-mkrescue -o $(OUTPUT_DIR)/moyuan_os.iso $(ISO_DIR) 2>/dev/null || \
		(echo "警告: grub-mkrescue未找到，无法创建ISO" && exit 1)
	@qemu-system-x86_64 \
		-cdrom $(OUTPUT_DIR)/moyuan_os.iso \
		-m 2G \
		-net none \
		-accel tcg \
		-vga std \
		-serial mon:stdio \
		-s \
		-S

# 运行调试环境（GDB + QEMU，直接内核启动）
debug: kernel
	@echo "运行调试环境（直接内核启动）..."
	@qemu-system-x86_64 \
		-kernel $(KERNEL) \
		-m 2G \
		-smp 4 \
		-netdev user,id=net0,net=192.168.1.0/24,dhcpstart=192.168.1.100 \
		-device e1000,netdev=net0 \
		-enable-kvm \
		-device intel-iommu \
		-serial stdio \
		-s -S

# 运行调试测试
debug_test: test_kernel
	@echo "运行调试测试..."
	@qemu-system-x86_64 \
		-kernel $(TEST_KERNEL) \
		-m 2G \
		-smp 4 \
		-netdev user,id=net0,net=192.168.1.0/24,dhcpstart=192.168.1.100 \
		-device e1000,netdev=net0 \
		-enable-kvm \
		-device intel-iommu \
		-serial stdio \
		-s -S

.PHONY: all kernel test_kernel uefi_bootloader iso clean run run_test run_uefi run_grub debug_build debug_uefi debug_grub debug debug_test
