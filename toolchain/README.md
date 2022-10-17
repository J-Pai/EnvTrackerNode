# Bazel Toolchain

Personal Bazel based toolchain for cross-compiling.

Toolchain utilizes Clang/LLVM [13.0.0](https://github.com/llvm/llvm-project/releases/tag/llvmorg-13.0.0) as the primary compiler.

Below are the supported targets (other than linux x86_64).

## Ubuntu 20.04 x86_64

Example invocation:

```
bazel run @toolchain//bazel_demo:hello_world
```

## Ubuntu 20.04 AArch64

Meant to target Ubuntu 20.04 running on a Raspberry Pi 4.

Based on https://github.com/mjbots/rpi_bazel, but modified to use
[Bazel Platforms](https://bazel.build/concepts/platforms) instead of
`--crosstool_top`.

Install Aarch64 sysroot and libraries with the following command prior to
starting the build.

```bash
sudo apt install gcc-aarch64-linux-gnu
sudo apt install g++-aarch64-linux-gnu
```

Example invocation:

```
bazel build --config=arm64 @toolchain//bazel_demo:hello_world
```


## Sources/Inspiration
- https://github.com/mjbots/rpi_bazel
- https://ltekieli.com/cross-compiling-with-bazel/
