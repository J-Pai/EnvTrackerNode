cc_binary(
    name = "demo_client",
    srcs = ["demo_client.cc"],
    deps = [
        "@com_github_grpc_grpc//:grpc++",
        "@com_github_google_glog//:glog",
        "@org_llvm_libcxx//:libcxx",
        "//proto:core_node_cc_grpc",
    ],
)

cc_binary(
    name = "core_node",
    srcs = [
        "core_node.cc",
    ],
    deps = [
        "@com_github_grpc_grpc//:grpc++",
        "@com_github_google_glog//:glog",
        "@org_llvm_libcxx//:libcxx",
        "//proto:core_node_cc_grpc",
    ],
)
