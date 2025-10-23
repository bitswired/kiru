# justfile - Run commands from workspace root

# Default recipe (shows help)
default:
    @just --list

check-version:
    uv run utils/check_version_sync.py && uv run utils/check_version_newer.py

bump-version version:
    uv run utils/bump_version.py {{version}}

tag-version *args:
    uv run utils/tag_version.py {{args}}
