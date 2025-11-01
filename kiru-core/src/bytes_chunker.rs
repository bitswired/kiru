use crate::chunker::{Chunker, ChunkingError, StringBuffer};

struct BytesChunkIndices {
    start: usize,
    end: usize,
    new_position: usize,
}

#[derive(Clone)]
pub struct BytesChunker {
    chunk_size: usize,
    overlap: usize,
}

impl BytesChunker {
    pub fn new(chunk_size: usize, overlap: usize) -> Result<Self, ChunkingError> {
        if overlap >= chunk_size {
            return Err(ChunkingError::InvalidArguments {
                chunk_size,
                overlap,
            });
        }

        Ok(Self {
            chunk_size,
            overlap,
        })
    }

    fn next_chunk_indices(
        &self,
        buffer: &str,
        current_position: usize,
    ) -> Option<BytesChunkIndices> {
        let buffer_len = buffer.len();

        // Done
        if current_position >= buffer_len {
            return None;
        }

        let start = current_position;

        // Start MUST be at char boundary
        assert!(
            buffer.is_char_boundary(start),
            "Bug: start position {} is not at char boundary",
            start
        );

        // Target end position (in bytes)
        let target_end = (start + self.chunk_size).min(buffer_len);

        // Adjust end backwards to char boundary
        let end = if target_end == buffer_len {
            buffer_len // End of string is always valid
        } else if buffer.is_char_boundary(target_end) {
            target_end // Lucky - already at boundary
        } else {
            // Search backwards (max 3 bytes for UTF-8)
            (target_end.saturating_sub(3)..target_end)
                .rev()
                .find(|&i| buffer.is_char_boundary(i))
                .expect("Bug: no char boundary found")
        };

        // If we've reached the end of text, we're done after this chunk
        if end >= buffer_len {
            return Some(BytesChunkIndices {
                start,
                end,
                new_position: buffer_len,
            });
        }

        // Calculate next position
        let actual_chunk_len = end - start;
        let step = actual_chunk_len.saturating_sub(self.overlap);

        let target_next_pos = start + step;

        // Adjust next position forward to char boundary
        let next_pos = if buffer.is_char_boundary(target_next_pos) {
            target_next_pos
        } else {
            // Search backward (max 3 bytes) to ensure we get AT LEAST the requested overlap
            (target_next_pos.saturating_sub(3)..=target_next_pos)
                .rev()
                .find(|&i| buffer.is_char_boundary(i))
                .expect("Bug: no char boundary found")
        };

        Some(BytesChunkIndices {
            start,
            end,
            new_position: next_pos,
        })
    }
}

impl Chunker for BytesChunker {
    fn chunk_string(self, input: String) -> impl Iterator<Item = String> {
        let mut current_position = 0;

        std::iter::from_fn(move || {
            let next = self.next_chunk_indices(&input, current_position)?;
            current_position = next.new_position;
            Some(input[next.start..next.end].to_string())
        })
    }

    fn chunk_stream(self, input: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
        let mut string_buffer = StringBuffer::new(input, self.chunk_size * 5);

        std::iter::from_fn(move || loop {
            let buffer = string_buffer.buffer();
            let next = self.next_chunk_indices(buffer, string_buffer.position);

            match next {
                // if the stream is done and no more chunks can be made, return None
                None if string_buffer.done => return None,

                // if no chunk can be made but the stream is not done, fill more data and try again
                None if !string_buffer.done => {
                    string_buffer.fill();
                    continue;
                }

                None => unreachable!(), // handled above

                // if the chunk end reaches the buffer end but the stream is not done, fill more data and try again
                Some(BytesChunkIndices { end, .. })
                    if !string_buffer.done && end == buffer.len() =>
                {
                    string_buffer.fill();
                    continue;
                }

                // otherwise, return the chunk
                Some(ref n) => {
                    let chunk = buffer[n.start..n.end].to_string();
                    string_buffer.set_position(n.new_position);
                    return Some(chunk);
                }
            };
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_chunker_stream() {
        // let reader = FileUtf8BlockReader::new(FILE_PATH, 1024 * 8).unwrap();

        let reader = vec!["01234".to_string(), "56789".to_string()].into_iter();
        let overlap = 2;
        let chunk_size = 6;

        let chunker = BytesChunker::new(chunk_size, overlap).unwrap();
        let mut chunked_iter = chunker.chunk_stream(reader);

        // for loop on the chunked iter
        for chunk in chunked_iter.by_ref() {
            println!("Chunk length: {}", chunk.len());
            println!("Chunk content: {}", &chunk);
        }
    }
}
