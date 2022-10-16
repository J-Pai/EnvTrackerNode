load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def build_tools():
    http_archive(
        name = "org_llvm_clang",
        urls = [
            "https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04.tar.xz",
        ],
        sha256 = "2c2fb857af97f41a5032e9ecadf7f78d3eff389a5cd3c9ec620d24f134ceb3c8",
        strip_prefix = "clang+llvm-13.0.0-x86_64-linux-gnu-ubuntu-20.04",
        build_file = Label("//clang:package.BUILD"),
    )
    http_archive(
        name = "org_llvm_libcxx",
        urls = [
            "https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/libcxx-13.0.0.src.tar.xz",
        ],
        sha256 = "3682f16ce33bb0a8951fc2c730af2f9b01a13b71b2b0dc1ae1e7034c7d86ca1a",
        strip_prefix = "libcxx-13.0.0.src",
        build_file = Label("//clang:libcxx.BUILD"),
    )
    http_archive(
        name = "org_llvm_libcxxabi",
        urls = [
            "https://github.com/llvm/llvm-project/releases/download/llvmorg-13.0.0/libcxxabi-13.0.0.src.tar.xz",
        ],
        sha256 = "becd5f1cd2c03cd6187558e9b4dc8a80b6d774ff2829fede88aa1576c5234ce3",
        strip_prefix = "libcxxabi-13.0.0.src",
        build_file = Label("//clang:libcxxabi.BUILD"),
    )

    native.register_toolchains(
        "@toolchain//:linux_x86_64_toolchain",
        "@toolchain//:linux_aarch64_toolchain",
    )
