use super::stream::*;
use super::ChunkingError;

#[derive(Debug, Clone, Copy)]
struct CharPosition {
    start: usize,
    len: usize,
}

fn build_char_positions(text: &str, offset: usize) -> Vec<CharPosition> {
    text.char_indices()
        .map(|(pos, ch)| CharPosition {
            start: pos + offset,
            len: ch.len_utf8(),
        })
        .collect()
}

fn chunk_indices(
    text: &str,
    char_positions: &Vec<CharPosition>,
    chunk_size: usize,
    overlap: usize,
    current_byte_position: &mut usize,
    current_char_position: &mut usize,
) -> Option<(usize, usize)> {
    let text_len = text.len();
    let chars_len = char_positions.len();

    // Done
    if *current_char_position >= chars_len {
        return None;
    }

    let start_idx = *current_char_position;
    let end_idx = (start_idx + chunk_size).min(chars_len);
    let start_byte = char_positions[start_idx].start;

    let end_byte = if end_idx >= chars_len {
        text_len
    } else {
        char_positions[end_idx - 1].start + char_positions[end_idx - 1].len
    };

    // If we've reached the end of text, we're done after this chunk
    if end_idx >= chars_len {
        *current_char_position = chars_len;
        *current_byte_position = text_len;
        return Some((start_byte, end_byte));
    }

    // Calculate next position
    let step = chunk_size.saturating_sub(overlap);

    // return Some((start_byte, end_byte));
    let next_char_position = start_idx + step;
    let next_byte_position = char_positions[next_char_position].start;

    // Update positions
    *current_char_position = next_char_position;
    *current_byte_position = next_byte_position;

    Some((start_byte, end_byte))
}

pub fn chunk_string_by_characters(
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
    let mut current_char_position = 0;

    let char_positions = build_char_positions(&text, 0);

    let iterator = std::iter::from_fn(move || {
        let (start, end) = chunk_indices(
            &text,
            &char_positions,
            chunk_size,
            overlap,
            &mut current_position,
            &mut current_char_position,
        )?;

        Some(text[start..end].to_string())
    });

    Ok(iterator)
}

pub fn chunk_file_by_characters(
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
    let mut char_positions = Vec::new();
    let mut position = 0;
    let mut char_position = 0;
    let mut file_done = false;

    let iterator = std::iter::from_fn(move || {
        loop {
            // Ensure we have enough data in the buffer for a full chunk
            // Keep reading until we have chunk_size * 5 bytes OR reach EOF
            while !file_done && char_positions.len() - char_position < chunk_size * 5 {
                match reader.next() {
                    Some(block) => {
                        let cp = build_char_positions(&block, buffer.len());
                        buffer.push_str(&block);
                        char_positions.extend(cp);
                    }
                    None => {
                        file_done = true;
                        break;
                    }
                }
            }

            // Try to get a chunk from current buffer
            if let Some((start, end)) = chunk_indices(
                &buffer,
                &char_positions,
                chunk_size,
                overlap,
                &mut position,
                &mut char_position,
            ) {
                return Some(buffer[start..end].to_string());
            }

            // If we can't get a chunk and file is done, we're done
            if file_done {
                return None;
            }

            // Compact the buffer if needed
            if char_position > chunk_size * 5 {
                let keep_from_chars = char_position.saturating_sub(chunk_size * 2);
                let keep_from_bytes = char_positions[keep_from_chars].start;
                buffer.drain(..keep_from_bytes);
                char_positions.drain(..keep_from_chars);

                // remove inplace from char_positions
                for cp in char_positions.iter_mut() {
                    cp.start -= keep_from_bytes;
                }

                position = position.saturating_sub(keep_from_bytes);
                char_position = char_position.saturating_sub(keep_from_chars);
            }
        }
    });

    Ok(iterator)
}
#[cfg(test)]
mod tests {

    use super::*;

    // #[test]
    // fn test_build_char_positions_ascii() {
    //     let text = "hello🥲a";
    //     let char_positions: Vec<_> = text
    //         .char_indices()
    //         .map(|(pos, ch)| CharPosition {
    //             start: pos,
    //             len: ch.len_utf8(),
    //         })
    //         .collect();

    //     let chunk_size = 3;
    //     let overlap = 2;

    //     let mut current_byte_position: usize = 0;
    //     let mut current_char_position: usize = 0;

    //     println!("{:?}", text);
    //     println!("{:?}", char_positions);
    //     println!("{:?}", text.len());
    //     println!("{:?}", text[5..(5 + 4)].to_string());

    //     while let Some(result) = chunk_indices(
    //         text,
    //         &char_positions,
    //         chunk_size,
    //         overlap,
    //         &mut current_byte_position,
    //         &mut current_char_position,
    //     ) {
    //         // Process the result
    //         println!(
    //             "Got chunk: {:?}, content: {}",
    //             result,
    //             &text[result.0..result.1]
    //         );
    //         // Do something with result...
    //     }
    // }

    #[test]
    fn test_chunk_string_by_characters() {
        let text = "01234567890123456789";
        let chunk_size = 10;
        let overlap = 3;

        let chunks: Vec<String> = chunk_string_by_characters(text.to_string(), chunk_size, overlap)
            .unwrap()
            .collect();

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "0123456789");
        assert_eq!(chunks[1], "7890123456");
        assert_eq!(chunks[2], "456789");
    }

    #[test]
    fn test_chunk_file_by_characters() {
        let path = "../test-data/realistic-100.0mb.txt".to_string();

        let chunk_size = 8192;
        let overlap = 0;
        let chunks: Vec<String> = chunk_file_by_characters(path, chunk_size, overlap)
            .unwrap()
            .collect();
    }
}
