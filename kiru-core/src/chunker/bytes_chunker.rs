use std::{io, path};

use super::stream::*;
use super::ChunkingError;

fn build_char_positions(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(text.len()))
        .collect()
}

fn chunk_indices(
    text: &str,
    chunk_size: usize,
    overlap: usize,
    current_position: &mut usize,
) -> Option<(usize, usize)> {
    let text_len = text.len();

    // Done
    if *current_position >= text_len {
        return None;
    }

    let start = *current_position;

    // Start MUST be at char boundary
    assert!(
        text.is_char_boundary(start),
        "Bug: start position {} is not at char boundary",
        start
    );

    // Target end position (in bytes)
    let target_end = (start + chunk_size).min(text_len);

    // Adjust end backwards to char boundary
    let end = if target_end == text_len {
        text_len // End of string is always valid
    } else if text.is_char_boundary(target_end) {
        target_end // Lucky - already at boundary
    } else {
        // Search backwards (max 3 bytes for UTF-8)
        (target_end.saturating_sub(3)..target_end)
            .rev()
            .find(|&i| text.is_char_boundary(i))
            .expect("Bug: no char boundary found")
    };

    // If we've reached the end of text, we're done after this chunk
    if end >= text_len {
        *current_position = text_len;
        return Some((start, end));
    }

    // Calculate next position
    let actual_chunk_len = end - start;
    let step = actual_chunk_len.saturating_sub(overlap);

    // Ensure we make progress (should never happen with reasonable parameters)
    assert!(
        step > 0,
        "No progress: chunk_len={}, overlap={}, chunk_size={}. Need larger chunk_size vs overlap.",
        actual_chunk_len,
        overlap,
        chunk_size
    );

    let next_pos = start + step;

    // Adjust next position forward to char boundary
    *current_position = if text.is_char_boundary(next_pos) {
        next_pos
    } else {
        // Search backward (max 3 bytes) to ensure we get AT LEAST the requested overlap
        (next_pos.saturating_sub(3)..=next_pos)
            .rev()
            .find(|&i| text.is_char_boundary(i))
            .expect("Bug: no char boundary found")
    };

    Some((start, end))
}

pub fn chunk_string_by_bytes(
    text: String,
    chunk_size: usize,
    overlap: usize,
) -> Result<impl Iterator<Item = String> + Send + Sync, ChunkingError> {
    if overlap >= chunk_size {
        return Err(ChunkingError::InvalidArguments {
            chunk_size,
            overlap,
        });
    }

    let mut current_position = 0;

    let iterator = std::iter::from_fn(move || {
        let (start, end) = chunk_indices(&text, chunk_size, overlap, &mut current_position)?;
        Some(text[start..end].to_string())
    });

    Ok(iterator)
}

pub fn chunk_file_by_bytes(
    path: String,
    chunk_size: usize,
    overlap: usize,
) -> Result<impl Iterator<Item = String> + Send + Sync, ChunkingError> {
    if overlap >= chunk_size {
        return Err(ChunkingError::InvalidArguments {
            chunk_size,
            overlap,
        });
    }

    let mut reader = FileUtf8BlockReader::new(&path, 1024 * 8)?;
    let mut buffer = String::new();
    let mut position = 0;
    let mut file_done = false;

    let iterator = std::iter::from_fn(move || {
        loop {
            // Ensure we have enough data in the buffer for a full chunk
            // Keep reading until we have chunk_size * 5 bytes OR reach EOF
            while !file_done && buffer.len() - position < chunk_size * 5 {
                match reader.next() {
                    Some(block) => {
                        buffer.push_str(&block);
                    }
                    None => {
                        file_done = true;
                        break;
                    }
                }
            }

            // Try to get a chunk from current buffer
            if let Some((start, end)) = chunk_indices(&buffer, chunk_size, overlap, &mut position) {
                return Some(buffer[start..end].to_string());
            }

            // If we can't get a chunk and file is done, we're done
            if file_done {
                return None;
            }

            // Compact the buffer if needed
            if position > chunk_size * 5 {
                let keep_from = position.saturating_sub(chunk_size * 2);

                let boundary = (0..=keep_from.min(buffer.len()))
                    .rev()
                    .find(|&i| buffer.is_char_boundary(i))
                    .unwrap_or(0);

                buffer.drain(..boundary);
                position = position.saturating_sub(boundary);
            }
        }
    });

    Ok(iterator)
}
