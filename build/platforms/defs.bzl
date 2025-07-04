os = ["linux", "macos", "windows"]
arch = ["x86_64", "aarch64"]

def platform_select(format, os = os, arch = arch):
    return {
        "//build/platforms:{}_{}".format(o, a): format.format(os = o, arch = a)
        for a in arch
        for o in os
    }
