mod common;

use std::fs::File;

use common::helpers::{assert_all_valid_utf8, create_temp_file};
use kiru::{BytesChunker, Chunker, ChunkingError, Source, StreamType};
use proptest::prelude::*;

// ============================================================================
// BYTE-SPECIFIC HELPERS
// ============================================================================

/// Find overlap between chunks in bytes
fn find_byte_overlap(prev: &str, next: &str, target_overlap: usize) -> usize {
    let max_check = target_overlap.min(next.len()).min(prev.len());

    for len in (1..=max_check).rev() {
        if !next.is_char_boundary(len) {
            continue;
        }

        let prev_start = prev.len().saturating_sub(len);
        if !prev.is_char_boundary(prev_start) {
            continue;
        }

        if prev[prev_start..] == next[..len] {
            return len;
        }
    }

    0
}

/// Reconstruct text from byte-chunked chunks
fn reconstruct_from_byte_chunks(chunks: &[String], overlap: usize, original: &str) -> String {
    let mut reconstructed = String::new();
    let mut original_pos = 0;

    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            reconstructed.push_str(chunk);
            original_pos = chunk.len();
        } else {
            // Calculate where this chunk should start in the original
            let expected_start = original_pos.saturating_sub(overlap);

            // Find the actual char boundary at or before expected_start
            let actual_start = (0..=expected_start)
                .rev()
                .find(|&pos| original.is_char_boundary(pos))
                .unwrap_or(0);

            // The overlap is from actual_start to original_pos
            let actual_overlap = original_pos - actual_start;

            // Skip the overlap in current chunk
            let skip_boundary = (0..=chunk.len())
                .find(|&pos| chunk.is_char_boundary(pos) && pos >= actual_overlap)
                .unwrap_or(chunk.len());

            reconstructed.push_str(&chunk[skip_boundary..]);
            original_pos += chunk.len() - skip_boundary;
        }
    }

    reconstructed
}

/// Validate byte chunk sizes
fn assert_byte_chunk_sizes(chunks: &[String], chunk_size: usize, tolerance: usize) {
    for (i, chunk) in chunks.iter().enumerate() {
        let size = chunk.len(); // bytes

        if i < chunks.len() - 1 {
            assert!(
                size >= chunk_size.saturating_sub(tolerance),
                "Chunk {} too small: {} bytes < {} bytes (tolerance: {} bytes)",
                i,
                size,
                chunk_size.saturating_sub(tolerance),
                tolerance
            );
            assert!(
                size <= chunk_size + tolerance,
                "Chunk {} too large: {} bytes > {} bytes (tolerance: {} bytes)",
                i,
                size,
                chunk_size + tolerance,
                tolerance
            );
        }
    }
}

/// Validate byte overlaps
fn assert_byte_overlaps(chunks: &[String], overlap: usize, min_tolerance: usize) {
    for i in 0..chunks.len().saturating_sub(1) {
        let current = &chunks[i];
        let next = &chunks[i + 1];

        let actual_overlap = find_byte_overlap(current, next, overlap + min_tolerance);
        let min_overlap = overlap.saturating_sub(min_tolerance);

        assert!(
            actual_overlap >= min_overlap,
            "Overlap between chunks {} and {} too small: {} bytes < {} bytes (target: {} bytes)",
            i,
            i + 1,
            actual_overlap,
            min_overlap,
            overlap
        );
    }
}

/// Main validation for byte chunker
fn assert_byte_chunks_valid(
    chunks: &[String],
    original_text: &str,
    chunk_size: usize,
    overlap: usize,
    tolerance: usize,
) {
    // 1. Valid UTF-8
    assert_all_valid_utf8(chunks);

    // 2. Chunk sizes in bounds
    assert_byte_chunk_sizes(chunks, chunk_size, tolerance);

    // 3. Overlaps correct
    if chunks.len() > 1 {
        assert_byte_overlaps(chunks, overlap, tolerance);
    }

    // 4. Reconstruction
    let reconstructed = reconstruct_from_byte_chunks(chunks, overlap, &original_text);
    assert_eq!(
        reconstructed,
        original_text,
        "Reconstruction failed! Expected length: {}, got: {}",
        original_text.len(),
        reconstructed.len()
    );
}

// ============================================================================
// PROPERTY-BASED TESTS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn string_chunks_complete_validation(
        text in "\\PC{100,5000}",
        chunk_size in 50usize..500,
        overlap in 10usize..50,
    ) {
        prop_assume!(overlap < chunk_size - 10);
        prop_assume!(!text.is_empty());

        let mut chunker = BytesChunker::new(chunk_size, overlap)?;
        let chunks = chunker.chunk_string(text.clone()).collect::<Vec<_>>();

        if !chunks.is_empty() {
            assert_byte_chunks_valid(&chunks, &text, chunk_size, overlap, 7);
        }
    }

    #[test]
    fn file_chunks_complete_validation(
        text in "\\PC{100,5000}",
        chunk_size in 50usize..500,
        overlap in 10usize..50,
    ) {
        prop_assume!(overlap < chunk_size - 10);
        prop_assume!(!text.is_empty());

        let (_dir, path) = create_temp_file(&text);
        let mut chunker = BytesChunker::new(chunk_size, overlap)?;
        let stream = StreamType::from_source(&Source::File(path))?;
        let chunks = chunker.chunk_stream(stream).collect::<Vec<_>>();

        if !chunks.is_empty() {
            assert_byte_chunks_valid(&chunks, &text, chunk_size, overlap, 7);
        }
    }

    #[test]
    fn file_large_content(
        pattern in "\\PC{100,500}",
        repeats in 10usize..100,
        chunk_size in 100usize..1000,
        overlap in 10usize..100,
    ) {
        prop_assume!(overlap < chunk_size - 10);

        let text = pattern.repeat(repeats);
        let (_dir, path) = create_temp_file(&text);

        let mut chunker = BytesChunker::new(chunk_size, overlap)?;
        let stream = StreamType::from_source(&Source::File(path))?;
        let chunks = chunker.chunk_stream(stream).collect::<Vec<_>>();

        assert_byte_chunks_valid(&chunks, &text, chunk_size, overlap, 10);
    }

    #[test]
    fn file_with_multibyte_chars(
        emoji_count in 10usize..100,
        chunk_size in 50usize..200,
        overlap in 10usize..100,
    ) {
        prop_assume!(overlap < chunk_size - 10);

        let emojis = ["🎉", "🎊", "🎈", "🎁", "🎂", "❤️", "🌟", "✨"];
        let text: String = (0..emoji_count)
            .map(|i| emojis[i % emojis.len()])
            .collect();

        let (_dir, path) = create_temp_file(&text);

        let mut chunker = BytesChunker::new(chunk_size, overlap)?;
        let stream = StreamType::from_source(&Source::File(path))?;
        let chunks = chunker.chunk_stream(stream).collect::<Vec<_>>();

        assert_byte_chunks_valid(&chunks, &text, chunk_size, overlap, 10);
    }
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn edge_case_empty_string() {
    let mut chunker = BytesChunker::new(100, 10).unwrap();
    let chunks = chunker.chunk_string("".to_string()).collect::<Vec<_>>();

    assert!(chunks.is_empty());
}

#[test]
fn edge_case_empty_file() {
    let (_dir, path) = create_temp_file("");

    let mut chunker = BytesChunker::new(100, 10).unwrap();
    let stream = StreamType::from_source(&Source::File(path)).unwrap();
    let chunks: Vec<_> = chunker.chunk_stream(stream).collect();

    assert!(chunks.is_empty());
}

#[test]
fn edge_case_string_smaller_than_chunk() {
    let mut chunker = BytesChunker::new(100, 10).unwrap();
    let chunks: Vec<_> = chunker.chunk_string("Hi".to_string()).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "Hi");
}

#[test]
fn edge_case_file_smaller_than_chunk() {
    let (_dir, path) = create_temp_file("Hi");

    let mut chunker = BytesChunker::new(100, 10).unwrap();
    let stream = StreamType::from_source(&Source::File(path)).unwrap();
    let chunks: Vec<_> = chunker.chunk_stream(stream).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "Hi");
}

#[test]
fn edge_case_string_exactly_chunk_size() {
    let text = "12345";

    let mut chunker = BytesChunker::new(5, 0).unwrap();
    let chunks: Vec<_> = chunker.chunk_string(text.to_string()).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
}

#[test]
fn edge_case_file_exactly_chunk_size() {
    let text = "12345";
    let (_dir, path) = create_temp_file(text);

    let mut chunker = BytesChunker::new(5, 0).unwrap();
    let stream = StreamType::from_source(&Source::File(path)).unwrap();
    let chunks: Vec<_> = chunker.chunk_stream(stream).collect();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn error_overlap_equals_chunk_size() {
    let result = BytesChunker::new(5, 5);

    assert!(matches!(
        result,
        Err(ChunkingError::InvalidArguments {
            chunk_size: 5,
            overlap: 5
        })
    ));
}

#[test]
fn error_overlap_greater_than_chunk_size() {
    let result = BytesChunker::new(5, 10);
    assert!(matches!(
        result,
        Err(ChunkingError::InvalidArguments {
            chunk_size: 5,
            overlap: 10
        })
    ));
}

#[test]
fn error_file_not_found() {
    let result = StreamType::from_source(&Source::File(
        "/path/that/definitely/does/not/exist/file.txt".to_string(),
    ));
    assert!(matches!(result, Err(ChunkingError::Io(_))));
}
