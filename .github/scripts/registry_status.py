import os
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path


def package_coordinates(manifest_path=Path("Cargo.toml")):
    package = tomllib.loads(Path(manifest_path).read_text())["package"]
    return package["name"], package["version"]


def is_published(
    manifest_path=Path("Cargo.toml"),
    opener=None,
):
    opener = opener or urllib.request.urlopen
    package, version = package_coordinates(manifest_path)
    name = urllib.parse.quote(package, safe="")
    version = urllib.parse.quote(version, safe="")
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{name}/{version}",
        headers={"User-Agent": "binlayout-release-workflow"},
    )
    try:
        with opener(request, timeout=30):
            return True
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise


def write_status(output_path, published):
    with open(output_path, "a") as output:
        output.write(f"published={str(published).lower()}\n")


def main():
    published = is_published()
    write_status(os.environ["GITHUB_OUTPUT"], published)
    if published:
        package, version = package_coordinates()
        print(f"{package} {version} is already published; skipping.")


if __name__ == "__main__":
    main()
