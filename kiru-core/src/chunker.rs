use crossbeam_channel::bounded;
use glob::glob;
use rayon::prelude::*;
use std::io;
use std::thread;
use thiserror::Error;

use crate::{BytesChunker, CharactersChunker, StreamType};

#[derive(Debug, Clone)]
pub enum Source {
    Text(String),
    File(String),
    Http(String),
}

#[derive(Debug, Clone)]
pub enum SourceGenerator {
    Glob(String),
    Sitemap(String),
}

pub enum HigherOrderSource {
    Source(Source),
    SourceGenerator(SourceGenerator),
}

impl HigherOrderSource {
    pub fn into_sources(self) -> Result<Vec<Source>, ChunkingError> {
        match self {
            HigherOrderSource::Source(s) => Ok(vec![s]),
            HigherOrderSource::SourceGenerator(SourceGenerator::Glob(pattern)) => {
                let paths = glob(&pattern).map_err(|_| {
                    ChunkingError::Io(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid glob pattern",
                    ))
                })?;
                let sources = paths
                    .filter_map(|entry| match entry {
                        Ok(path) => Some(Source::File(path.to_string_lossy().to_string())),
                        Err(_) => None,
                    })
                    .collect::<Vec<_>>();
                Ok(sources)
            }
            HigherOrderSource::SourceGenerator(SourceGenerator::Sitemap(url)) => {
                println!("Fetching sitemap from URL: {}", url);
                Err(ChunkingError::Unknown) // Placeholder for future implementation
            }
        }
    }

    pub fn into_flattened_sources(
        higher_order_sources: Vec<HigherOrderSource>,
    ) -> Result<Vec<Source>, ChunkingError> {
        higher_order_sources
            .into_iter()
            .map(|hos| hos.into_sources())
            .collect::<Result<Vec<Vec<Source>>, ChunkingError>>()
            .map(|vecs| vecs.into_iter().flatten().collect())
    }
}

#[derive(Error, Debug)]
pub enum ChunkingError {
    #[error("error reading file")]
    Io(#[from] io::Error),
    #[error("error making HTTP request: {0}")]
    Http(String),
    #[error("the overlap ({overlap}) must be less than the chunk size ({chunk_size})")]
    InvalidArguments { chunk_size: usize, overlap: usize },
    #[error("unknown data store error")]
    Unknown,
}

pub struct StringBuffer<I>
where
    I: Iterator<Item = String>,
{
    stream: I,
    buffer: String,
    min_buffer_size: usize,
    pub done: bool,
    pub position: usize,
}

impl<I> StringBuffer<I>
where
    I: Iterator<Item = String>,
{
    pub fn new(stream: I, min_buffer_size: usize) -> Self {
        Self {
            stream,
            buffer: String::with_capacity(min_buffer_size * 2),
            min_buffer_size,
            done: false,
            position: 0,
        }
    }

    pub fn fill(&mut self) {
        self.compact();
        self.fill_no_compact();
    }

    pub fn fill_no_compact(&mut self) {
        // if we are not done and buffer already meets min size, try to add one block
        if !self.done && self.buffer.len() >= self.min_buffer_size {
            if let Some(chunk) = self.stream.next() {
                self.buffer.push_str(&chunk);
            } else {
                self.done = true;
            }
        } else {
            // keep filling until done or buffer meets min size
            while !self.done && self.buffer.len() < self.min_buffer_size {
                match self.stream.next() {
                    Some(chunk) => {
                        self.buffer.push_str(&chunk);
                    }
                    None => {
                        self.done = true;
                        break;
                    }
                }
            }
        }
    }

    pub fn buffer(&self) -> &String {
        &self.buffer
    }

    fn compact(&mut self) {
        if self.position > self.buffer.len() / 2 {
            self.buffer.drain(0..self.position);
            self.position = 0;
        }
    }

    pub fn compact_to(&mut self, byte_position: usize) {
        if byte_position > 0 {
            self.buffer.drain(0..byte_position);
            self.position = self.position.saturating_sub(byte_position);
        }
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

pub trait Chunker {
    fn chunk_string(self, input: String) -> impl Iterator<Item = String>;
    fn chunk_stream(self, input: impl Iterator<Item = String>) -> impl Iterator<Item = String>;
}

#[derive(Clone)]
pub enum ChunkerEnum {
    Bytes { chunk_size: usize, overlap: usize },
    Characters { chunk_size: usize, overlap: usize },
}

pub struct ChunkerBuilder {}

impl ChunkerBuilder {
    pub fn by_bytes(chunker_params: ChunkerEnum) -> ChunkerWithStrategy {
        ChunkerWithStrategy { chunker_params }
    }

    pub fn by_characters(chunker_params: ChunkerEnum) -> ChunkerWithStrategy {
        ChunkerWithStrategy { chunker_params }
    }
}

// Update ChunkerWithStrategy to use ChunkerEnum
pub struct ChunkerWithStrategy {
    chunker_params: ChunkerEnum,
}

impl ChunkerWithStrategy {
    pub fn on_source(
        &self,
        source: Source,
    ) -> Result<Box<dyn Iterator<Item = String> + Send + Sync>, ChunkingError> {
        let stream = StreamType::from_source(&source)?;

        match self.chunker_params {
            ChunkerEnum::Bytes {
                chunk_size,
                overlap,
            } => {
                let chunker = BytesChunker::new(chunk_size, overlap)?;
                Ok(Box::new(chunker.chunk_stream(stream)))
            }
            ChunkerEnum::Characters {
                chunk_size,
                overlap,
            } => {
                let chunker = CharactersChunker::new(chunk_size, overlap)?;
                Ok(Box::new(chunker.chunk_stream(stream)))
            }
        }
    }

    pub fn on_sources(
        &self,
        sources: Vec<Source>,
    ) -> Result<Box<dyn Iterator<Item = String> + Send + Sync>, ChunkingError> {
        let iterators: Vec<Box<dyn Iterator<Item = String> + Send + Sync>> = sources
            .into_iter()
            .map(|s| self.on_source(s))
            .collect::<Result<Vec<_>, _>>()?;

        // Chain all iterators together
        let chained = iterators.into_iter().flatten();

        Ok(Box::new(chained))
    }

    pub fn on_sources_par(&self, sources: Vec<Source>) -> Result<Vec<String>, ChunkingError> {
        sources
            .into_par_iter()
            .map(|source| {
                // Each thread: fetches source + chunks it + collects
                let iter = self.on_source(source)?;
                Ok(iter.collect::<Vec<String>>())
            })
            .collect::<Result<Vec<Vec<String>>, ChunkingError>>()
            .map(|chunks| chunks.into_iter().flatten().collect())
    }

    pub fn on_sources_par_stream(
        &self,
        sources: Vec<Source>,
        channel_size: usize,
    ) -> Result<Box<dyn Iterator<Item = String> + Send + Sync>, ChunkingError> {
        // Pre-validate: check all sources are accessible
        for source in &sources {
            StreamType::from_source(source)?; // This validates the source
        }

        let (sender, receiver) = bounded(channel_size);
        let chunker_params = self.chunker_params.clone();

        thread::spawn({
            move || {
                sources.into_par_iter().for_each(|source| {
                    let sender = sender.clone();
                    let chunker = ChunkerWithStrategy {
                        chunker_params: chunker_params.clone(),
                    };

                    // Should not fail since we pre-validated
                    if let Ok(iter) = chunker.on_source(source) {
                        for chunk in iter {
                            if sender.send(chunk).is_err() {
                                break;
                            }
                        }
                    }
                });
            }
        });

        let iterator = std::iter::from_fn(move || receiver.recv().ok());

        Ok(Box::new(iterator))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_chunker_usage() {
        let hos: Vec<HigherOrderSource> = vec![HigherOrderSource::SourceGenerator(
            SourceGenerator::Glob(String::from("**/*.rs")),
        )];

        let sources = HigherOrderSource::into_flattened_sources(hos).unwrap();

        println!("{:?}", sources);

        let u = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
            chunk_size: 1024,
            overlap: 128,
        })
        .on_sources_par_stream(sources, 1000)
        .unwrap();

        // Add assertions here
        for chunk in u {
            println!("-------------------------------------");
            println!("{}", chunk);
            println!("-------------------------------------");
        }
    }

    #[test]
    fn chunka() {
        let sources = vec!["../test-data/realistic-*"; 10]
            .into_iter()
            .map(|s| HigherOrderSource::SourceGenerator(SourceGenerator::Glob(s.to_string())))
            .collect();
        let sources = HigherOrderSource::into_flattened_sources(sources).unwrap();

        let start = Instant::now();
        let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
            chunk_size: 1024,
            overlap: 128,
        });
        let chunks = chunker
            .on_sources_par_stream(sources, 10000)
            .unwrap()
            .collect::<Vec<_>>();
        let size: usize = chunks.iter().map(|s| s.len()).sum();
        let elapsed = start.elapsed();

        println!("Total size: {:?} MB", size as f64 / 1024.0 / 1024.0);
        println!("Elapsed time: {:?} seconds", elapsed.as_secs_f64());
        println!(
            "Throughput: {:?} MB/s",
            (size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
        )
    }
}
