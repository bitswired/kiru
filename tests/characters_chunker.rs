mod common;

use common::helpers::{assert_all_valid_utf8, create_temp_file};
use kiru::{chunk_file_by_characters, chunk_string_by_characters, ChunkingError};
use proptest::prelude::*;

// ============================================================================
// CHARACTER-SPECIFIC HELPERS
// ============================================================================

/// Find overlap between chunks in characters
fn find_char_overlap(prev: &str, next: &str, target_overlap: usize) -> usize {
    let prev_chars: Vec<char> = prev.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();

    let max_check = target_overlap.min(prev_chars.len()).min(next_chars.len());

    for len in (1..=max_check).rev() {
        let prev_start = prev_chars.len().saturating_sub(len);
        if prev_chars[prev_start..] == next_chars[..len] {
            return len;
        }
    }

    0
}

/// Reconstruct text from character-chunked chunks
fn reconstruct_from_char_chunks(chunks: &[String], overlap: usize) -> String {
    let mut reconstructed = String::new();

    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            reconstructed.push_str(chunk);
        } else {
            let curr_chars: Vec<char> = chunk.chars().collect();
            // Convert character overlap to byte position
            let skip_bytes: usize = curr_chars.iter().take(overlap).map(|c| c.len_utf8()).sum();
            reconstructed.push_str(&chunk[skip_bytes..]);
        }
    }

    reconstructed
}

/// Validate character chunk sizes
fn assert_char_chunk_sizes(chunks: &[String], chunk_size: usize, tolerance: usize) {
    for (i, chunk) in chunks.iter().enumerate() {
        let size = chunk.chars().count(); // characters, not bytes!

        if i < chunks.len() - 1 {
            assert!(
                size >= chunk_size.saturating_sub(tolerance),
                "Chunk {} too small: {} chars < {} chars (tolerance: {} chars)",
                i,
                size,
                chunk_size.saturating_sub(tolerance),
                tolerance
            );
            assert!(
                size <= chunk_size + tolerance,
                "Chunk {} too large: {} chars > {} chars (tolerance: {} chars)",
                i,
                size,
                chunk_size + tolerance,
                tolerance
            );
        }
    }
}

/// Validate character overlaps
fn assert_char_overlaps(chunks: &[String], overlap: usize, min_tolerance: usize) {
    for i in 0..chunks.len().saturating_sub(1) {
        let current = &chunks[i];
        let next = &chunks[i + 1];

        let actual_overlap = find_char_overlap(current, next, overlap + min_tolerance);
        let min_overlap = overlap.saturating_sub(min_tolerance);

        assert!(
            actual_overlap >= min_overlap,
            "Overlap between chunks {} and {} too small: {} chars < {} chars (target: {} chars)",
            i,
            i + 1,
            actual_overlap,
            min_overlap,
            overlap
        );
    }
}

/// Main validation for character chunker
fn assert_char_chunks_valid(
    chunks: &[String],
    original_text: &str,
    chunk_size: usize,
    overlap: usize,
    tolerance: usize,
) {
    // 1. Valid UTF-8
    assert_all_valid_utf8(chunks);

    // 2. Chunk sizes in bounds (measured in characters)
    assert_char_chunk_sizes(chunks, chunk_size, tolerance);

    // 3. Overlaps correct (measured in characters)
    if chunks.len() > 1 {
        assert_char_overlaps(chunks, overlap, tolerance);
    }

    // 4. Reconstruction
    let reconstructed = reconstruct_from_char_chunks(chunks, overlap);
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

        let chunks: Vec<String> = chunk_string_by_characters(
            text.clone(),
            chunk_size,
            overlap
        )?.collect();

        if !chunks.is_empty() {
            assert_char_chunks_valid(&chunks, &text, chunk_size, overlap, 7);
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
        let chunks: Vec<String> = chunk_file_by_characters(
            path,
            chunk_size,
            overlap
        )?.collect();

        if !chunks.is_empty() {
            assert_char_chunks_valid(&chunks, &text, chunk_size, overlap, 7);
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
        let chunks: Vec<String> = chunk_file_by_characters(
            path,
            chunk_size,
            overlap
        )?.collect();

        assert_char_chunks_valid(&chunks, &text, chunk_size, overlap, 10);
    }

    #[test]
    fn file_with_multibyte_chars(
        emoji_count in 10usize..100,
        chunk_size in 50usize..200,
        overlap in 5usize..20,
    ) {
        prop_assume!(overlap < chunk_size - 10);


        let emojis = ["🎉", "🎊", "🎈", "🎁", "🎂", "❤️", "🌟", "✨"];
        let text: String = (0..emoji_count)
            .map(|i| emojis[i % emojis.len()])
            .collect();

        let (_dir, path) = create_temp_file(&text);
        let chunks: Vec<String> = chunk_file_by_characters(
            path,
            chunk_size,
            overlap
        )?.collect();
        println!("--------------------------------");
        println!("{:?}", text);
        println!("{:?}", chunks);
        println!("--------------------------------");

        assert_char_chunks_valid(&chunks, &text, chunk_size, overlap, 10);
    }
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn edge_case_empty_string() {
    let chunks: Vec<_> = chunk_string_by_characters("".to_string(), 100, 10)
        .unwrap()
        .collect();
    assert!(chunks.is_empty());
}

#[test]
fn edge_case_empty_file() {
    let (_dir, path) = create_temp_file("");
    let chunks: Vec<_> = chunk_file_by_characters(path, 100, 10).unwrap().collect();
    assert!(chunks.is_empty());
}

#[test]
fn edge_case_string_smaller_than_chunk() {
    let chunks: Vec<_> = chunk_string_by_characters("Hi".to_string(), 100, 10)
        .unwrap()
        .collect();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "Hi");
}

#[test]
fn edge_case_file_smaller_than_chunk() {
    let (_dir, path) = create_temp_file("Hi");
    let chunks: Vec<_> = chunk_file_by_characters(path, 100, 10).unwrap().collect();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "Hi");
}

#[test]
fn edge_case_string_exactly_chunk_size() {
    let text = "12345";
    let chunks: Vec<_> = chunk_string_by_characters(text.to_string(), 5, 0)
        .unwrap()
        .collect();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
}

#[test]
fn edge_case_file_exactly_chunk_size() {
    let text = "12345";
    let (_dir, path) = create_temp_file(text);
    let chunks: Vec<_> = chunk_file_by_characters(path, 5, 0).unwrap().collect();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
}

#[test]
fn edge_case_multibyte_chars_exactly() {
    // 5 emojis = 5 characters (but 20 bytes)
    let text = "🎉🎊🎈🎁🎂";
    let chunks: Vec<_> = chunk_string_by_characters(text.to_string(), 5, 0)
        .unwrap()
        .collect();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], text);
    assert_eq!(chunks[0].chars().count(), 5);
}

#[test]
fn edge_case_mixed_multibyte() {
    // Mix ASCII and multibyte
    let text = "abc🎉def🎊ghi";
    let chunks: Vec<_> = chunk_string_by_characters(text.to_string(), 6, 2)
        .unwrap()
        .collect();

    println!("{:?}", chunks);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].chars().count(), 6);
    assert_eq!(chunks[1].chars().count(), 6);
    assert_eq!(chunks[2].chars().count(), 3);
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn error_overlap_equals_chunk_size() {
    let result = chunk_string_by_characters("test".to_string(), 5, 5);
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
    let result = chunk_string_by_characters("test".to_string(), 5, 10);
    assert!(matches!(
        result,
        Err(ChunkingError::InvalidArguments {
            chunk_size: 5,
            overlap: 10
        })
    ));
}

#[test]
fn error_file_overlap_equals_chunk_size() {
    let (_dir, path) = create_temp_file("test content");
    let result = chunk_file_by_characters(path, 5, 5);
    assert!(matches!(
        result,
        Err(ChunkingError::InvalidArguments {
            chunk_size: 5,
            overlap: 5
        })
    ));
}

#[test]
fn error_file_overlap_greater_than_chunk_size() {
    let (_dir, path) = create_temp_file("test content");
    let result = chunk_file_by_characters(path, 5, 10);
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
    let result = chunk_file_by_characters(
        "/path/that/definitely/does/not/exist/file.txt".to_string(),
        100,
        10,
    );
    assert!(matches!(result, Err(ChunkingError::Io(_))));
}
