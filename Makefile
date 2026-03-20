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

# 内核构建目标
KERNEL = $(OUTPUT_DIR)/kernel.elf

# 测试内核构建目标
TEST_KERNEL = $(TEST_OUTPUT_DIR)/test_kernel.elf

# 引导加载器构建目标
UEFI_BOOTLOADER = $(OUTPUT_DIR)/bootloader.efi

# 构建目标
all: kernel uefi_bootloader

# 构建内核
kernel:
	@mkdir -p $(OUTPUT_DIR)
	@echo "构建内核..."
	@RUSTFLAGS="-C link-arg=-Tkernel/core_microkernel/linker.ld -C relocation-model=static" $(CARGO) build --target $(TARGET) --release
	@cp target/$(TARGET)/release/kernel $(KERNEL)

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

# 清理构建产物
clean:
	@echo "清理构建产物..."
	@$(CARGO) clean
	@cd bootloader/uefi && $(CARGO) clean
	@rm -rf $(OUTPUT_DIR)

# 运行QEMU模拟器
run:
	@echo "运行模拟器..."
	@qemu-system-x86_64 -kernel $(KERNEL) -m 512M

# 运行测试
run_test:
	@echo "运行测试..."
	@qemu-system-x86_64 -kernel $(TEST_KERNEL) -m 512M -serial stdio -cpu qemu64 -machine pc -accel tcg

# 运行UEFI模拟器
run_uefi:
	@echo "运行UEFI模拟器..."
	@qemu-system-x86_64 -bios /usr/share/qemu/OVMF.fd -hda fat:rw:$(OUTPUT_DIR) -m 2G

# 调试构建（包含调试信息）
debug_build:
	@mkdir -p $(OUTPUT_DIR)
	@echo "构建调试内核..."
	@RUSTFLAGS="-C link-arg=-Tkernel/core_microkernel/linker.ld -C relocation-model=pic" $(CARGO) build --target $(TARGET)
	@cp target/$(TARGET)/debug/kernel $(KERNEL)

# 运行UEFI调试环境（GDB + QEMU）
debug_uefi:
	@echo "运行UEFI调试环境..."
	@qemu-system-x86_64 \
		-drive format=raw,file=/tmp/moyuan_os.img \
		-bios /usr/share/qemu/OVMF.fd \
		-m 2G \
		-net none \
		-accel tcg \
		-vga std \
		-serial stdio \
		-s \
		-S

# 运行GRUB调试环境（GDB + QEMU）
debug_grub:
	@echo "运行GRUB调试环境..."
	@qemu-system-x86_64 \
		-cdrom /tmp/moyuan_os.iso \
		-m 2G \
		-net none \
		-accel tcg \
		-vga std \
		-serial mon:stdio \
		-s \
		-S

# 运行调试环境（GDB + QEMU）
debug:
	@echo "运行调试环境..."
	@qemu-system-x86_64 \
		-kernel $(KERNEL) \
		-m 2G \
		-smp 4 \
		-netdev user,id=net0,net=192.168.1.0/24,dhcpstart=192.168.1.100 \
		-device e1000,netdev=net0 \
		-enable-kvm \
		-device intel-iommu \
		-s -S

# 运行调试测试
debug_test:
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

.PHONY: all kernel test_kernel uefi_bootloader clean run run_test run_uefi debug_build debug_uefi debug_grub debug debug_test
