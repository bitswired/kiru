use std::collections::VecDeque;

use crate::chunker::{Chunker, ChunkingError, StringBuffer};

#[derive(Debug, Clone, Copy)]
struct CharPosition {
    start: usize,
    len: usize,
}

struct CharactersChunkIndices {
    start: usize,
    end: usize,
    new_byte_position: usize,
    new_char_position: usize,
}

#[derive(Clone)]
pub struct CharactersChunker {
    chunk_size: usize,
    overlap: usize,
    char_positions: VecDeque<CharPosition>,
    current_char_position: usize,
}

impl CharactersChunker {
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
            char_positions: VecDeque::new(),
            current_char_position: 0,
        })
    }
    fn build_char_positions(&mut self, text: &str, offset: usize) {
        let cp = text.char_indices().map(|(pos, ch)| CharPosition {
            start: pos + offset,
            len: ch.len_utf8(),
        });
        self.char_positions.extend(cp);
    }

    fn compact(&mut self, string_buffer: &mut StringBuffer<impl Iterator<Item = String>>) {
        // Compact char positions and string buffer if we've consumed more than half
        if self.current_char_position > self.char_positions.len() / 2 {
            let keep_from_chars = self.current_char_position;
            let keep_from_bytes = self.char_positions[keep_from_chars].start;
            self.char_positions.drain(0..self.current_char_position);
            string_buffer.compact_to(keep_from_bytes);
            self.current_char_position = 0;

            // shift all char positions back by keep_from_bytes
            for cp in self.char_positions.iter_mut() {
                if cp.start < keep_from_bytes {
                    println!("cp start {}, keep from bytes {}", cp.start, keep_from_bytes);
                    // print all values in char_positions
                }
                cp.start -= keep_from_bytes;
            }
        }
    }

    fn next_chunk_indices(&self, buffer: &str) -> Option<CharactersChunkIndices> {
        let buffer_len = buffer.len();
        let chars_len = self.char_positions.len();

        // Done
        if self.current_char_position >= chars_len {
            return None;
        }

        let start_idx = self.current_char_position;
        let end_idx = (start_idx + self.chunk_size).min(chars_len);
        let start_byte = self.char_positions[start_idx].start;

        let end_byte = if end_idx >= chars_len {
            buffer_len
        } else {
            let char_pos = self.char_positions[end_idx - 1];
            char_pos.start + char_pos.len
        };

        // If we've reached the end of text, we're done after this chunk
        if end_idx >= chars_len {
            return Some(CharactersChunkIndices {
                start: start_byte,
                end: end_byte,
                new_byte_position: buffer_len,
                new_char_position: chars_len,
            });
        }

        // Calculate next position
        let step = self.chunk_size.saturating_sub(self.overlap);

        // return Some((start_byte, end_byte));
        let next_char_position = start_idx + step;
        let next_byte_position = self.char_positions[next_char_position].start;

        Some(CharactersChunkIndices {
            start: start_byte,
            end: end_byte,
            new_byte_position: next_byte_position,
            new_char_position: next_char_position,
        })
    }
}

impl Chunker for CharactersChunker {
    fn chunk_string(mut self, input: String) -> impl Iterator<Item = String> {
        self.build_char_positions(&input, 0);

        std::iter::from_fn(move || {
            let next = self.next_chunk_indices(&input)?;
            self.current_char_position = next.new_char_position;
            Some(input[next.start..next.end].to_string())
        })
    }

    fn chunk_stream(mut self, input: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
        let mut string_buffer = StringBuffer::new(input, self.chunk_size * 5);

        std::iter::from_fn(move || loop {
            let buffer = string_buffer.buffer();
            let next = self.next_chunk_indices(buffer);

            match next {
                // if the stream is done and no more chunks can be made, return None
                None if string_buffer.done => return None,

                // if no chunk can be made but the stream is not done, fill more data and try again
                None if !string_buffer.done => {
                    let old_buffer_len = buffer.len();
                    string_buffer.fill_no_compact();
                    let new_buffer_len = string_buffer.buffer().len();
                    self.build_char_positions(
                        &string_buffer.buffer()[old_buffer_len..new_buffer_len],
                        old_buffer_len,
                    );
                    self.compact(&mut string_buffer);
                    continue;
                }

                None => unreachable!(), // handled above

                // if the chunk end reaches the buffer end but the stream is not done, fill more data and try again
                Some(CharactersChunkIndices { end, .. })
                    if !string_buffer.done && end == buffer.len() =>
                {
                    let old_buffer_len = buffer.len();
                    string_buffer.fill_no_compact();
                    let new_buffer_len = string_buffer.buffer().len();
                    self.build_char_positions(
                        &string_buffer.buffer()[old_buffer_len..new_buffer_len],
                        old_buffer_len,
                    );
                    self.compact(&mut string_buffer);
                    continue;
                }

                // otherwise, return the chunk
                Some(ref n) => {
                    let chunk = buffer[n.start..n.end].to_string();
                    string_buffer.set_position(n.new_byte_position);
                    self.current_char_position = n.new_char_position;
                    return Some(chunk);
                }
            };
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    // use std::path::PathBuf;
    // use crate::stream::FileUtf8BlockReader;

    // fn get_test_file_path() -> String {
    //     let manifest_dir = env!("CARGO_MANIFEST_DIR");
    //     PathBuf::from(manifest_dir)
    //         .join("../test-data/realistic-1.0mb.txt")
    //         .to_string_lossy()
    //         .into_owned()
    // }

    // #[test]
    // fn test_string_buffer_compaction() {
    //     let reader = FileUtf8BlockReader::new(get_test_file_path().as_str(), 1024 * 8).unwrap();

    //     let mut string_buffer = StringBuffer::new(reader, 1024 * 16);

    //     while !string_buffer.done {
    //         let content = string_buffer.buffer();
    //         let content_len = content.len();
    //         let can = content.is_char_boundary(content_len - 10);
    //         if can {
    //             string_buffer.set_position(content_len - 10);
    //             assert!(
    //                 string_buffer.position == 0,
    //                 "Position should be updated correctly"
    //             );
    //             assert!(string_buffer.len() <= 10);
    //         }
    //     }
    // }

    #[test]
    fn test_characters_chunker_stream() {
        // let reader = FileUtf8BlockReader::new(FILE_PATH, 1024 * 8).unwrap();

        let reader = vec!["01234".to_string(), "56789".to_string()].into_iter();
        let overlap = 2;
        let chunk_size = 6;

        let chunker = CharactersChunker::new(chunk_size, overlap).unwrap();
        let mut chunked_iter = chunker.chunk_stream(reader);

        // for loop on the chunked iter
        for chunk in chunked_iter.by_ref() {
            println!("Chunk length: {}", chunk.len());
            println!("Chunk content: {}", &chunk);
        }
    }
}
