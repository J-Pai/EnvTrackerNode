# Bazel Toolchain

Personal Bazel based toolchain for cross-compiling.

Toolchain utilizes Clang/LLVM [13.0.0](https://github.com/llvm/llvm-project/releases/tag/llvmorg-13.0.0) as the primary compiler.

Below are the supported targets (other than linux x86_64).

## Ubuntu 20.04 AArch64

Meant to target Ubuntu 20.04 running on a Raspberry Pi 4.

Based on https://github.com/mjbots/rpi_bazel, but modified to use Bazel
platforms instead of `--crosstool_top`.
