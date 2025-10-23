# utils/tag_version.py
# /// script
# dependencies = [
#   "tomlkit",
# ]
# ///
from pathlib import Path

import tomlkit


def get_current_version() -> str:
    """Get version from kiru-py/Cargo.toml"""
    py_toml = Path("kiru-py/Cargo.toml")
    with open(py_toml) as f:
        return tomlkit.load(f)["package"]["version"]


if __name__ == "__main__":
    print(get_current_version())
