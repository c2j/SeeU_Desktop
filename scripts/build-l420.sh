#!/bin/bash
# Build script for Linux targets using vcpkg with fallback support

cd ..
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
docker build -t seeu-desktop-linux-builder -f Dockerfile.l420 .
docker create --name seeu-temp-container seeu-desktop-linux-builder
docker cp seeu-temp-container:/output/seeu_desktop target/arm64v8/
docker rm seeu-temp-container