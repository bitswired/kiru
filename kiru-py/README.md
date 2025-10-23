# kiru ⚡

> **Chunk 'em all** - The fastest text chunker for Python

Lightning-fast text chunking for RAG applications, powered by Rust 🦀

[![PyPI](https://img.shields.io/pypi/v/kiru.svg)](https://pypi.org/project/kiru/)
[![Python](https://img.shields.io/pypi/pyversions/kiru.svg)](https://pypi.org/project/kiru/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why kiru?

Building a RAG system? Chunking documents for vector search? **kiru** makes text chunking a non-issue:

- 🚀 **10-100x faster than pure Python alternatives**
- 🎯 **Precise UTF-8 handling** - never breaks characters
- 💪 **Parallel processing** - utilize all your CPU cores
- 🔄 **Streaming support** - process files larger than RAM
- 🐍 **Pythonic API** - feels natural, works everywhere
- 🦀 **Rust-powered** - zero-copy performance, Python convenience

## Performance

```python
# Benchmarked on 100MB text file, 4KB chunks, 10% overlap

| Library    | Strategy | Time (s) | Memory (MB) | Throughput (MB/s) |
|------------|----------|----------|-------------|-------------------|
| kiru       | bytes    | 0.19     | 10          | 526.3             |
| kiru       | chars    | 0.41     | 12          | 243.9             |
| LangChain  | chars    | 9.87     | 850         | 10.1              |
```

**kiru is 50x faster and uses 85x less memory than LangChain!**

## Installation

```bash
pip install kiru
```

## Quick Start

### Basic Usage

```python
from kiru import Chunker

# Simple string chunking
chunker = Chunker.by_bytes(
    chunk_size=512,  # 512 bytes per chunk
    overlap=128      # 128 bytes overlap
)

chunks = chunker.on_string("Your long document text...").all()
```

### RAG Pipeline Example

```python
from kiru import Chunker
import chromadb

# Initialize chunker for your embedding model's context window
chunker = Chunker.by_characters(
    chunk_size=1000,  # Characters, not tokens
    overlap=100       # Maintain context between chunks
)

# Process documents for vector database
documents = [
    "file://report.pdf",
    "https://example.com/article",
    "glob://*.md"
]

# Parallel processing with streaming
chunks_iter = chunker.on_sources_par(documents, channel_size=1000)

# Stream directly to vector DB
for chunk in chunks_iter:
    embedding = embed(chunk)  # Your embedding function
    vector_db.add(chunk, embedding)
```

## Chunking Strategies

### By Bytes
Perfect for token-limited models and consistent memory usage:

```python
chunker = Chunker.by_bytes(chunk_size=4096, overlap=512)
```

### By Characters
Ideal for character-limited APIs and precise control:

```python
chunker = Chunker.by_characters(chunk_size=1000, overlap=200)
```

## Input Sources

kiru handles multiple input types seamlessly:

```python
# From string
chunks = chunker.on_string("text...").all()

# From file
chunks = chunker.on_file("/path/to/document.txt").all()

# From URL
chunks = chunker.on_http("https://example.com/article").all()

# Multiple sources (parallel processing!)
sources = [
    "file://doc1.txt",
    "https://example.com/page",
    "text://Inline text content",
    "glob://*.md"  # All markdown files
]

# Process in parallel, stream results
for chunk in chunker.on_sources_par(sources):
    process(chunk)
```

## Advanced Features

### Streaming Large Files

Process files without loading them into memory:

```python
# Process a 10GB file with constant memory usage
for chunk in chunker.on_file("huge_dataset.txt"):
    # Each chunk is generated on-demand
    send_to_processing_queue(chunk)
```

### Parallel Processing

Utilize all CPU cores for maximum throughput:

```python
# Chunk 100 documents in parallel
documents = ["file://doc{}.txt".format(i) for i in range(100)]

# Returns iterator immediately, chunks processed in background
chunks_iter = chunker.on_sources_par(
    documents,
    channel_size=10000  # Buffer size for parallel processing
)

# Chunks arrive as soon as they're ready
for chunk in chunks_iter:
    vector_db.insert(chunk)
```

### Integration Examples

#### With LangChain

```python
from kiru import Chunker
from langchain.embeddings import OpenAIEmbeddings
from langchain.vectorstores import Chroma

# Use kiru for fast chunking
chunker = Chunker.by_bytes(chunk_size=1000, overlap=100)
chunks = chunker.on_file("document.pdf").all()

# Then use LangChain for embeddings and storage
embeddings = OpenAIEmbeddings()
vector_store = Chroma.from_texts(chunks, embeddings)
```

#### With LlamaIndex

```python
from kiru import Chunker
from llama_index import Document, VectorStoreIndex

# Chunk with kiru
chunker = Chunker.by_characters(chunk_size=512, overlap=50)
chunks = chunker.on_sources(["file://doc1.txt", "file://doc2.txt"]).all()

# Create LlamaIndex documents
documents = [Document(text=chunk) for chunk in chunks]
index = VectorStoreIndex.from_documents(documents)
```

## Benchmarking

Compare performance yourself:

```python
import time
from kiru import Chunker

# kiru
start = time.time()
chunker = Chunker.by_bytes(4096, 512)
kiru_chunks = chunker.on_file("large_file.txt").all()
print(f"kiru: {time.time() - start:.2f}s")

# LangChain
from langchain.text_splitter import CharacterTextSplitter

start = time.time()
with open("large_file.txt") as f:
    text = f.read()
splitter = CharacterTextSplitter(chunk_size=4096, chunk_overlap=512)
langchain_chunks = splitter.split_text(text)
print(f"LangChain: {time.time() - start:.2f}s")
```

## API Reference

### Chunker

```python
# Create chunkers
Chunker.by_bytes(chunk_size: int, overlap: int) -> ChunkerBuilder
Chunker.by_characters(chunk_size: int, overlap: int) -> ChunkerBuilder
```

### ChunkerBuilder

```python
# Single source methods
.on_string(text: str) -> ChunkerIterator
.on_file(path: str) -> ChunkerIterator
.on_http(url: str) -> ChunkerIterator

# Multiple sources
.on_sources(sources: List[str]) -> ChunkerIterator
.on_sources_par(sources: List[str], channel_size: int = 1000) -> ChunkerIterator
```

### ChunkerIterator

```python
# Get all chunks at once
.all() -> List[str]

# Or iterate one by one
for chunk in chunker_iterator:
    process(chunk)
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*Built with 🦀 Rust for ⚡ Python*

**Ready to chunk 'em all?** Get started with `pip install kiru`!