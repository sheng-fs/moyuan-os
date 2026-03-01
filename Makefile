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

# 引导汇编文件
BOOT_ASM = kernel/arch/x86_64/boot/boot.asm
BOOT_OBJ = target/$(TARGET)/debug/libboot.a

# 构建目标
all: boot kernel uefi_bootloader

# 编译引导汇编
boot:
	@echo "编译引导汇编..."
	@mkdir -p target/$(TARGET)/debug
	@$(NASM) -f elf64 $(BOOT_ASM) -o target/$(TARGET)/debug/boot.o
	@ar rcs $(BOOT_OBJ) target/$(TARGET)/debug/boot.o

# 构建内核
kernel:
	@mkdir -p $(OUTPUT_DIR)
	@echo "构建内核..."
	@$(CARGO) build --target $(TARGET) --release
	@cp target/$(TARGET)/release/kernel $(KERNEL)

# 构建测试内核
test_kernel:
	@mkdir -p $(TEST_OUTPUT_DIR)
	@echo "构建测试内核..."
	@$(CARGO) build --target $(TARGET) --features test --release
	@cp target/$(TARGET)/release/kernel $(TEST_KERNEL)

# 构建UEFI引导加载器
uefi_bootloader:
	@mkdir -p $(OUTPUT_DIR)
	@echo "构建UEFI引导加载器..."
	@cd bootloader/uefi && $(CARGO) build --target x86_64-unknown-uefi --release
	@cp bootloader/uefi/target/x86_64-unknown-uefi/release/bootloader.efi $(UEFI_BOOTLOADER)

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
	@qemu-system-x86_64 -kernel $(TEST_KERNEL) -m 512M -serial stdio

# 运行UEFI模拟器
run_uefi:
	@echo "运行UEFI模拟器..."
	@qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -hda fat:rw:$(OUTPUT_DIR) -m 512M

.PHONY: all boot kernel test_kernel uefi_bootloader clean run run_test run_uefi
