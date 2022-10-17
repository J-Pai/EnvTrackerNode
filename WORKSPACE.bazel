workspace(name = "EnvTrackerNode")

load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

git_repository(
    name = "toolchain",
    branch = "master",
    remote = "git@github.com:J-Pai/bazel_xcompile_toolchain.git",
)

load("@toolchain//:build.bzl", "build_tools")

build_tools()
