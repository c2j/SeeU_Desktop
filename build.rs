use std::env;

fn main() {
    // Only run vcpkg integration if we're not in a docs.rs build
    if env::var("DOCS_RS").is_err() {
        // Configure vcpkg for cross-platform builds
        let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

        println!("cargo:rerun-if-changed=vcpkg.json");
        println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
        println!("cargo:rerun-if-env-changed=VCPKG_DEFAULT_TRIPLET");

        // 强制禁用 pkg-config 相关的环境变量检查
        configure_no_pkg_config();
        
        // Set up vcpkg configuration based on target
        match target.as_str() {
            "x86_64-pc-windows-msvc" => {
                env::set_var("VCPKG_DEFAULT_TRIPLET", "x64-windows-static");
                configure_vcpkg_windows();
            }
            "i686-pc-windows-msvc" => {
                env::set_var("VCPKG_DEFAULT_TRIPLET", "x86-windows-static");
                configure_vcpkg_windows();
            }
            "x86_64-unknown-linux-gnu" => {
                env::set_var("VCPKG_DEFAULT_TRIPLET", "x64-linux");
                configure_vcpkg_linux();
            }
            "aarch64-unknown-linux-gnu" => {
                env::set_var("VCPKG_DEFAULT_TRIPLET", "arm64-linux");
                configure_vcpkg_linux();
            }
            "x86_64-unknown-linux-musl" => {
                env::set_var("VCPKG_DEFAULT_TRIPLET", "x64-linux");
                configure_vcpkg_linux();
            }
            "aarch64-unknown-linux-musl" => {
                env::set_var("VCPKG_DEFAULT_TRIPLET", "arm64-linux");
                configure_vcpkg_linux();
            }
            _ => {
                println!("cargo:warning=Unsupported target for vcpkg: {}", target);
            }
        }
        
        // Try to find and configure vcpkg
        if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
            println!("cargo:rustc-env=VCPKG_ROOT={}", vcpkg_root);
        } else {
            // Try common vcpkg installation paths
            let possible_paths = [
                "/usr/local/share/vcpkg",
                "/opt/vcpkg",
                "C:\\vcpkg",
                "C:\\tools\\vcpkg",
                "C:\\dev\\vcpkg",
            ];
            
            for path in &possible_paths {
                if std::path::Path::new(path).exists() {
                    println!("cargo:rustc-env=VCPKG_ROOT={}", path);
                    env::set_var("VCPKG_ROOT", path);
                    break;
                }
            }
        }
        
        // Configure vcpkg for static linking
        if env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default().contains("crt-static") {
            println!("cargo:rustc-link-arg=-static");
        }
    }
}

fn configure_vcpkg_windows() {
    // Windows-specific vcpkg configuration
    println!("cargo:rustc-link-lib=static=user32");
    println!("cargo:rustc-link-lib=static=shell32");
    println!("cargo:rustc-link-lib=static=ole32");
    println!("cargo:rustc-link-lib=static=oleaut32");
    println!("cargo:rustc-link-lib=static=uuid");
    println!("cargo:rustc-link-lib=static=comctl32");
    println!("cargo:rustc-link-lib=static=comdlg32");
    println!("cargo:rustc-link-lib=static=gdi32");
    println!("cargo:rustc-link-lib=static=winspool");
    println!("cargo:rustc-link-lib=static=winmm");
    println!("cargo:rustc-link-lib=static=advapi32");
    
    // Enable static CRT linking for Windows
    if env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc" {
        println!("cargo:rustc-link-arg=/DEFAULTLIB:libcmt");
    }
}

fn configure_vcpkg_linux() {
    // 禁用 pkg-config，强制使用 vcpkg 或 vendored 依赖


    println!("cargo:warning=Configured for Linux build without pkg-config dependencies");
}

fn configure_no_pkg_config() {
    // 禁用所有可能的 pkg-config 查找
    env::set_var("PKG_CONFIG_ALLOW_CROSS", "0");
    env::set_var("PKG_CONFIG", "false");

    // 禁用 system-deps 的 pkg-config 查找
    env::set_var("SYSTEM_DEPS_NO_PKG_CONFIG", "1");

    // 禁用特定库的 pkg-config 查找
    let no_pkg_config_libs = [
        "GLIB_2_0", "GOBJECT_2_0", "GIO_2_0", "CAIRO", "PANGO", "PANGOCAIRO",
        "GDK_PIXBUF_2_0", "GTK_3_0", "ATK", "FONTCONFIG", "FREETYPE2", "HARFBUZZ",
        "EXPAT", "BZIP2", "ZLIB", "LIBPNG", "JPEG"
    ];

    for lib in &no_pkg_config_libs {
        env::set_var(&format!("{}_NO_PKG_CONFIG", lib), "1");
    }

    // 强制使用 vendored 依赖
    env::set_var("OPENSSL_STATIC", "1");
    env::set_var("OPENSSL_VENDORED", "1");
    env::set_var("LIBSQLITE3_SYS_USE_PKG_CONFIG", "0");
    env::set_var("LIBZ_SYS_STATIC", "1");

    println!("cargo:warning=Disabled pkg-config for all system dependencies");
}
