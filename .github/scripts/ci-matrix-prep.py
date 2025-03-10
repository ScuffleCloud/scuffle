import sys
import os
import json
import subprocess
from typing import Optional
from dataclasses import dataclass, asdict

# Stdin is the github context
GITHUB_CONTEXT: dict = json.loads(sys.stdin.read())

GITHUB_DEFAULT_RUNNER = "ubuntu-24.04"
LINUX_X86_64 = "ubicloud-standard-8"
LINUX_ARM64 = "ubicloud-standard-8-arm"
WINDOWS_X86_64 = "windows-2025"
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


def pr_number() -> Optional[int]:
    if is_pr():
        return GITHUB_CONTEXT["event"]["number"]
    elif is_brawl("try"):
        return int(GITHUB_CONTEXT["ref"].strip("refs/heads/automation/brawl/try/"))

    return None


# The output should be in the form
# matrix=<json>


@dataclass
class RustSetup:
    toolchain: str
    shared_key: Optional[str]
    components: str = ""
    tools: str = ""
    cache_backend: str = "ubicloud"


@dataclass
class FfmpegSetup:
    version: Optional[str] = None


@dataclass
class DocsMatrix:
    artifact_name: Optional[str]
    pr_number: Optional[int]
    deploy_docs: bool


@dataclass
class ClippyMatrix:
    powerset: bool


@dataclass
class TestMatrix:
    pr_number: Optional[int]
    commit_sha: str


@dataclass
class GrindMatrix:
    env: str


@dataclass
class FmtMatrix:
    pass


@dataclass
class HakariMatrix:
    pass


@dataclass
class SemverChecksMatrix:
    pass


@dataclass
class Job:
    runner: str
    job_name: str
    rust: Optional[RustSetup]
    ffmpeg: Optional[FfmpegSetup]
    inputs: (
        GrindMatrix
        | DocsMatrix
        | ClippyMatrix
        | TestMatrix
        | FmtMatrix
        | HakariMatrix
        | SemverChecksMatrix
    )
    job: str
    secrets: Optional[list[str]] = None


def create_docs_jobs() -> list[Job]:
    jobs: list[Job] = []

    deploy_docs = not is_brawl("merge") and not is_fork_pr()

    jobs.append(
        Job(
            runner=LINUX_X86_64,
            job_name="Docs (Linux x86_64)",
            job="docs",
            ffmpeg=FfmpegSetup(),
            inputs=DocsMatrix(
                artifact_name="docs",
                deploy_docs=deploy_docs,
                pr_number=pr_number(),
            ),
            rust=RustSetup(
                toolchain="nightly",
                components="rust-docs",
                shared_key="docs-linux-x86_64",
                tools="",
                cache_backend="ubicloud",
            ),
            secrets=(
                ["CF_DOCS_API_KEY", "CF_DOCS_ACCOUNT_ID"] if deploy_docs else None
            ),
        )
    )

    if is_brawl():
        jobs.append(
            Job(
                runner=LINUX_ARM64,
                job_name="Docs (Linux arm64)",
                job="docs",
                ffmpeg=FfmpegSetup(),
                inputs=DocsMatrix(
                    artifact_name=None,
                    deploy_docs=False,
                    pr_number=pr_number(),
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-docs",
                    shared_key="docs-linux-arm64",
                    tools="",
                    cache_backend="ubicloud",
                ),
            )
        )

        jobs.append(
            Job(
                runner=WINDOWS_X86_64,
                job_name="Docs (Windows x86_64)",
                job="docs",
                ffmpeg=FfmpegSetup(),
                inputs=DocsMatrix(
                    artifact_name=None,
                    deploy_docs=False,
                    pr_number=pr_number(),
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-docs",
                    shared_key="docs-windows-x86_64",
                    tools="",
                    cache_backend="github",
                ),
            )
        )

        jobs.append(
            Job(
                runner=MACOS_X86_64,
                job_name="Docs (macOS x86_64)",
                job="docs",
                ffmpeg=FfmpegSetup(),
                inputs=DocsMatrix(
                    artifact_name=None,
                    deploy_docs=False,
                    pr_number=pr_number(),
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-docs",
                    shared_key="docs-macos-x86_64",
                    tools="",
                    cache_backend="github",
                ),
            )
        )

        jobs.append(
            Job(
                runner=MACOS_ARM64,
                job_name="Docs (macOS arm64)",
                job="docs",
                ffmpeg=FfmpegSetup(),
                inputs=DocsMatrix(
                    artifact_name=None,
                    deploy_docs=False,
                    pr_number=pr_number(),
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="rust-docs",
                    shared_key="docs-macos-arm64",
                    tools="",
                    cache_backend="github",
                ),
            )
        )

    return jobs


def create_clippy_jobs() -> list[Job]:
    jobs: list[Job] = []

    jobs.append(
        Job(
            runner=LINUX_X86_64,
            job_name="Clippy (Linux x86_64)",
            job="clippy",
            ffmpeg=FfmpegSetup(),
            inputs=ClippyMatrix(
                powerset=is_brawl(),
            ),
            rust=RustSetup(
                toolchain="nightly",
                components="clippy",
                shared_key="clippy-linux-x86_64",
                tools="cargo-nextest,cargo-llvm-cov,cargo-hakari,just",
                cache_backend="ubicloud",
            ),
        )
    )

    if is_brawl():
        jobs.append(
            Job(
                runner=LINUX_ARM64,
                job_name="Clippy (Linux arm64)",
                job="clippy",
                ffmpeg=FfmpegSetup(),
                inputs=ClippyMatrix(
                    powerset=True,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="clippy",
                    shared_key="clippy-linux-arm64",
                    tools="cargo-nextest,cargo-llvm-cov,cargo-hakari,just",
                    cache_backend="ubicloud",
                ),
            )
        )

        jobs.append(
            Job(
                runner=WINDOWS_X86_64,
                job_name="Clippy (Windows x86_64)",
                job="clippy",
                ffmpeg=FfmpegSetup(),
                inputs=ClippyMatrix(
                    powerset=True,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="clippy",
                    shared_key="clippy-windows-x86_64",
                    tools="cargo-nextest,cargo-llvm-cov,cargo-hakari,just",
                    cache_backend="github",
                ),
            )
        )

        jobs.append(
            Job(
                runner=MACOS_X86_64,
                job_name="Clippy (macOS x86_64)",
                job="clippy",
                ffmpeg=FfmpegSetup(),
                inputs=ClippyMatrix(
                    powerset=True,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="clippy",
                    shared_key="clippy-macos-x86_64",
                    tools="cargo-nextest,cargo-llvm-cov,cargo-hakari,just",
                    cache_backend="github",
                ),
            )
        )

        jobs.append(
            Job(
                runner=MACOS_ARM64,
                job_name="Clippy (macOS arm64)",
                job="clippy",
                ffmpeg=FfmpegSetup(),
                inputs=ClippyMatrix(
                    powerset=True,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="clippy",
                    shared_key="clippy-macos-arm64",
                    tools="cargo-nextest,cargo-llvm-cov,cargo-hakari,just",
                    cache_backend="github",
                ),
            )
        )

    return jobs


def create_test_jobs() -> list[Job]:
    jobs: list[Job] = []

    secrets = ["CODECOV_TOKEN"] if not is_fork_pr() else None

    commit_sha = os.environ["SHA"]
    if is_brawl("try"):
        commit_sha = (
            subprocess.check_output(["git", "log", "-n", "1", "--pretty=format:%H"])
            .decode()
            .strip()
        )

    jobs.append(
        Job(
            runner=LINUX_X86_64,
            job_name="Test (Linux x86_64)",
            job="test",
            ffmpeg=FfmpegSetup(),
            inputs=TestMatrix(
                pr_number=pr_number(),
                commit_sha=commit_sha,
            ),
            rust=RustSetup(
                toolchain="nightly",
                components="llvm-tools-preview",
                shared_key="test-linux-x86_64",
                tools="cargo-nextest,cargo-llvm-cov",
                cache_backend="ubicloud",
            ),
            secrets=secrets,
        )
    )

    if is_brawl():
        jobs.append(
            Job(
                runner=LINUX_ARM64,
                job_name="Test (Linux arm64)",
                job="test",
                ffmpeg=FfmpegSetup(),
                inputs=TestMatrix(
                    pr_number=pr_number(),
                    commit_sha=commit_sha,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="llvm-tools-preview",
                    shared_key="test-linux-arm64",
                    tools="cargo-nextest,cargo-llvm-cov",
                    cache_backend="ubicloud",
                ),
                secrets=secrets,
            )
        )

        jobs.append(
            Job(
                runner=WINDOWS_X86_64,
                job_name="Test (Windows x86_64)",
                job="test",
                ffmpeg=FfmpegSetup(),
                inputs=TestMatrix(
                    pr_number=pr_number(),
                    commit_sha=commit_sha,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="llvm-tools-preview",
                    shared_key="test-windows-x86_64",
                    tools="cargo-nextest,cargo-llvm-cov",
                    cache_backend="github",
                ),
                secrets=secrets,
            )
        )

        jobs.append(
            Job(
                runner=MACOS_X86_64,
                job_name="Test (macOS x86_64)",
                job="test",
                ffmpeg=FfmpegSetup(),
                inputs=TestMatrix(
                    pr_number=pr_number(),
                    commit_sha=commit_sha,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="llvm-tools-preview",
                    shared_key="test-macos-x86_64",
                    tools="cargo-nextest,cargo-llvm-cov",
                    cache_backend="github",
                ),
                secrets=secrets,
            )
        )

        jobs.append(
            Job(
                runner=MACOS_ARM64,
                job_name="Test (macOS arm64)",
                job="test",
                ffmpeg=FfmpegSetup(),
                inputs=TestMatrix(
                    pr_number=pr_number(),
                    commit_sha=commit_sha,
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    components="llvm-tools-preview",
                    shared_key="test-macos-arm64",
                    tools="cargo-nextest,cargo-llvm-cov",
                    cache_backend="github",
                ),
                secrets=secrets,
            )
        )

    return jobs


def create_grind_jobs() -> list[Job]:
    jobs: list[Job] = []

    if is_brawl():
        jobs.append(
            Job(
                runner=LINUX_X86_64,
                job_name="Grind (Linux x86_64)",
                job="grind",
                ffmpeg=FfmpegSetup(),
                inputs=GrindMatrix(
                    env=json.dumps(
                        {
                            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": "valgrind --error-exitcode=1 --leak-check=full --gen-suppressions=all --suppressions=$(pwd)/valgrind_suppressions.log",
                        }
                    ),
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    shared_key="grind-linux-x86_64",
                    tools="cargo-nextest",
                    cache_backend="ubicloud",
                ),
            )
        )

        jobs.append(
            Job(
                runner=LINUX_ARM64,
                job_name="Grind (Linux arm64)",
                job="grind",
                ffmpeg=FfmpegSetup(),
                inputs=GrindMatrix(
                    env=json.dumps(
                        {
                            "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER": "valgrind --error-exitcode=1 --leak-check=full --gen-suppressions=all --suppressions=$(pwd)/valgrind_suppressions.log",
                        }
                    ),
                ),
                rust=RustSetup(
                    toolchain="nightly",
                    shared_key="grind-linux-arm64",
                    tools="cargo-nextest",
                    cache_backend="ubicloud",
                ),
            )
        )

    return jobs


def create_fmt_jobs() -> list[Job]:
    jobs: list[Job] = []

    jobs.append(
        Job(
            runner=GITHUB_DEFAULT_RUNNER,
            job_name="Fmt",
            job="fmt",
            ffmpeg=None,
            inputs=FmtMatrix(),
            rust=RustSetup(
                toolchain="nightly",
                components="rustfmt",
                shared_key=None,
                cache_backend="github",
            ),
        )
    )

    return jobs


def create_hakari_jobs() -> list[Job]:
    jobs: list[Job] = []

    jobs.append(
        Job(
            runner=GITHUB_DEFAULT_RUNNER,
            job_name="Hakari",
            job="hakari",
            ffmpeg=None,
            inputs=HakariMatrix(),
            rust=RustSetup(
                toolchain="nightly",
                components="rustfmt",
                tools="cargo-hakari",
                shared_key=None,
                cache_backend="github",
            ),
        )
    )

    return jobs


def create_semver_checks_jobs() -> list[Job]:
    jobs: list[Job] = []

    jobs.append(
        Job(
            runner=LINUX_X86_64,
            job_name="Semver-checks",
            job="semver-checks",
            ffmpeg=FfmpegSetup(),
            inputs=SemverChecksMatrix(),
            rust=RustSetup(
                toolchain="stable",
                components="rust-docs",
                tools="cargo-semver-checks",
                shared_key="cargo-semver-checks",
                cache_backend="ubicloud",
            ),
        )
    )

    return jobs


def create_jobs() -> list[Job]:
    jobs = (
        create_docs_jobs()
        + create_clippy_jobs()
        + create_test_jobs()
        + create_grind_jobs()
        + create_fmt_jobs()
        + create_hakari_jobs()
        + create_semver_checks_jobs()
    )

    return jobs


def main():
    print(f"matrix={json.dumps([asdict(job) for job in create_jobs()])}")


if __name__ == "__main__":
    main()
