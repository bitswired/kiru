# /// script
# dependencies = [
#   "tomlkit",
# ]
# ///

import sys
from pathlib import Path

import tomlkit


def main():
    core_toml = Path("kiru-core/Cargo.toml")
    py_toml = Path("kiru-py/Cargo.toml")
    pyproject_toml = Path("kiru-py/pyproject.toml")

    # Read versions
    with open(core_toml) as f:
        core_ver = tomlkit.load(f)["package"]["version"]
    with open(py_toml) as f:
        py_ver = tomlkit.load(f)["package"]["version"]
    with open(pyproject_toml) as f:
        pyproject_ver = tomlkit.load(f)["project"].get("version", None)

    print(f"kiru-core version: {core_ver}")
    print(f"kiru-py version: {py_ver}")
    print(f"pyproject.toml version: {pyproject_ver or 'dynamic'}")

    # Check consistency
    if pyproject_ver is None:  # Dynamic version, assume kiru-py/Cargo.toml
        if core_ver != py_ver:
            print(
                f"Error: Versions mismatch: kiru-core ({core_ver}) != kiru-py ({py_ver})"
            )
            sys.exit(1)
    else:  # Static version in pyproject.toml
        if not (core_ver == py_ver == pyproject_ver):
            print(
                f"Error: Versions mismatch: kiru-core ({core_ver}), kiru-py ({py_ver}), pyproject ({pyproject_ver})"
            )
            sys.exit(1)

    print("All versions match.")


if __name__ == "__main__":
    main()
