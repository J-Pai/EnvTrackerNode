package(default_visibility = ["//visibility:public"])

filegroup(
    name = "raw_headers",
    srcs = glob(["include/**"]),
)

cc_library(
    name = "libcxxabi",
    hdrs = [":raw_headers"],
    srcs = glob([
        "src/*.cpp",
        "src/*.hpp",
        "src/*.h",
        "src/include/*.h",
        "src/demangle/*.h",
    ], exclude = [
        "src/stdlib_new_delete.cpp",
        "src/cxa_noexception.cpp",
    ]),
    copts = [
        "-Iexternal/org_llvm_libcxx/include",
        "-D_LIBCPP_BUILDING_LIBRARY",
        "-DNDEBUG",
        "-fvisibility-inlines-hidden",
    ],
    includes = ["include"],
    deps = ["@org_llvm_libcxx//:headers"],
)
