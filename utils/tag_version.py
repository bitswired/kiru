# utils/tag_version.py
# /// script
# dependencies = [
#   "tomlkit",
# ]
# ///
import argparse
import subprocess
import sys
from pathlib import Path

import tomlkit


def get_current_version() -> str:
    """Get version from kiru-py/Cargo.toml"""
    py_toml = Path("kiru-py/Cargo.toml")
    with open(py_toml) as f:
        return tomlkit.load(f)["package"]["version"]


def tag_exists(tag: str) -> bool:
    """Check if a git tag exists"""
    try:
        subprocess.run(
            ["git", "rev-parse", tag],
            capture_output=True,
            check=True,
        )
        return True
    except subprocess.CalledProcessError:
        return False


def create_tag(tag: str, force: bool = False):
    """Create a git tag"""
    cmd = ["git", "tag", tag, "-m", f"Release {tag}"]
    if force:
        cmd.insert(2, "-f")  # Insert --force flag

    try:
        subprocess.run(cmd, check=True)
        print(f"✓ Created tag: {tag}")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to create tag: {e}")
        sys.exit(1)


def push_tag(tag: str, force: bool = False):
    """Push a git tag to remote"""
    cmd = ["git", "push", "origin", tag]
    if force:
        cmd.append("--force")

    try:
        subprocess.run(cmd, check=True)
        print(f"✓ Pushed tag: {tag}")
    except subprocess.CalledProcessError as e:
        print(f"✗ Failed to push tag: {e}")
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Tag the current commit with the current version"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Force move the tag if it already exists",
    )
    parser.add_argument(
        "--push",
        action="store_true",
        help="Push the tag to remote after creating it",
    )
    args = parser.parse_args()

    # Get current version
    version = get_current_version()
    tag = f"v{version}"

    print(f"Current version: {version}")
    print(f"Tag to create: {tag}")

    # Check if tag exists
    if tag_exists(tag):
        if args.force:
            print(f"⚠ Tag {tag} already exists, forcing move...")
            create_tag(tag, force=True)
            if args.push:
                push_tag(tag, force=True)
        else:
            print(f"✗ Error: Tag {tag} already exists")
            print("  Use --force to move the tag to the current commit")
            sys.exit(1)
    else:
        create_tag(tag, force=False)
        if args.push:
            push_tag(tag, force=False)

    print(f"\n✓ Successfully tagged commit as {tag}")
    if args.push:
        print("✓ Tag pushed to remote")
    else:
        print(f"  Run 'git push origin {tag}' to push to remote")


if __name__ == "__main__":
    main()
