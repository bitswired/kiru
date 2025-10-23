# kiru ⚡🗡️

> **Cut through text at the speed of light**

The fastest text chunking library for RAG applications. Available for both Rust and Python.

[![Crates.io](https://img.shields.io/crates/v/kiru.svg)](https://crates.io/crates/kiru)
[![PyPI](https://img.shields.io/pypi/v/kiru.svg)](https://pypi.org/project/kiru/)
[![Documentation](https://docs.rs/kiru/badge.svg)](https://docs.rs/kiru)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What is kiru?

kiru is a high-performance text chunking library designed for modern RAG (Retrieval-Augmented Generation) systems. When you need to split millions of documents for vector databases or process streaming data in real-time, kiru delivers unmatched speed without sacrificing correctness.

### Key Features

- **⚡ Blazing Fast**: 500+ MB/s throughput, 10-100x faster than pure Python alternatives
- **🎯 UTF-8 Safe**: Never breaks multi-byte characters or emoji
- **💾 Memory Efficient**: Stream gigabyte files with constant memory usage
- **🚀 Parallel Processing**: Utilize all CPU cores automatically
- **🔌 Multiple Sources**: Files, URLs, strings, and glob patterns
- **🛠️ Flexible Strategies**: Chunk by bytes or characters
- **🦀 Rust Core**: Zero-copy performance with memory safety
- **🐍 Python Bindings**: Pythonic API for ease of use

## Performance Comparison

```
100MB File, 4KB chunks, 10% overlap:

| Implementation | Time    | Memory | Throughput |
|----------------|---------|--------|------------|
| kiru (Rust)    | 0.19s   | 5MB    | 526 MB/s   |
| kiru (Python)  | 0.21s   | 10MB   | 476 MB/s   |
| LangChain      | 9.87s   | 850MB  | 10 MB/s    |

kiru is 50x faster and uses 85x less memory than alternatives!
```

## Quick Start

### Python 🐍

```bash
pip install kiru
```

```python
from kiru import Chunker

# Create a chunker
chunker = Chunker.by_bytes(
    chunk_size=1024,  # 1KB chunks
    overlap=128       # 128 bytes overlap
)

# Chunk text
chunks = chunker.on_string("Your text here...").all()

# Chunk files in parallel
sources = ["file://doc1.txt", "https://example.com/page", "glob://*.md"]
for chunk in chunker.on_sources_par(sources):
    process(chunk)
```

### Rust 🦀

```toml
[dependencies]
kiru = "0.1"
```

```rust
use kiru::{BytesChunker, Chunker};

// Create a chunker
let chunker = BytesChunker::new(1024, 128)?;

// Chunk text
let chunks: Vec<String> = chunker
    .chunk_string("Your text here...".to_string())
    .collect();

// Stream large files
use kiru::{Source, StreamType};
let stream = StreamType::from_source(&Source::File("huge.txt".to_string()))?;
for chunk in chunker.chunk_stream(stream) {
    process(chunk);
}
```

## Use Cases

### Building RAG Systems

```python
# Perfect for vector database ingestion
chunker = Chunker.by_bytes(512, 50)  # Tuned for embedding models

documents = ["glob://knowledge_base/**/*.md"]
chunks = chunker.on_sources_par(documents, channel_size=10000)

for chunk in chunks:
    embedding = model.encode(chunk)
    vector_db.insert(chunk, embedding)
```

### Real-time Processing

```python
# Stream processing without memory overhead
for chunk in chunker.on_file("10GB_file.txt"):
    # Each chunk generated on-demand
    send_to_queue(chunk)
```

### Parallel Document Processing

```rust
// Process hundreds of documents concurrently
let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
    chunk_size: 4096,
    overlap: 512,
});

let sources = vec!["glob://docs/**/*.txt"];
let chunks = chunker.on_sources_par_stream(sources, 1000)?;
```

## Architecture

```
┌─────────────────────────────────────────┐
│           Python Application            │
├─────────────────────────────────────────┤
│          kiru-py (PyO3 Bindings)        │
├─────────────────────────────────────────┤
│         kiru-core (Rust Library)        │
├─────────────────────────────────────────┤
│        ┌──────────┬───────────┐         │
│        │ Chunkers │ Streaming │         │  
│        │  Engine  │   Engine  │         │
│        └──────────┴───────────┘         │
└─────────────────────────────────────────┘
```


## Chunking Strategies

### Bytes Chunking
- Splits on byte boundaries while respecting UTF-8
- Fastest performance (500+ MB/s)
- Ideal for token-limited models

### Characters Chunking  
- Splits on character (grapheme) boundaries
- Consistent character counts
- Perfect for character-limited APIs

## Project Structure

```
kiru/
├── kiru-core/        # Rust implementation
│   ├── src/          # Core chunking algorithms
│   ├── benches/      # Performance benchmarks
│   └── tests/        # Property-based tests
├── kiru-py/          # Python bindings
│   ├── src/          # PyO3 wrapper code
│   └── python/       # Python tests & benchmarks
└── justfile          # Development commands
```

## Development

```bash
# Setup
git clone https://github.com/yourusername/kiru.git
cd kiru

# Run tests
just test-all

# Run benchmarks
just bench-rust
just bench-py

# Build everything
just build-all
```

## Benchmarks

Run comprehensive benchmarks:

```bash
# Rust benchmarks
cd kiru-core
cargo bench

# Python benchmarks
cd kiru-py
python python/bench.py
```

## Why "kiru"?

"Kiru" (切る) is Japanese for "to cut" - reflecting the library's purpose of cutting text into chunks at lightning speed.

## Contributing

We welcome contributions! Please check out our [Contributing Guide](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Credits

Built with:
- [PyO3](https://pyo3.rs) - Rust bindings for Python
- [Rayon](https://github.com/rayon-rs/rayon) - Data parallelism for Rust
- [maturin](https://www.maturin.rs) - Build and publish Rust Python extensions

---

**Ready to cut through text at the speed of light?**

Get started with [Python](https://pypi.org/project/kiru/) or [Rust](https://crates.io/crates/kiru) today!