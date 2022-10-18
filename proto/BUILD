package(default_visibility = ["//visibility:public"])

load("@com_github_grpc_grpc//bazel:cc_grpc_library.bzl", "cc_grpc_library")
load("@com_github_grpc_grpc//bazel:grpc_build_system.bzl", "grpc_proto_library")

proto_library(
    name = "core_node_proto",
    srcs = ["core_node.proto"],
)

cc_proto_library(
    name = "core_node_cc_proto",
    deps = [":core_node_proto"],
)

cc_grpc_library(
    name = "core_node_cc_grpc",
    srcs = [
        ":core_node_proto",
    ],
    grpc_only = True,
    deps = [":core_node_cc_proto"],
)
