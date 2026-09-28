import json
import os
import re
import subprocess
import tomllib
from pathlib import Path

VERSION_RE = re.compile(
    r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
)


def version_and_tag(manifest_path=Path("Cargo.toml")):
    version = tomllib.loads(Path(manifest_path).read_text())["package"]["version"]
    if not VERSION_RE.fullmatch(version):
        raise SystemExit(f"Invalid package version: {version!r}")
    return version, "v" + version


def api(repository, path, allow_missing=False, runner=None):
    runner = runner or subprocess.run
    result = runner(
        ["gh", "api", f"repos/{repository}/{path}"],
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return json.loads(result.stdout)
    if allow_missing and "(HTTP 404)" in result.stderr:
        return None
    raise SystemExit(result.stderr.strip() or "GitHub API request failed")


def create_or_reuse_release(
    repository,
    target,
    output_path,
    manifest_path=Path("Cargo.toml"),
    api_func=None,
    runner=None,
):
    api_func = api_func or api
    runner = runner or subprocess.run
    version, tag = version_and_tag(manifest_path)
    with open(output_path, "a") as output:
        output.write(f"tag={tag}\n")

    if api_func(repository, f"releases/tags/{tag}", True) is not None:
        print(f"Release {tag} already exists; skipping.")
        return "existing"

    reference = api_func(repository, f"git/ref/tags/{tag}", True)
    if reference is not None:
        obj = reference["object"]
        for _ in range(16):
            if obj["type"] != "tag":
                break
            obj = api_func(repository, f"git/tags/{obj['sha']}")["object"]
        if obj["type"] != "commit" or obj["sha"] != target:
            raise SystemExit(
                f"Tag {tag} does not point to the tested commit; refusing to change it"
            )

    command = [
        "gh",
        "release",
        "create",
        tag,
        "--repo",
        repository,
        "--target",
        target,
        "--title",
        tag,
        "--generate-notes",
    ]
    if "-" in version.split("+", 1)[0]:
        command.append("--prerelease")
    runner(command, check=True)
    return "created"


def main():
    repository = os.environ["GH_REPO"]
    target = os.environ["RELEASE_SHA"]
    if not re.fullmatch(r"[0-9a-f]{40}", target):
        raise SystemExit("Expected the full SHA of the successful CI commit")
    create_or_reuse_release(repository, target, os.environ["GITHUB_OUTPUT"])


if __name__ == "__main__":
    main()
