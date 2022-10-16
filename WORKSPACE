workspace(name = "EnvTrackerNode")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

local_repository(
  name = "toolchain",
  path = "toolchain",
)

load("@toolchain//:build.bzl", "build_tools")

build_tools()
