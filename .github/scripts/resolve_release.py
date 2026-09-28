import os
import subprocess
import tomllib
from pathlib import Path


def expected_tag(manifest_path=Path("Cargo.toml")):
    manifest = tomllib.loads(Path(manifest_path).read_text())
    return "v" + manifest["package"]["version"]


def resolve_release(
    release_tag,
    output_path,
    manifest_path=Path("Cargo.toml"),
    rev_parse=None,
):
    expected = expected_tag(manifest_path)
    if release_tag != expected:
        raise SystemExit(
            f"Tag {release_tag!r} must match package version {expected!r}"
        )

    if rev_parse is None:
        rev_parse = lambda: subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip()
    commit = rev_parse()
    with open(output_path, "a") as output:
        output.write(f"commit={commit}\n")
    return commit


def main():
    resolve_release(
        os.environ["RELEASE_TAG"],
        os.environ["GITHUB_OUTPUT"],
    )


if __name__ == "__main__":
    main()
