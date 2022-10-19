# Build Rules for openSSL.
# Rules pulled from:
# https://github.com/openssl/openssl/issues/3840#issuecomment-380006002
#
# Utilizing cc_library approach so boringSSL can be replaced with openSSL
# during the build.

cc_library(
    name = "crypto",
    hdrs = glob(
        ["include/openssl/*.h"],
        exclude = ["include/openssl/opensslconf.h"],
    ) + ["include/openssl/opensslconf.h"],
    srcs = ["libcrypto.a"],
    includes = ["include"],
    linkopts = ["-lpthread", "-ldl"],
    visibility = ["//visibility:public"],
    data = [":openssl-build"],
)

cc_library(
    name = "ssl",
    deps = [":crypto"],
    hdrs = glob(
        ["include/openssl/*.h"],
        exclude = ["include/openssl/opensslconf.h"],
    ) + ["include/openssl/opensslconf.h"],
    srcs = ["libssl.a"],
    includes = ["include"],
    visibility = ["//visibility:public"],
)

genrule(
    name = "openssl-build",
    srcs = glob(
        ["**/*"],
        exclude=[
            "bazel-*",
            "libcrypto.a",
            "libssl.a",
            "include/openssl/opensslconf.h",
        ]
    ),
    outs = [
        "libcrypto.a",
        "libssl.a",
        "include/openssl/opensslconf.h",
    ],
    cmd = """
          OPENSSL_ROOT=$$(dirname $(location Configure))
          pushd $$OPENSSL_ROOT
              export CC=../../$(CC)
              export CC_DIR=$$(dirname $${CC})
              export AR=$${CC_DIR}/llvm-ar
              export NM=$${CC_DIR}/llvm-nm
              export RANLIB=$${CC_DIR}/llvm-ranlib
              export CFLAGS="$(CC_FLAGS)"
              export TARGET=linux-x86_64
              if [[ "$${CFLAGS}" == *"target=aarch64-linux"* ]]; then
                  export TARGET=linux-aarch64
              fi
              ./Configure no-shared $${TARGET}
              make -s -j$$(nproc)
          popd
          cp $$OPENSSL_ROOT/libcrypto.a $(location libcrypto.a)
          cp $$OPENSSL_ROOT/libssl.a $(location libssl.a)
          cp $$OPENSSL_ROOT/include/openssl/opensslconf.h $(location include/openssl/opensslconf.h)
          """,
    message = "Generating OpenSSL libraries",
    toolchains = [
        "@bazel_tools//tools/cpp:current_cc_toolchain",
        "@bazel_tools//tools/cpp:cc_flags",
    ],
)
