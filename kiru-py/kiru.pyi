"""Kiru text chunking library."""

from typing import Iterator, List, Optional

__version__: str

class Chunker:
    """A factory for creating chunkers with specific strategies (bytes or characters)."""

    @staticmethod
    def by_bytes(chunk_size: int, overlap: int) -> "ChunkerBuilder":
        """
        Create a byte-based chunker.

        Args:
            chunk_size: Size of each chunk in bytes.
            overlap: Number of overlapping bytes between chunks (must be less than chunk_size).

        Returns:
            ChunkerBuilder: A builder for chunking sources.

        Raises:
            ValueError: If chunk_size is 0 or overlap >= chunk_size.
        """
        ...

    @staticmethod
    def by_characters(chunk_size: int, overlap: int) -> "ChunkerBuilder":
        """
        Create a character-based chunker.

        Args:
            chunk_size: Size of each chunk in characters.
            overlap: Number of overlapping characters between chunks (must be less than chunk_size).

        Returns:
            ChunkerBuilder: A builder for chunking sources.

        Raises:
            ValueError: If chunk_size is 0 or overlap >= chunk_size.
        """
        ...

class ChunkerBuilder:
    """A builder for chunking various sources using a specified strategy."""

    def on_string(self, text: str) -> "ChunkerIterator":
        """
        Chunk a single string input.

        Args:
            text: The input text to chunk.

        Returns:
            ChunkerIterator: An iterator over the chunks.

        Raises:
            ValueError: If the input cannot be processed.
        """
        ...

    def on_file(self, path: str) -> "ChunkerIterator":
        """
        Chunk a single file from a local path.

        Args:
            path: The path to the file (e.g., "path/to/file.txt").

        Returns:
            ChunkerIterator: An iterator over the chunks.

        Raises:
            ValueError: If the file cannot be read (e.g., does not exist).
        """
        ...

    def on_http(self, url: str) -> "ChunkerIterator":
        """
        Chunk content from an HTTP/HTTPS URL.

        Args:
            url: The URL to fetch content from (e.g., "http://example.com/text").

        Returns:
            ChunkerIterator: An iterator over the chunks.

        Raises:
            ValueError: If the URL cannot be fetched or content cannot be processed.
        """
        ...

    def on_sources(self, source_strings: List[str]) -> "ChunkerIterator":
        """
        Chunk multiple sources specified as strings with prefixes.

        Supported prefixes:
        - `file://` for local files (e.g., "file://path/to/file.txt").
        - `http://` or `https://` for URLs (e.g., "http://example.com/text").
        - `text://` for raw text (e.g., "text://Hello world").
        - `glob://` for glob patterns (e.g., "glob://*.txt").
        - No prefix assumes a text string.

        Args:
            source_strings: A list of source strings with optional prefixes.

        Returns:
            ChunkerIterator: An iterator over the chunks from all sources.

        Raises:
            ValueError: If any source has an invalid prefix, uses an unsupported type (e.g., "sitemap://"),
                        or cannot be processed (e.g., file not found, invalid glob).
        """
        ...

    def on_sources_par(
        self, source_strings: List[str], channel_size: Optional[int] = None
    ) -> "ChunkerIterator":
        """
        Chunk multiple sources in parallel, specified as strings with prefixes.

        Supported prefixes:
        - `file://` for local files (e.g., "file://path/to/file.txt").
        - `http://` or `https://` for URLs (e.g., "http://example.com/text").
        - `text://` for raw text (e.g., "text://Hello world").
        - `glob://` for glob patterns (e.g., "glob://*.txt").
        - No prefix assumes a text string.

        Args:
            source_strings: A list of source strings with optional prefixes.
            channel_size: Number of chunks to buffer in the channel (default: 100).

        Returns:
            ChunkerIterator: An iterator over the chunks from all sources.

        Raises:
            ValueError: If any source has an invalid prefix, uses an unsupported type (e.g., "sitemap://"),
                        or cannot be processed (e.g., file not found, invalid glob).
        """
        ...

class ChunkerIterator:
    """An iterator over chunks produced from one or more sources."""

    def all(self) -> List[str]:
        """Collect all chunks into a list.

        Returns:
            A list of all chunks.
        """
        ...

    def __iter__(self) -> Iterator[str]:
        """Return an iterator over the chunks.

        Returns:
            The iterator itself.
        """
        ...

    def __next__(self) -> str:
        """Get the next chunk.

        Returns:
            The next chunk.

        Raises:
            StopIteration: If no more chunks are available.
        """
        ...

__all__ = ["Chunker", "ChunkerBuilder", "ChunkerIterator"]
