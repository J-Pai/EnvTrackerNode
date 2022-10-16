def _cc_toolchain_config_impl(ctx):
    print(ctx.attr.name)
    return [
        cc_common.create_cc_toolchain_config_info(
            ctx = ctx,
            features = [],
            action_configs = [],
            artifact_name_patterns = [],
            cxx_builtin_include_directories = [],
            toolchain_identifier = "",
            host_system_name = "linux",
            target_system_name = "linux",
            target_cpu = "cpu",
            target_libc = "libc",
            compiler = "clang",
            abi_version = "local",
            abi_libc_version = "local",
            tool_paths = [],
            make_variables = [],
            builtin_sysroot = None,
            cc_target_os = None,
        )
    ]

cc_toolchain_config = rule(
    implementation = _cc_toolchain_config_impl,
    attrs = {
    },
    provides = [CcToolchainConfigInfo],
)
