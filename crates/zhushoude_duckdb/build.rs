// 暂时简化build.rs，避免依赖冲突
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:warning=模型下载功能暂时禁用，将在后续阶段实现");
}
