package(default_visibility = ["//visibility:public"])
load("@toolchain//:cc_config.bzl", "cc_toolchain_config")

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
        "bin/clang-cpp",
    ],
)

filegroup(
    name = "clang_libs",
    srcs = glob(["lib/clang/13.0.0/lib/linux/*.a"]),
)

filegroup(
    name = "includes",
    srcs = glob([
        "include/c++/**",
        "lib/clang/13.0.0/include/**",
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

filegroup(
    name = "clang",
    srcs = [
        ":binaries",
        ":clang_libs"
    ],
)

filegroup(
    name = "clang_all",
    srcs = [
        ":clang",
        ":includes",
        ":runtime_libs",
        ":static_libs",
        "@org_llvm_libcxx//:raw_headers",
        "@org_llvm_libcxxabi//:raw_headers",
        "@toolchain//clang:clang_config",
    ],
)

cc_toolchain_config(
    name = "linux_x86_64_toolchain_config",
    target_cpu = "x86_64",
    builtin_include_directories = [
        "/usr/include/",
        "external/toolchain/clang/include",
        "external/org_llvm_clang/include/",
        "external/org_llvm_clang/lib/clang/13.0.0/include/",
    ],
)

toolchain(
    name = "linux_x86_64_toolchain",
    toolchain = ":linux_x86_64_cc_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
)

cc_toolchain(
    name = "linux_x86_64_cc_toolchain",
    toolchain_identifier = "linux_x86_64-cc-toolchain",
    toolchain_config = "linux_x86_64_toolchain_config",
    all_files = ":clang_all",
    compiler_files = ":clang_all",
    dwp_files = ":clang",
    linker_files = ":clang",
    ar_files = ":clang",
    as_files = ":clang",
    objcopy_files = ":clang",
    strip_files = ":clang",
    supports_param_files = True,
)

toolchain(
    name = "linux_arm64_toolchain",
    toolchain = ":linux_arm64_cc_toolchain",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:arm64",
    ],
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
)

cc_toolchain_config(
    name = "linux_arm64_toolchain_config",
    target_cpu = "arm64",
    target_system_name = "linux_arm64",
    builtin_include_directories = [
        "external/org_llvm_clang/include/",
        "external/org_llvm_clang/lib/clang/13.0.0/include/",
    ],
)

cc_toolchain(
    name = "linux_arm64_cc_toolchain",
    toolchain_identifier = "linux_arm64-cc-toolchain",
    toolchain_config = "linux_arm64_toolchain_config",
    all_files = ":clang_all",
    compiler_files = ":clang_all",
    dwp_files = ":clang",
    linker_files = ":clang",
    ar_files = ":clang",
    as_files = ":clang",
    objcopy_files = ":clang",
    strip_files = ":clang",
    supports_param_files = True,
)
