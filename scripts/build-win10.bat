#!/bin/bash
# Build script for Linux targets using vcpkg with fallback support

cd ../target
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc