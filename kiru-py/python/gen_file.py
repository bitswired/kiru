import argparse
import random


def generate_random_utf8_file(size_mb: float, filename: str) -> str:
    """Generate a file with random UTF-8 content."""
    ranges = [
        (0x0020, 0x007E),  # ASCII printable
        (0x00A0, 0x00FF),  # Latin-1 supplement
        (0x0100, 0x017F),  # Latin extended A
        (0x0370, 0x03FF),  # Greek
        (0x0400, 0x04FF),  # Cyrillic
        (0x4E00, 0x4F9F),  # CJK ideographs (sample)
        (0x1F600, 0x1F64F),  # Emoticons
    ]

    target_bytes = int(size_mb * 1024 * 1024)

    print(f"Generating {filename} ({size_mb} MB)...")

    chars = []
    current_bytes = 0

    while current_bytes < target_bytes:
        start, end = random.choice(ranges)
        char_code = random.randint(start, end)

        try:
            char = chr(char_code)
            char_bytes = len(char.encode("utf-8"))

            if current_bytes + char_bytes <= target_bytes:
                chars.append(char)
                current_bytes += char_bytes
            else:
                break
        except (ValueError, UnicodeEncodeError):
            continue

    content = "".join(chars)

    with open(filename, "w", encoding="utf-8") as f:
        f.write(content)

    actual_size = len(content.encode("utf-8"))
    print(f"✓ Created {filename} ({actual_size / (1024 * 1024):.2f} MB)")
    return filename


def main():
    parser = argparse.ArgumentParser(description="Generate random UTF-8 test files")
    parser.add_argument(
        "size_mb",
        type=float,
        help="File size in megabytes (e.g., 100 for 100MB)",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=str,
        default=None,
        help="Output filename (default: <size_mb>mb.txt)",
    )

    args = parser.parse_args()

    filename = args.output or f"{args.size_mb}mb.txt"
    generate_random_utf8_file(args.size_mb, filename)


if __name__ == "__main__":
    main()
