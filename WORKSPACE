load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_foreign_cc",
    strip_prefix = "rules_foreign_cc-0.7.0",
    url = "https://github.com/bazelbuild/rules_foreign_cc/archive/0.7.0.tar.gz",
)

load("@rules_foreign_cc//foreign_cc:repositories.bzl", "rules_foreign_cc_dependencies")

rules_foreign_cc_dependencies()

_ALL_CONTENT = """\
filegroup(
    name = "all_srcs",
    srcs = glob(["**"]),
    visibility = ["//visibility:public"],
)
"""

http_archive(
    name = "mongo_c_driver",
    build_file_content = _ALL_CONTENT,
    strip_prefix = "mongo-c-driver-1.21.0",
    url = "https://github.com/mongodb/mongo-c-driver/releases/download/1.21.0/mongo-c-driver-1.21.0.tar.gz",
)

http_archive(
    name = "mongo_cxx_driver",
    build_file_content = _ALL_CONTENT,
    strip_prefix = "mongo-cxx-driver-r3.6.6",
    url = "https://github.com/mongodb/mongo-cxx-driver/releases/download/r3.6.6/mongo-cxx-driver-r3.6.6.tar.gz",
)
