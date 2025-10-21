use std::fs::File;
use std::io::Read;

pub struct FileUtf8BlockReader {
    reader: File,
    block_size: usize,
    leftover: Vec<u8>,
    done: bool,
}

impl FileUtf8BlockReader {
    pub fn new(path: &str, block_size: usize) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        Ok(Self {
            reader: file,
            block_size,
            leftover: vec![],
            done: false,
        })
    }
}

impl Iterator for FileUtf8BlockReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Start with leftover bytes from previous iteration
        let mut buffer = Vec::with_capacity(self.block_size + 4); // +4 for potential UTF-8 leftover
        buffer.extend_from_slice(&self.leftover);
        self.leftover.clear();

        // Always try to read exactly block_size bytes
        let mut temp = vec![0u8; self.block_size];
        let n = match self.reader.read(&mut temp) {
            Ok(0) => {
                self.done = true;
                0
            }
            Ok(n) => n,
            Err(_) => {
                self.done = true;
                return None;
            }
        };

        // If we read nothing and have no leftover, we're done
        if n == 0 && buffer.is_empty() {
            return None;
        }

        buffer.extend_from_slice(&temp[..n]);

        // Validate UTF-8
        let valid_up_to = match std::str::from_utf8(&buffer) {
            Ok(_) => buffer.len(),
            Err(e) => {
                let valid = e.valid_up_to();
                // Save incomplete UTF-8 sequence for next iteration
                // (At most 3 bytes for incomplete UTF-8 sequence)
                self.leftover.extend_from_slice(&buffer[valid..]);
                valid
            }
        };

        // If we have no valid UTF-8 at all, something is wrong
        if valid_up_to == 0 {
            if self.done {
                return None;
            }
            // This shouldn't normally happen, but skip this byte and continue
            eprintln!("Warning: No valid UTF-8 found in block");
            return self.next();
        }

        let text = std::str::from_utf8(&buffer[..valid_up_to])
            .expect("Already validated")
            .to_string();

        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const file_path: &str =
        "/Users/jimzer/Projects/bitswired-clean/kiru/test-data/realistic-100.0mb.txt";

    #[test]
    fn s() {
        let reader = FileUtf8BlockReader::new(file_path, 1024 * 8).unwrap();

        let mut min_chunk_len = usize::MAX;
        let mut max_chunk_len = 0;
        let mut total_len = 0;

        for line in reader {
            total_len += line.len();
            if line.len() > max_chunk_len {
                max_chunk_len = line.len();
            }
            if line.len() < min_chunk_len {
                min_chunk_len = line.len();
            }

            if line.len() < 100 {
                println!("Chunk len {}", line.len());
            }
        }

        println!(
            "Total len: {}, Max len: {}, Min len: {}",
            total_len, max_chunk_len, min_chunk_len
        );
    }
}
