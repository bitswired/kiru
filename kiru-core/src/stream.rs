use std::fs::File;
use std::io::Read;

use crate::ChunkingError;
use crate::Source;

use reqwest::blocking::Response;

pub struct FileUtf8BlockReader {
    reader: File,
    block_size: usize,
    leftover: Vec<u8>,
    done: bool,
}

impl FileUtf8BlockReader {
    pub fn new(path: &str, block_size: usize) -> Result<Self, ChunkingError> {
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

pub struct HttpUtf8BlockReader {
    response: Response,
    block_size: usize,
    leftover: Vec<u8>,
    done: bool,
}

impl HttpUtf8BlockReader {
    pub fn new(url: &str, block_size: usize) -> Result<Self, ChunkingError> {
        // Create a blocking HTTP client and send a GET request
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| ChunkingError::Http(e.to_string()))?;

        let response = client
            .get(url)
            .send()
            .map_err(|e| ChunkingError::Http(e.to_string()))?;

        // Ensure the response is successful (status code 2xx)
        if !response.status().is_success() {
            return Err(ChunkingError::Http(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        Ok(Self {
            response,
            block_size,
            leftover: vec![],
            done: false,
        })
    }
}

impl Iterator for HttpUtf8BlockReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Start with leftover bytes from previous iteration
        let mut buffer = Vec::with_capacity(self.block_size + 4); // +4 for potential UTF-8 leftover
        buffer.extend_from_slice(&self.leftover);
        self.leftover.clear();

        // Try to read exactly block_size bytes from the HTTP response
        let mut temp = vec![0u8; self.block_size];
        let n = match self.response.read(&mut temp) {
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

pub enum StreamType {
    File(FileUtf8BlockReader),
    Text(std::vec::IntoIter<String>),
    Http(HttpUtf8BlockReader),
}

impl StreamType {
    pub fn from_source(source: &Source) -> Result<Self, ChunkingError> {
        match source {
            Source::File(path) => {
                let reader = FileUtf8BlockReader::new(path, 1024 * 8)?;
                Ok(StreamType::File(reader))
            }
            Source::Text(text) => {
                let iterator = vec![text.clone()].into_iter();
                Ok(StreamType::Text(iterator))
            }
            Source::Http(url) => {
                let reader = HttpUtf8BlockReader::new(url, 1024 * 8)?;
                Ok(StreamType::Http(reader))
            }
        }
    }
}

impl Iterator for StreamType {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        match self {
            StreamType::File(r) => r.next(),
            StreamType::Text(r) => r.next(),
            StreamType::Http(r) => r.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BytesChunker, Chunker};

    use super::*;

    const file_path: &str = "../..//test-data/realistic-100.0mb.txt";

    #[test]
    fn s() {
        let reader = FileUtf8BlockReader::new(file_path, 1024 * 64).unwrap();

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

    #[test]
    fn ttt() {
        let reader = StreamType::from_source(&Source::Http(
            "https://en.wikipedia.org/wiki/List_of_French_monarchs".to_string(),
        ))
        .unwrap();

        let chunker = BytesChunker::new(1024 * 16, 1024 * 4).unwrap();

        for x in chunker.chunk_stream(reader) {
            println!("Chunk len: {}", x.len());
            println!("Chunk content: {}", &x);
        }
    }
}
