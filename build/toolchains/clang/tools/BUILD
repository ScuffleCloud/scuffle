# Copyright 2024 The Bazel Authors. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

load("@rules_cc//cc/toolchains:tool.bzl", "cc_tool")
load("@rules_cc//cc/toolchains:tool_map.bzl", "cc_tool_map")

licenses(["notice"])

# This `select` happens under the target configuration. For macOS,
# llvm-libtool-darwin should be used when creating static libraries even if the
# exec platform is linux.
alias(
    name = "all_tools",
    actual = select({
        "@platforms//os:macos": ":macos-tools",
        "//conditions:default": ":default_tools",
    }),
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

COMMON_TOOLS = {
    "@rules_cc//cc/toolchains/actions:assembly_actions": ":clang",
    "@rules_cc//cc/toolchains/actions:c_compile": ":clang",
    "@rules_cc//cc/toolchains/actions:cpp_compile_actions": ":clang++",
    "@rules_cc//cc/toolchains/actions:link_actions": ":lld",
    "@rules_cc//cc/toolchains/actions:objcopy_embed_data": ":llvm-objcopy",
    "@rules_cc//cc/toolchains/actions:strip": ":llvm-strip",
}

cc_tool_map(
    name = "default_tools",
    tags = ["manual", "no-cache"],
    tools = COMMON_TOOLS | {
        "@rules_cc//cc/toolchains/actions:ar_actions": ":llvm-ar",
    },
    visibility = ["//visibility:private"],
)

cc_tool_map(
    name = "macos-tools",
    tags = ["manual"],
    tools = COMMON_TOOLS | {
        "@rules_cc//cc/toolchains/actions:ar_actions": ":llvm-libtool-darwin",
    },
    visibility = ["//visibility:private"],
)

cc_tool(
    name = "clang",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/clang",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/clang",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/clang",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/clang",
    }),
    data = [
        ":exec_platform_builtin_headers",
        ":exec_platform_multicall_support_files",
    ],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "clang++",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/clang++",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/clang++",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/clang++",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/clang++",
    }),
    data = [
        ":exec_platform_builtin_headers",
        ":exec_platform_multicall_support_files",
    ],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "lld",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/clang++",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/clang++",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/clang++",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/clang++",
    }),
    data = [
        ":exec_platform_linker_builtins",
        ":exec_platform_multicall_support_files",
    ],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "llvm-ar",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/llvm-ar",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/llvm-ar",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/llvm-ar",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/llvm-ar",
    }),
    data = [":exec_platform_multicall_support_files"],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "llvm-libtool-darwin",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/llvm-libtool-darwin",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/llvm-libtool-darwin",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/llvm-libtool-darwin",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/llvm-libtool-darwin",
    }),
    data = [":exec_platform_multicall_support_files"],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "llvm-objcopy",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/llvm-objcopy",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/llvm-objcopy",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/llvm-objcopy",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/llvm-objcopy",
    }),
    data = [":exec_platform_multicall_support_files"],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "llvm-objdump",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/llvm-objdump",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/llvm-objdump",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/llvm-objdump",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/llvm-objdump",
    }),
    data = [":exec_platform_multicall_support_files"],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "llvm-cov",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/llvm-cov",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/llvm-cov",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/llvm-cov",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/llvm-cov",
    }),
    data = [":exec_platform_multicall_support_files"],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "llvm-strip",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/llvm-strip",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/llvm-strip",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/llvm-strip",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/llvm-strip",
    }),
    data = [":exec_platform_multicall_support_files"],
    tags = ["manual", "no-cache"],
)

cc_tool(
    name = "clang-tidy",
    src = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:bin/clang-tidy",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:bin/clang-tidy",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:bin/clang-tidy",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:bin/clang-tidy",
    }),
    data = [
        ":exec_platform_builtin_headers",
        ":exec_platform_multicall_support_files",
    ],
    tags = ["manual", "no-cache"],
)

#################################
#   Platform-specific aliases   #
#################################

# These aliases are used to reduce duplication of `select` statements throughout
# this build file. The select statements in these aliases are evaluated under
# the exec configuration.

alias(
    name = "exec_platform_builtin_headers",
    actual = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:builtin_headers",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:builtin_headers",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:builtin_headers",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:builtin_headers",
    }),
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

alias(
    name = "exec_platform_multicall_support_files",
    actual = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:multicall_support_files",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:multicall_support_files",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:multicall_support_files",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:multicall_support_files",
    }),
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:private"],
)

alias(
    name = "exec_platform_linker_builtins",
    actual = select({
        "//build/platforms:linux-aarch64": "@clang-linux-aarch64//:linker_builtins",
        "//build/platforms:linux-x86_64": "@clang-linux-x86_64//:linker_builtins",
        "//build/platforms:macos-aarch64": "@clang-macos-aarch64//:linker_builtins",
        "//build/platforms:macos-x86_64": "@clang-macos-x86_64//:linker_builtins",
    }),
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:private"],
)
