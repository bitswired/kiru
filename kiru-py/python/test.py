import pytest
from hypothesis import assume, given, settings
from hypothesis import strategies as st
from kiru import Chunker
from langchain.text_splitter import CharacterTextSplitter


def get_langchain_chunks(text: str, chunk_size: int, overlap: int) -> list[str]:
    """Get chunks from LangChain with settings that match Kiru's behavior."""
    splitter = CharacterTextSplitter(
        chunk_size=chunk_size,
        chunk_overlap=overlap,
        separator="",
        is_separator_regex=False,
        keep_separator=True,
        strip_whitespace=False,
    )
    return splitter.split_text(text)


def get_kiru_chunks(text: str, chunk_size: int, overlap: int) -> list[str]:
    """Get chunks from Kiru."""
    chunker = Chunker.by_characters(chunk_size=chunk_size, overlap=overlap)
    return chunker.on_string(text).all()


def chunks_are_identical(
    kiru_chunks: list[str], langchain_chunks: list[str]
) -> tuple[bool, str]:
    """Compare chunks and return (is_identical, error_message)."""
    if len(kiru_chunks) != len(langchain_chunks):
        return (
            False,
            f"Different number of chunks: Kiru={len(kiru_chunks)}, LangChain={len(langchain_chunks)}",
        )

    for i, (kiru_chunk, langchain_chunk) in enumerate(
        zip(kiru_chunks, langchain_chunks)
    ):
        if kiru_chunk != langchain_chunk:
            # Find the first difference
            for j, (k_char, l_char) in enumerate(zip(kiru_chunk, langchain_chunk)):
                if k_char != l_char:
                    return False, (
                        f"Chunk {i} differs at position {j}: "
                        f"Kiru='{k_char}' ({ord(k_char)}), LangChain='{l_char}' ({ord(l_char)})\n"
                        f"Kiru chunk: {repr(kiru_chunk)}\n"
                        f"LangChain chunk: {repr(langchain_chunk)}"
                    )

            # If we get here, one chunk is a prefix of the other
            if len(kiru_chunk) != len(langchain_chunk):
                longer = (
                    kiru_chunk
                    if len(kiru_chunk) > len(langchain_chunk)
                    else langchain_chunk
                )
                shorter_len = min(len(kiru_chunk), len(langchain_chunk))
                return False, (
                    f"Chunk {i} has different length: Kiru={len(kiru_chunk)}, LangChain={len(langchain_chunk)}\n"
                    f"Extra characters: {repr(longer[shorter_len:])}"
                )

    return True, ""


class TestKiruLangChainComparison:
    """Test suite comparing Kiru and LangChain chunking behavior."""

    def test_simple_case(self):
        """Test a simple known case."""
        text = "Hello world! This is a test string. " * 5
        chunk_size = 50
        overlap = 10

        kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
        langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

        is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
        assert is_identical, error

    def test_no_overlap(self):
        """Test chunking with no overlap."""
        text = "abcdefghijklmnopqrstuvwxyz" * 10
        chunk_size = 25
        overlap = 0

        kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
        langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

        is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
        assert is_identical, error

    def test_full_overlap(self):
        """Test chunking where overlap equals chunk_size - 1."""
        text = "Hello world! " * 10
        chunk_size = 20
        overlap = 19

        kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
        langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

        is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
        assert is_identical, error

    def test_whitespace_preservation(self):
        """Test that whitespace at chunk boundaries is preserved."""
        text = "word   word\t\tword\n\nword"
        chunk_size = 6
        overlap = 2

        kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
        langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

        is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
        assert is_identical, error

    @given(
        text=st.text(min_size=1, max_size=30_000),
        chunk_size=st.integers(min_value=1, max_value=3_000),
        overlap=st.integers(min_value=0, max_value=99),
    )
    @settings(max_examples=100, deadline=5000)
    def test_hypothesis_random_inputs(self, text: str, chunk_size: int, overlap: int):
        """Property-based test with random inputs."""
        # Ensure overlap is less than chunk_size
        assume(overlap < chunk_size)
        # Skip empty or very short texts that might cause edge cases
        assume(len(text) >= chunk_size)

        try:
            kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
            langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

            is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
            assert is_identical, (
                f"Chunks differ for text={repr(text[:50])}, chunk_size={chunk_size}, overlap={overlap}\n{error}"
            )

        except Exception as e:
            # If both implementations fail the same way, that's acceptable
            pytest.fail(
                f"Unexpected error with text={repr(text[:50])}, chunk_size={chunk_size}, overlap={overlap}: {e}"
            )

    @pytest.mark.parametrize(
        "chunk_size,overlap",
        [
            (10, 0),
            (10, 5),
            (10, 9),
            (50, 10),
            (100, 20),
            (5, 1),
            (1, 0),
        ],
    )
    def test_parametrized_sizes(self, chunk_size: int, overlap: int):
        """Test various chunk sizes and overlaps with known text."""
        text = "The quick brown fox jumps over the lazy dog. " * 20

        kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
        langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

        is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
        assert is_identical, error


#     def test_unicode_characters(self):
#         """Test with unicode characters."""
#         text = "Hello 世界! Café naïve résumé 🚀🎉 " * 10
#         chunk_size = 25
#         overlap = 5

#         kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
#         langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

#         is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
#         assert is_identical, error

#     def test_edge_case_single_character(self):
#         """Test with single character chunk size."""
#         text = "abcdefghijk"
#         chunk_size = 1
#         overlap = 0

#         kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
#         langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

#         is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
#         assert is_identical, error

#     def test_edge_case_text_shorter_than_chunk(self):
#         """Test when text is shorter than chunk size."""
#         text = "short"
#         chunk_size = 100
#         overlap = 10

#         kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
#         langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

#         is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
#         assert is_identical, error


# if __name__ == "__main__":
#     # Run a simple comparison for manual testing
#     print("Running simple comparison...")

#     text = "Hello world! This is a test string. " * 20
#     chunk_size = 100
#     overlap = 20

#     kiru_chunks = get_kiru_chunks(text, chunk_size, overlap)
#     langchain_chunks = get_langchain_chunks(text, chunk_size, overlap)

#     print(f"Kiru chunks: {len(kiru_chunks)}")
#     print(f"LangChain chunks: {len(langchain_chunks)}")

#     is_identical, error = chunks_are_identical(kiru_chunks, langchain_chunks)
#     if is_identical:
#         print("✓ Chunks are identical!")
#     else:
#         print(f"✗ Chunks differ: {error}")

#     print("\nRun 'pytest t.py' to run the full test suite")
