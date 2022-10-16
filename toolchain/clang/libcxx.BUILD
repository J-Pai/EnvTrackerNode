package(default_visibility = ["//visibility:public"])

filegroup(
    name = "raw_headers",
    srcs = glob(["include/**"]),
)

cc_library(
    name = "headers",
    hdrs = [":raw_headers"],
)

cc_library(
    name = "libcxx",
    hdrs = glob(["include/**"]),
    srcs = glob([
        "src/*.cpp",
        "src/include/*.h",
        "filesystem/*.cpp",
        "filesystem/*.h",
    ]),
    textual_hdrs = glob([
        "src/support/runtime/**",
    ]),
    copts = [
        "-Iexternal/org_llvm_libcxx/src/include",
        "-Iexternal/org_llvm_libcxx/src",
        "-D_LIBCPP_BUILDING_LIBRARY",
        "-D_LIBCPP_HAS_NO_PRAGMA_SYSTEM_HEADER",
        "-DLIBCXX_BUILDING_LIBCXXABI",
        "-DLIBCXX_CXX_ABI=libstdc++",
        "-DNDEBUG",
        "-fvisibility-inlines-hidden",
    ],
    includes = ["include"],
    deps = [
        ":headers",
        "@org_llvm_libcxxabi//:libcxxabi",
    ],
)
Footer
© 2022 GitHub, Inc.
Footer navigation
Terms
Privacy
Security
