package(default_visibility = ["//visibility:public"])

filegroup(
    name = "binaries",
    srcs = [
        "bin/clang",
        "bin/clang++",
        "bin/llvm-dwp",
        "bin/llvm-objcopy",
        "bin/llvm-objdump",
        "bin/llvm-strip",
        "bin/ld.lld",
        "bin/llvm-ar",
        "bin/llvm-nm",
        "bin/llvm-cov",
    ],
)

filegroup(
    name = "clang_libs",
    srcs = glob(["lib/clang/10.0.0/lib/linux/*.a"]),
)

filegroup(
    name = "includes",
    srcs = glob([
        "include/c++/**",
        "lib/clang/10.0.0/include/**",
    ]),
)

filegroup(
    name = "runtime_libs",
    srcs = [
        "lib/libc++.so.1",
        "lib/libc++abi.so.1",
        "lib/libunwind.so.1",
    ],
)

filegroup(
    name = "static_libs",
    srcs = [
        "lib/libc++.a",
        "lib/libc++abi.a",
        "lib/libunwind.a",
    ],
)
