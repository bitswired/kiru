# kiru 🗡️

> **Cut through text at the speed of light**

A blazingly fast text chunking library for Rust, designed for RAG applications and large-scale text processing.

[![Crates.io](https://img.shields.io/crates/v/kiru.svg)](https://crates.io/crates/kiru)
[![Documentation](https://docs.rs/kiru/badge.svg)](https://docs.rs/kiru)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why kiru?

When building RAG (Retrieval-Augmented Generation) systems, you need to chunk documents *fast*. Whether you're processing millions of documents for your vector database or streaming real-time data, kiru delivers:

- ⚡ **Lightning Fast**: Process 100MB+ files at 500+ MB/s
- 🎯 **UTF-8 Safe**: Handles multi-byte characters correctly, never breaking in the middle of a character
- 🔄 **Zero-Copy Streaming**: Process gigabyte files with constant memory usage
- 🚀 **Parallel Processing**: Chunk multiple sources concurrently with Rayon
- 🎨 **Flexible Strategies**: Chunk by bytes or characters, your choice

## Benchmarks

```
file_chunking_by_bytes/4kb_chunk
                        time:   [195.32 ms 195.88 ms 196.44 ms]
                        thrpt:  [509.11 MB/s 510.56 MB/s 512.03 MB/s]

string_chunking_by_characters/4k_chars  
                        time:   [412.45 ms 413.21 ms 413.97 ms]
                        thrpt:  [241.57 MB/s 242.01 MB/s 242.46 MB/s]
```

*Benchmarked on 100MB text files with 4KB chunks and 10% overlap*

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
kiru = "0.1"
```

### Basic Usage

```rust
use kiru::{BytesChunker, Chunker};

// Chunk a string
let chunker = BytesChunker::new(1024, 128)?; // 1KB chunks, 128 bytes overlap
let chunks: Vec<String> = chunker
    .chunk_string("Your long text here...".to_string())
    .collect();

// Chunk a file (streaming, constant memory)
use kiru::{Source, StreamType};

let chunker = BytesChunker::new(4096, 512)?;
let stream = StreamType::from_source(&Source::File("huge_file.txt".to_string()))?;
for chunk in chunker.chunk_stream(stream) {
    // Process each chunk as it's generated
    send_to_vector_db(chunk);
}
```

### Advanced: Parallel Processing

```rust
use kiru::{ChunkerBuilder, ChunkerEnum};

// Process multiple sources in parallel
let sources = vec![
    "file://document1.txt",
    "https://example.com/page",
    "glob://*.md",
]
.into_iter()
.map(|s| s.to_string())
.collect();

let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
    chunk_size: 4096,
    overlap: 512,
});

// Stream chunks as they're processed in parallel
let chunks = chunker.on_sources_par_stream(sources, 1000)?;
for chunk in chunks {
    // Chunks arrive as soon as they're ready from any source
    process_chunk(chunk);
}
```

## Chunking Strategies

### BytesChunker
- Chunks by **byte count** while respecting UTF-8 boundaries
- Perfect for when you need consistent memory usage
- Ideal for embeddings with token limits

### CharactersChunker  
- Chunks by **character count** (grapheme clusters)
- Ensures exact character counts regardless of byte representation
- Best for character-limited APIs or display purposes

## Source Types

kiru can chunk from multiple sources:

- **Files**: Local filesystem paths
- **HTTP/HTTPS**: Web pages and APIs
- **Strings**: In-memory text
- **Glob patterns**: Multiple files matching a pattern

## Use Cases for RAG

### Vector Database Ingestion

```rust
// Process an entire knowledge base
let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
    chunk_size: 512,  // Optimized for embedding models
    overlap: 50,      // Maintain context between chunks
});

let sources = vec!["glob://./knowledge_base/**/*.md"];
let chunks = chunker.on_sources_par_stream(sources, 10000)?;

// Send to your vector DB
for chunk in chunks {
    let embedding = embed_text(&chunk);
    vector_db.insert(chunk, embedding);
}
```

### Real-time Document Processing

```rust
// Stream-process documents as they arrive
let chunker = BytesChunker::new(1024, 128)?;

// Process without loading entire file into memory
let stream = StreamType::from_source(&Source::File(new_document))?;
for chunk in chunker.chunk_stream(stream) {
    update_index(chunk);
}
```

## Performance Tips

1. **Use parallel processing** for multiple files: `on_sources_par_stream()`
2. **Tune chunk size** based on your embedding model's context window
3. **Adjust overlap** to balance between context preservation and storage
4. **Use BytesChunker** for maximum throughput
5. **Stream large files** instead of loading them into memory

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*Built with ❤️ for the RAG community*