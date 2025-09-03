def eslint_test(name, eslint_bin, data = [], args = []):
    eslint_bin.eslint_test(
        name = name,
        data = data,
        chdir = native.package_name(),
        fixed_args = [
            "--color",
        ] + args,
    )
