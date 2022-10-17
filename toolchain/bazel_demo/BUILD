cc_binary(
    name = "hello_world",
    srcs = ["hello_world.cc"],
    deps = [
        "//bazel_demo:hello_lib",
    ],
)

cc_library(
    name = "hello_lib",
    srcs = ["hello_lib.cc"],
    hdrs = ["hello_lib.h"],
    deps = [
        "@org_llvm_libcxx//:libcxx",
    ],
)
