mod bytes_chunker;
mod characters_chunker;
mod stream;
pub use bytes_chunker::*;
pub use characters_chunker::*;

use std::io;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Source {
    Text(String),
    File(String),
}

#[derive(Error, Debug)]
pub enum ChunkingError {
    #[error("error reading file")]
    Io(#[from] io::Error),
    #[error("the overlap ({overlap}) must be less than the chunk size ({chunk_size})")]
    InvalidArguments { chunk_size: usize, overlap: usize },
    #[error("unknown data store error")]
    Unknown,
}
