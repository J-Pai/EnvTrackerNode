load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def build_tools():
    http_archive(
        name = "org_llvm_clang",
        urls = [
            "https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04.tar.xz",
        ],
        strip_prefix = "clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04",
        build_file = Label("@toolchain//clang:package.BUILD"),
    )
    http_archive(
        name = "org_llvm_libcxx",
        urls = [
            "https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/libcxx-13.0.0.src.tar.xz",
        ],
        strip_prefix = "libcxx-13.0.0.src",
        build_file = Label("@toolchain//clang:libcxx.BUILD"),
    )
    http_archive(
        name = "org_llvm_libcxxabi",
        urls = [
            "https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/libcxxabi-13.0.0.src.tar.xz",
        ],
        strip_prefix = "libcxxabi-13.0.0.src",
        build_file = Label("@toolchain//clang:libcxxabi.BUILD"),
    )

    native.register_toolchains(
        "@org_llvm_clang//:linux_x86_64_toolchain",
        "@org_llvm_clang//:linux_arm64_toolchain",
    )
