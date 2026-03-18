fn main() {
    // 编译 pvh.S 文件
    let mut build = cc::Build::new();
    build.file("src/pvh.S");
    build.compile("pvh");
}
