package(default_visibility = ["//visibility:public"])

load(":cc_config.bzl", "cc_toolchain_config")

platform(
    name = "linux_x86_64",
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
)

platform(
    name = "linux_aarch64",
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:arm64",
    ],
)

filegroup(name = "empty")

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

cc_toolchain_config(
    name = "linux_x86_64_toolchain_config",
)

cc_toolchain(
    name = "linux_x86_64_cc_toolchain",
    toolchain_identifier = "linux_x86_64-cc-toolchain",
    toolchain_config = "linux_x86_64_toolchain_config",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    ar_files = ":empty",
    as_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = True,
)

toolchain(
    name = "linux_aarch64_toolchain",
    toolchain = ":linux_aarch64_cc_toolchain",
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
    name = "linux_aarch64_toolchain_config",
)

cc_toolchain(
    name = "linux_aarch64_cc_toolchain",
    toolchain_identifier = "linux_aarch64-cc-toolchain",
    toolchain_config = "linux_aarch64_toolchain_config",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    ar_files = ":empty",
    as_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = True,
)
