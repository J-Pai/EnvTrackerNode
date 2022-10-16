def _sysroot_impl(ctx):
    result = ctx.execute([
        "bash",
        "-c",
        "mkdir -p sysroot && tar xvf {} -C sysroot".format(ctx.path(ctx.attr.sysroot)),
    ])

    if result.return_code != 0:
        fail(result.stdout + result.stderr)

    ctx.file("WORKSPACE", "workspace(name = \"{}\")\n".format(ctx.name))

    ctx.file("BUILD", "")
    ctx.file(
        "sysroot/BUILD",
        """
            filegroup(
                name="sysroot",
                srcs=glob(["**"]),
                visibility = ["//visibility:public"],
            )
        """,
    )

_sysroot_repository = repository_rule(
    implementation = _sysroot_impl,
    "sysroot": attr.label(
        allow_files = True,
    ),
)

def sysroot_file(target = ""):
    if target == "ubuntu_aarch64":
        _sysroot_repository(
            name = "ubuntu_aarch64",
            sysroot = "//sysroot:ubuntu_aarch64_sysroot.tar.xz",
        )
