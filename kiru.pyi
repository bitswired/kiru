"""Kiru text chunking library."""

from typing import Iterator, List

__version__: str

class BytesChunkerBuilder:
    """Builder for byte-based chunking."""

    def from_text(self, text: str) -> Chunker:
        """Chunk text from a string."""
        ...

    def from_file(self, path: str) -> Chunker:
        """Chunk text from a file."""
        ...

class CharactersChunkerBuilder:
    """Builder for character-based chunking."""

    def from_text(self, text: str) -> Chunker:
        """Chunk text from a string."""
        ...

    def from_file(self, path: str) -> Chunker:
        """Chunk text from a file."""
        ...

class Chunker:
    """Text chunker with fluent API."""

    @staticmethod
    def by_bytes(chunk_size: int, overlap: int) -> BytesChunkerBuilder:
        """
        Create a byte-based chunker.

        Args:
            chunk_size: Size of each chunk in bytes.
            overlap: Number of overlapping bytes between chunks.

        Returns:
            A BytesChunkerBuilder instance.

        Raises:
            ValueError: If overlap >= chunk_size.
        """
        ...

    @staticmethod
    def by_characters(chunk_size: int, overlap: int) -> CharactersChunkerBuilder:
        """
        Create a character-based chunker.

        Args:
            chunk_size: Size of each chunk in characters.
            overlap: Number of overlapping characters between chunks.

        Returns:
            A CharactersChunkerBuilder instance.

        Raises:
            ValueError: If overlap >= chunk_size.
        """
        ...

    def all(self) -> List[str]:
        """Collect all chunks into a list."""
        ...

    def __iter__(self) -> Iterator[str]:
        """Return an iterator over the chunks."""
        ...

    def __next__(self) -> str:
        """Return the next chunk."""
        ...

__all__ = ["Chunker", "BytesChunkerBuilder", "CharactersChunkerBuilder"]
