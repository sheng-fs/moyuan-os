# 设置环境变量
$env:RUSTFLAGS = "-C link-arg=-Tkernel/core_microkernel/linker.ld"

# 构建内核
cargo build --target x86_64-unknown-none --release
