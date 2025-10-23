# /// script
# dependencies = [
#   "tomlkit",
#   "packaging",
# ]
# ///
import subprocess
import sys
from pathlib import Path

import tomlkit
from packaging import version


def get_latest_tag() -> str:
    try:
        result = subprocess.run(
            ["git", "tag", "--sort=-v:refname"],
            capture_output=True,
            text=True,
            check=True,
        )
        tags = result.stdout.splitlines()
        for tag in tags:
            if (
                tag.startswith("v") and version.parse(tag[1:]).release
            ):  # Ensure valid semver
                return tag[1:]  # Strip 'v'
        return "0.0.0"  # Fallback if no tags
    except subprocess.CalledProcessError:
        return "0.0.0"  # Fallback if git fails


def main():
    core_toml = Path("kiru-core/Cargo.toml")
    py_toml = Path("kiru-py/Cargo.toml")

    # Read versions
    with open(core_toml) as f:
        core_ver = tomlkit.load(f)["package"]["version"]
    with open(py_toml) as f:
        py_ver = tomlkit.load(f)["package"]["version"]

    # Get latest tag
    latest_tag = get_latest_tag()
    print(f"Latest tag: v{latest_tag}")
    print(f"kiru-core version: {core_ver}")
    print(f"kiru-py version: {py_ver}")

    # Compare versions
    latest_ver = version.parse(latest_tag)
    core_ver_obj = version.parse(core_ver)
    py_ver_obj = version.parse(py_ver)

    if core_ver_obj <= latest_ver:
        print(
            f"Error: kiru-core version ({core_ver}) is not newer than latest tag ({latest_tag})"
        )
        sys.exit(1)
    if py_ver_obj <= latest_ver:
        print(
            f"Error: kiru-py version ({py_ver}) is not newer than latest tag ({latest_tag})"
        )
        sys.exit(1)

    print("All versions are newer than the latest tag.")


if __name__ == "__main__":
    main()
