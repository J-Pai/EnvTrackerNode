load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "action_config",
    "feature",
    "flag_group",
    "flag_set",
    "env_entry",
    "env_set",
    "tool",
    "tool_path",
)

load(
    "@bazel_tools//tools/build_defs/cc:action_names.bzl",
    "ACTION_NAMES",
)

def _cc_toolchain_config_impl(ctx):
    print(ctx.attr.name, ctx.attr.target_cpu)

    all_compile_actions = [
        ACTION_NAMES.c_compile,
        ACTION_NAMES.cpp_compile,
        ACTION_NAMES.linkstamp_compile,
        ACTION_NAMES.cc_flags_make_variable,
        ACTION_NAMES.cpp_module_codegen,
        ACTION_NAMES.cpp_header_parsing,
        ACTION_NAMES.cpp_module_compile,
        ACTION_NAMES.assemble,
        ACTION_NAMES.preprocess_assemble,
        ACTION_NAMES.lto_indexing,
        ACTION_NAMES.lto_backend,
        ACTION_NAMES.strip,
        ACTION_NAMES.clif_match,
    ]

    all_link_actions = [
        ACTION_NAMES.cpp_link_executable,
        ACTION_NAMES.cpp_link_dynamic_library,
        ACTION_NAMES.cpp_link_nodeps_dynamic_library,
    ]

    pic_feature = feature(
        name = "pic",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(flags = ["-fPIC"], expand_if_available = "pic"),
                    flag_group(
                        flags = ["-fPIE"],
                        expand_if_not_available = "pic",
                    ),
                ],
            ),
        ],
    )

    supports_pic_feature = feature(name = "supports_pic", enabled = True)

    aarch64_feature = feature(
        name = "aarch64",
        flag_sets = [
            flag_set(
                actions = all_compile_actions + all_link_actions,
                flag_groups = [
                    flag_group(
                        flags = [
                            "--target=aarch64-linux-gnueabihf",
                        ],
                    ),
                ],
            ),
        ],
    )

    determinism_feature = feature(
        name = "determinism",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-Wno-builtin-macro-redefined",
                            "-D__DATE__=\"redacted\"",
                            "-D__TIMESTAMP__=\"redacted\"",
                            "-D__TIME__=\"redacted\"",
                        ],
                    ),
                ],
            ),
        ],
    )

    hardening_feature = feature(
        name = "hardening",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-U_FORTIFY_SOURCE",
                            "-D_FORTIFY_SOURCE=1",
                            "-fstack-protector",
                        ],
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                ],
                flag_groups = [flag_group(flags = ["-Wl,-z,relro,-z,now"])],
            ),
            flag_set(
                actions = [ACTION_NAMES.cpp_link_executable],
                flag_groups = [flag_group(flags = ["-pie", "-Wl,-z,relro,-z,now"])],
            ),
        ],
    )

    supports_dynamic_linker_feature = feature(name = "supports_dynamic_linker", enabled = True)

    warnings_feature = feature(
        name = "warnings",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = ["-Wall"],
                    ),
                ],
            ),
        ],
    )

    dbg_feature = feature(
        name = "dbg",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [flag_group(flags = ["-g3", "-ggdb"])],
            ),
        ],
        implies = ["common"],
    )

    disable_assertions_feature = feature(
        name = "disable-assertions",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [flag_group(flags = ["-DNDEBUG"])],
            ),
        ],
    )

    fastbuild_feature = feature(name = "fastbuild", implies = ["common"])

    user_compile_flags_feature = feature(
        name = "user_compile_flags",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["%{user_compile_flags}"],
                        iterate_over = "user_compile_flags",
                        expand_if_available = "user_compile_flags",
                    ),
                ],
            ),
        ],
    )

    opt_feature = feature(
        name = "opt",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [
                    flag_group(
                        flags = ["-O2", "-ffunction-sections", "-fdata-sections"],
                    ),
                ],
            ),
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [flag_group(flags = ["-Wl,--gc-sections"])],
            ),
        ],
        implies = ["common", "disable-assertions"],
    )

    frame_pointer_feature = feature(
        name = "frame-pointer",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.c_compile, ACTION_NAMES.cpp_compile],
                flag_groups = [flag_group(flags = ["-fno-omit-frame-pointer"])],
            ),
        ],
    )

    sysroot_feature = feature(
        name = "sysroot",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                ],
                flag_groups = [
                    flag_group(
                        flags = ["--sysroot=%{sysroot}"],
                        iterate_over = "sysroot",
                        expand_if_available = "sysroot",
                    ),
                ],
            ),
        ],
    )

    stdlib_feature = feature(
        name = "stdlib",
        flag_sets = [
            flag_set(
                actions = all_compile_actions,
                flag_groups = [flag_group(flags = [
                    "-nostdinc++",
                    "-isystem", "external/toolchain/clang/include",
                    "-isystem", "external/org_llvm_libcxx/include",
                    "-isystem", "external/org_llvm_libcxxabi/include",
                ])],
            ),
            flag_set(
                actions = all_link_actions,
                flag_groups = [flag_group(flags = [
                    "-lm",
                    "-lpthread",
                    "-ldl",
                    "-lrt",
                ])],
            ),
        ],
    )

    alwayslink_feature = feature(
        name = "alwayslink",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_executable,
                ],
                flag_groups = [flag_group(flags = ["-Wl,-no-as-needed"])],
            ),
        ],
    )

    no_canonical_prefixes_feature = feature(
        name = "no-canonical-prefixes",
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                ],
                flag_groups = [
                    flag_group(
                        flags = [
                            "-no-canonical-prefixes",
                        ],
                    ),
                ],
            ),
        ],
    )

    user_link_flags_feature = feature(
        name = "user_link_flags",
        flag_sets = [
            flag_set(
                actions = all_link_actions,
                flag_groups = [
                    flag_group(
                        flags = ["%{user_link_flags}"],
                        iterate_over = "user_link_flags",
                        expand_if_available = "user_link_flags",
                    ),
                ],
            ),
        ],
    )

    cpp20_feature = feature(
        name = "c++20",
        flag_sets = [
            flag_set(
                actions = [ACTION_NAMES.cpp_compile],
                flag_groups = [flag_group(flags = ["-std=c++20"])],
            ),
        ],
    )

    lld_feature = feature(
        name = "lld",
        flag_sets = [
            flag_set(
                actions = all_link_actions,
                flag_groups = [flag_group(flags= [
                    "-fuse-ld=lld",
                ])],
            ),
        ],
    )

    additional_features = []
    if ctx.attr.target_cpu == "arm64":
        additional_features.extend([
            "aarch64",
        ])

    common_feature = feature(
        name = "common",
        implies = [
            "stdlib",
            "c++20",
            "determinism",
            "alwayslink",
            "hardening",
            "warnings",
            "frame-pointer",
            "no-canonical-prefixes",
            "lld",
        ] + additional_features,
    )

    features = [
        cpp20_feature,
        stdlib_feature,
        determinism_feature,
        alwayslink_feature,
        pic_feature,
        hardening_feature,
        warnings_feature,
        frame_pointer_feature,
        no_canonical_prefixes_feature,
        disable_assertions_feature,
        common_feature,
        opt_feature,
        fastbuild_feature,
        dbg_feature,
        supports_dynamic_linker_feature,
        supports_pic_feature,
        lld_feature,
        aarch64_feature,
    ]

    cxx_builtin_include_directories = ctx.attr.builtin_include_directories

    tool_paths = [
        tool_path(name = "gcc", path = "bin/clang"),
        tool_path(name = "ar", path = "bin/llvm-ar"),
        tool_path(name = "compat-ld", path = "bin/ld.lld"),
        tool_path(name = "cpp", path = "bin/clang-cpp"),
        tool_path(name = "dwp", path = "bin/llvm-dwp"),
        tool_path(name = "gcov", path = "bin/fake"),
        tool_path(name = "ld", path = "bin/ld.lld"),
        tool_path(name = "nm", path = "bin/llvm-nm"),
        tool_path(name = "objcopy", path = "bin/llvm-objcopy"),
        tool_path(name = "objdump", path = "bin/llvm-objdump"),
        tool_path(name = "strip", path = "bin/llvm-strip"),
    ]

    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        features = features,
        action_configs = [],
        artifact_name_patterns = [],
        cxx_builtin_include_directories = cxx_builtin_include_directories,
        toolchain_identifier = ctx.attr.name,
        host_system_name = "linux",
        target_system_name = ctx.attr.target_system_name,
        target_cpu = ctx.attr.target_cpu,
        target_libc = "libc",
        compiler = "clang",
        abi_version = "local",
        abi_libc_version = "local",
        tool_paths = tool_paths,
        make_variables = [],
        builtin_sysroot = None,
        cc_target_os = None,
    )

cc_toolchain_config = rule(
    implementation = _cc_toolchain_config_impl,
    attrs = {
        "builtin_include_directories": attr.string_list(),
        "target_cpu": attr.string(),
        "target_system_name": attr.string(),
    },
    provides = [CcToolchainConfigInfo],
)
