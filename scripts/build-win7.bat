#!/bin/bash
# Build script for Linux targets using vcpkg with fallback support

cd ..
cargo build --release --target x86_64-pc-windows-gnu