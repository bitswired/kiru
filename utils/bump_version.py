# /// script
# dependencies = [
#   "tomlkit",
#   "packaging",
# ]
# ///
import argparse
import subprocess
from pathlib import Path

import tomlkit
from packaging import version


def bump_version(ver: str, bump_type: str) -> str:
    v = version.parse(ver)
    major, minor, patch = v.major, v.minor, v.micro
    if bump_type == "major":
        major += 1
        minor = patch = 0
    elif bump_type == "minor":
        minor += 1
        patch = 0
    elif bump_type == "patch":
        patch += 1
    return f"{major}.{minor}.{patch}"


def main():
    parser = argparse.ArgumentParser(description="Bump versions for kiru workspace")
    parser.add_argument(
        "bump_type", choices=["major", "minor", "patch"], help="Type of version bump"
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Simulate without making changes"
    )
    args = parser.parse_args()

    core_toml = Path("kiru-core/Cargo.toml")
    py_toml = Path("kiru-py/Cargo.toml")

    # Read current versions
    with open(core_toml) as f:
        core_data = tomlkit.load(f)
    with open(py_toml) as f:
        py_data = tomlkit.load(f)

    core_ver = core_data["package"]["version"]
    py_ver = py_data["package"]["version"]

    # Bump versions
    new_core_ver = bump_version(core_ver, args.bump_type)
    new_py_ver = bump_version(py_ver, args.bump_type)

    print(f"Bumping kiru-core from {core_ver} to {new_core_ver}")
    print(f"Bumping kiru-py from {py_ver} to {new_py_ver}")

    if args.dry_run:
        print("Dry run: No changes made.")
        return

    # Update kiru-core/Cargo.toml
    core_data["package"]["version"] = new_core_ver
    with open(core_toml, "w") as f:
        tomlkit.dump(core_data, f)

    # Update kiru-py/Cargo.toml (version and dependency)
    py_data["package"]["version"] = new_py_ver
    py_data.setdefault("dependencies", {}).update(
        {"kiru": {"path": "../kiru-core", "version": new_core_ver}}
    )
    with open(py_toml, "w") as f:
        tomlkit.dump(py_data, f)

    # Update Cargo.lock
    subprocess.run(["cargo", "generate-lockfile"], check=True)


if __name__ == "__main__":
    main()
