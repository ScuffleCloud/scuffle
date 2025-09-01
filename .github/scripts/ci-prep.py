import sys
import os
import json
import subprocess
from typing import Optional
from dataclasses import dataclass, asdict

# Stdin is the github context
GITHUB_CONTEXT: dict = json.loads(sys.stdin.read())

GITHUB_DEFAULT_RUNNER = "ubuntu-24.04"
LINUX_X86_64 = "ubicloud-standard-8-ubuntu-2404"
LINUX_ARM64 = "ubicloud-standard-8-arm-ubuntu-2404"
WINDOWS_X86_64 = "windows-2025"
WINDOWS_ARM = "windows-11-arm"
MACOS_X86_64 = "macos-13"
MACOS_ARM64 = "macos-15"


def is_brawl(mode: Optional[str] = None) -> bool:
    if mode is None:
        mode = ""
    else:
        mode = f"{mode}/"

    return GITHUB_CONTEXT["event_name"] == "push" and GITHUB_CONTEXT["ref"].startswith(
        f"refs/heads/automation/brawl/{mode}"
    )


def is_pr() -> bool:
    return GITHUB_CONTEXT["event_name"] == "pull_request"


def is_fork_pr() -> bool:
    return (
        is_pr()
        and GITHUB_CONTEXT["event"]["pull_request"]["head"]["repo"][
            "full_name"
        ].casefold()
        != "scufflecloud/scuffle".casefold()
    )


def is_dispatch_or_cron() -> bool:
    return GITHUB_CONTEXT["event_name"] in ["workflow_dispatch", "schedule"]


def pr_number() -> Optional[int]:
    if is_pr():
        return GITHUB_CONTEXT["event"]["number"]
    elif is_brawl("try"):
        return int(GITHUB_CONTEXT["ref"].strip("refs/heads/automation/brawl/try/"))

    return None


# The output should be in the form
# matrix=<json>


@dataclass
class DeployArtifact:
    deploy: bool
    artifact_name: str


@dataclass
class Deploy:
    pr_number: Optional[int]
    commit_sha: str
    rustdoc: Optional[DeployArtifact]
    docs: Optional[DeployArtifact]
    dashboard: Optional[DeployArtifact]


@dataclass
class MatrixEntry:
    runner: str
    os: str
    arch: str


@dataclass
class Test:
    pr_number: Optional[int]
    commit_sha: str
    matrix: list[MatrixEntry]


@dataclass
class Grind:
    pr_number: Optional[int]
    commit_sha: str
    matrix: list[MatrixEntry]


@dataclass
class CheckVendor:
    pass


@dataclass
class CheckFmt:
    pass


@dataclass
class Jobs:
    deploy: Optional[Deploy]
    test: Optional[Test]
    grind: Optional[Grind]
    check_vendor: Optional[CheckVendor]
    check_fmt: Optional[CheckFmt]


def deploy_docs() -> bool:
    return not is_brawl("merge") and not is_fork_pr() and not is_dispatch_or_cron()


def commit_sha() -> str:
    return os.environ["SHA"]


def create_deploy() -> Optional[Deploy]:
    return Deploy(
        pr_number=pr_number(),
        commit_sha=commit_sha() or "",
        rustdoc=DeployArtifact(deploy=deploy_docs(), artifact_name="rustdoc")
        if os.path.exists("docs")
        or os.path.exists("target-bazel/bin/docs")
        or os.path.exists("docs/rustdoc")
        else None,
        docs=DeployArtifact(deploy=deploy_docs(), artifact_name="docs")
        if os.path.exists("cloud/docs") or os.path.exists("docs")
        else None,
        dashboard=DeployArtifact(deploy=deploy_docs(), artifact_name="dashboard")
        if os.path.exists("cloud/dashboard")
        else None,
    )


def create_test() -> Optional[Test]:
    matrix = [MatrixEntry(runner=LINUX_X86_64, os="linux", arch="x86_64")]
    if is_brawl() or is_dispatch_or_cron():
        matrix.append(MatrixEntry(runner=LINUX_ARM64, os="linux", arch="aarch64"))

    return Test(
        pr_number=pr_number(),
        commit_sha=commit_sha(),
        matrix=matrix,
    )


def create_grind() -> Optional[Grind]:
    if not is_brawl() and not is_dispatch_or_cron():
        return None

    return Grind(
        commit_sha=commit_sha(),
        pr_number=pr_number(),
        matrix=[
            MatrixEntry(runner=LINUX_X86_64, os="linux", arch="x86_64"),
            # Currently aarch64 has an issue with valgrind not respecting
            # padding being uninitialized, reporting it as a "jump based on uninitialized"
            # MatrixEntry(runner=LINUX_ARM64, os="linux", arch="aarch64"),
        ],
    )


def create_check_vendor() -> Optional[CheckVendor]:
    return CheckVendor()


def create_fmt() -> Optional[CheckFmt]:
    return CheckFmt()


def create_jobs() -> Jobs:
    return Jobs(
        deploy=create_deploy(),
        check_vendor=create_check_vendor(),
        grind=create_grind(),
        test=create_test(),
        check_fmt=create_fmt(),
    )


def main():
    print(f"prep={json.dumps(asdict(create_jobs()))}")


if __name__ == "__main__":
    main()
