// zhushoude_duckdb build script
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // 检查是否启用模型下载功能
    if cfg!(feature = "model-download") {
        println!("cargo:rustc-cfg=model_download_enabled");
    }

    // 设置版本信息
    println!("cargo:rustc-env=ZHUSHOUDE_VERSION={}", env!("CARGO_PKG_VERSION"));

    // 检查目标平台
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm") {
        println!("cargo:rustc-cfg=target_wasm");
    }
}
