import argparse
import sys
import urllib.request
from typing import List, Tuple


def fetch_gutenberg_book(book_id: int) -> str:
    """Fetch a book from Project Gutenberg."""
    url = f"https://www.gutenberg.org/cache/epub/{book_id}/pg{book_id}.txt"
    try:
        print(f"Fetching book {book_id}...", end=" ", flush=True)
        with urllib.request.urlopen(url, timeout=30) as response:
            content = response.read().decode("utf-8", errors="ignore")
        print("✓")
        return content
    except Exception as e:
        print(f"✗ ({e})")
        return ""


def get_popular_books() -> List[Tuple[int, str]]:
    """Return a list of popular Project Gutenberg books."""
    return [
        (2600, "War and Peace"),
        (1342, "Pride and Prejudice"),
        (84, "Frankenstein"),
        (1661, "Sherlock Holmes"),
        (11, "Alice in Wonderland"),
        (1952, "The Yellow Wallpaper"),
        (2701, "Moby Dick"),
        (98, "A Tale of Two Cities"),
        (1260, "Jane Eyre"),
        (16, "Peter Pan"),
        (345, "Dracula"),
        (174, "The Picture of Dorian Gray"),
        (1080, "A Modest Proposal"),
        (135, "Les Misérables"),
        (4300, "Ulysses"),
        (1400, "Great Expectations"),
        (844, "The Importance of Being Earnest"),
        (1232, "The Prince"),
        (36, "The War of the Worlds"),
        (1727, "The Odyssey"),
    ]


def generate_realistic_file(size_mb: float, filename: str) -> str:
    """Generate a file with realistic text content from Project Gutenberg."""

    target_bytes = int(size_mb * 1024 * 1024)
    print(f"Generating {filename} ({size_mb} MB of realistic text)...")

    books = get_popular_books()
    accumulated_text = []
    current_bytes = 0
    book_index = 0

    while current_bytes < target_bytes:
        # Fetch next book
        if book_index >= len(books):
            # If we've used all books, cycle through again
            book_index = 0

        book_id, book_name = books[book_index]
        content = fetch_gutenberg_book(book_id)

        if not content:
            book_index += 1
            continue

        # Remove Project Gutenberg header/footer
        content = clean_gutenberg_text(content)

        # Calculate how much we need
        remaining_bytes = target_bytes - current_bytes
        content_bytes = content.encode("utf-8")

        if len(content_bytes) <= remaining_bytes:
            # Use entire book
            accumulated_text.append(content)
            current_bytes += len(content_bytes)
            print(
                f"  Added {book_name}: {len(content_bytes) / 1024:.1f} KB (total: {current_bytes / (1024 * 1024):.2f} MB), remaining: {(target_bytes - current_bytes) / (1024 * 1024):.2f} MB"
            )
        else:
            # Use partial book to reach exact size
            # Find the exact character position that gives us the target size
            truncated = truncate_to_byte_size(content, remaining_bytes)
            accumulated_text.append(truncated)
            current_bytes += len(truncated.encode("utf-8"))
            print(
                f"  Added {book_name} (partial): {len(truncated.encode('utf-8')) / 1024:.1f} KB (total: {current_bytes / (1024 * 1024):.2f} MB)"
            )
            break

        book_index += 1

    # Combine all text
    final_text = "\n\n".join(accumulated_text)

    # Write to file
    with open(filename, "w", encoding="utf-8") as f:
        f.write(final_text)

    # Verify size
    actual_size = len(final_text.encode("utf-8"))
    print(f"✓ Created {filename} ({actual_size / (1024 * 1024):.6f} MB)")
    print(f"  Target: {target_bytes:,} bytes")
    print(f"  Actual: {actual_size:,} bytes")
    print(f"  Diff: {actual_size - target_bytes:,} bytes")

    return filename


def clean_gutenberg_text(content: str) -> str:
    """Remove Project Gutenberg header and footer."""
    # Find start of actual content
    start_markers = [
        "*** START OF THE PROJECT GUTENBERG EBOOK",
        "*** START OF THIS PROJECT GUTENBERG EBOOK",
        "*END*THE SMALL PRINT",
    ]

    start_pos = 0
    for marker in start_markers:
        pos = content.find(marker)
        if pos != -1:
            # Find the end of this line
            start_pos = content.find("\n", pos) + 1
            break

    # Find end of actual content
    end_markers = [
        "*** END OF THE PROJECT GUTENBERG EBOOK",
        "*** END OF THIS PROJECT GUTENBERG EBOOK",
        "End of the Project Gutenberg EBook",
        "End of Project Gutenberg's",
    ]

    end_pos = len(content)
    for marker in end_markers:
        pos = content.find(marker)
        if pos != -1:
            end_pos = pos
            break

    return content[start_pos:end_pos].strip()


def truncate_to_byte_size(text: str, target_bytes: int) -> str:
    """Truncate text to exact byte size, respecting UTF-8 boundaries."""
    if not text:
        return ""

    # Binary search for the right character position
    left, right = 0, len(text)
    best_pos = 0

    while left <= right:
        mid = (left + right) // 2
        current_bytes = len(text[:mid].encode("utf-8"))

        if current_bytes == target_bytes:
            return text[:mid]
        elif current_bytes < target_bytes:
            best_pos = mid
            left = mid + 1
        else:
            right = mid - 1

    return text[:best_pos]


def main():
    parser = argparse.ArgumentParser(
        description="Generate realistic text files from Project Gutenberg",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python generate_realistic.py 10           # Generate 10MB file
  python generate_realistic.py 100 -o data.txt  # Generate 100MB file named data.txt
  
This script fetches real books from Project Gutenberg and combines them
to create a file of exactly the specified size.
        """,
    )
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
        help="Output filename (default: realistic_<size_mb>mb.txt)",
    )

    args = parser.parse_args()

    if args.size_mb <= 0:
        print("Error: size_mb must be positive")
        sys.exit(1)

    filename = args.output or f"realistic_{args.size_mb}mb.txt"

    try:
        generate_realistic_file(args.size_mb, filename)
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\nError: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
