use ::kiru as kiru_core;
use kiru_core::{
    BytesChunker, CharactersChunker, ChunkerBuilder, ChunkerWithStrategy, HigherOrderSource,
    Source, SourceGenerator,
};
use pyo3::prelude::*;

// ============================================================================
// Utility Functions
// ============================================================================

/// Parse a list of source strings into HigherOrderSource variants.
///
/// Args:
///     source_strings: List of strings with optional prefixes (e.g., "file://path.txt", "http://example.com").
///
/// Returns:
///     Result containing a vector of HigherOrderSource or a PyErr for invalid inputs.
///
/// Errors:
///     Returns PyValueError for unsupported prefixes (e.g., "sitemap://") or invalid prefixes.
fn parse_source_strings(source_strings: Vec<String>) -> PyResult<Vec<HigherOrderSource>> {
    source_strings
        .into_iter()
        .map(|s| match s.as_str() {
            u if u.starts_with("http://") || u.starts_with("https://") => {
                Ok(HigherOrderSource::Source(Source::Http(s)))
            }
            u if u.starts_with("file://") => Ok(HigherOrderSource::Source(Source::File(
                u.trim_start_matches("file://").to_string(),
            ))),
            u if u.starts_with("glob://") => Ok(HigherOrderSource::SourceGenerator(
                SourceGenerator::Glob(u.trim_start_matches("glob://").to_string()),
            )),
            u if u.starts_with("sitemap://") => Err(pyo3::exceptions::PyValueError::new_err(
                "Sitemap source type is not supported",
            )),
            u if u.starts_with("text://") => Ok(HigherOrderSource::Source(Source::Text(
                u.trim_start_matches("text://").to_string(),
            ))),
            u if u.contains("://") => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid source prefix: {u}"
            ))),
            _ => Ok(HigherOrderSource::Source(Source::Text(s))),
        })
        .collect::<PyResult<Vec<HigherOrderSource>>>()
}

// ============================================================================
// Python Classes
// ============================================================================

/// A factory for creating chunkers with specific strategies (bytes or characters).
#[pyclass]
pub struct Chunker;

/// Internal enum storing a concrete, static-dispatch builder.
enum PyChunker {
    Bytes(ChunkerWithStrategy<BytesChunker>),
    Chars(ChunkerWithStrategy<CharactersChunker>),
}

/// A wrapper around a chunker strategy, providing methods to chunk various sources.
#[pyclass]
pub struct ChunkerBuilderWrapper {
    inner: PyChunker,
}

/// An iterator over chunks produced from one or more sources.
#[pyclass]
pub struct ChunkerIterator {
    inner: Box<dyn Iterator<Item = String> + Send + Sync>,
}

// ============================================================================
// Python Methods
// ============================================================================

#[pymethods]
impl Chunker {
    /// Create a bytes-based chunker with the specified chunk size and overlap.
    ///
    /// Args:
    ///     chunk_size (int): The size of each chunk in bytes.
    ///     overlap (int): The number of bytes to overlap between chunks (must be less than chunk_size).
    ///
    /// Returns:
    ///     ChunkerBuilderWrapper: A wrapper for chunking sources.
    ///
    /// Raises:
    ///     ValueError: If chunk_size is 0 or overlap is not less than chunk_size.
    #[staticmethod]
    fn by_bytes(chunk_size: usize, overlap: usize) -> PyResult<ChunkerBuilderWrapper> {
        let chunker = ChunkerBuilder::by_bytes(chunk_size, overlap)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(ChunkerBuilderWrapper {
            inner: PyChunker::Bytes(chunker),
        })
    }

    /// Create a characters-based chunker with the specified chunk size and overlap.
    ///
    /// Args:
    ///     chunk_size (int): The size of each chunk in characters.
    ///     overlap (int): The number of characters to overlap between chunks (must be less than chunk_size).
    ///
    /// Returns:
    ///     ChunkerBuilderWrapper: A wrapper for chunking sources.
    ///
    /// Raises:
    ///     ValueError: If chunk_size is 0 or overlap is not less than chunk_size.
    #[staticmethod]
    fn by_characters(chunk_size: usize, overlap: usize) -> PyResult<ChunkerBuilderWrapper> {
        let chunker = ChunkerBuilder::by_characters(chunk_size, overlap)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(ChunkerBuilderWrapper {
            inner: PyChunker::Chars(chunker),
        })
    }
}

// Small helper to de-duplicate single-source handling.
impl ChunkerBuilderWrapper {
    fn on_source_internal(&self, source: Source) -> PyResult<ChunkerIterator> {
        match &self.inner {
            PyChunker::Bytes(b) => {
                let inner_iter = b
                    .on_source(source)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(ChunkerIterator {
                    inner: Box::new(inner_iter),
                })
            }
            PyChunker::Chars(c) => {
                let inner_iter = c
                    .on_source(source)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(ChunkerIterator {
                    inner: Box::new(inner_iter),
                })
            }
        }
    }

    fn on_sources_internal(&self, sources: Vec<Source>) -> PyResult<ChunkerIterator> {
        match &self.inner {
            PyChunker::Bytes(b) => {
                let inner_iter = b
                    .on_sources(sources)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(ChunkerIterator {
                    inner: Box::new(inner_iter),
                })
            }
            PyChunker::Chars(c) => {
                let inner_iter = c
                    .on_sources(sources)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(ChunkerIterator {
                    inner: Box::new(inner_iter),
                })
            }
        }
    }
    fn on_sources_par_internal(
        &self,
        sources: Vec<Source>,
        channel_size: usize,
    ) -> PyResult<ChunkerIterator> {
        match &self.inner {
            PyChunker::Bytes(b) => {
                let inner_iter = b
                    .on_sources_par_stream(sources, channel_size)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(ChunkerIterator {
                    inner: Box::new(inner_iter),
                })
            }
            PyChunker::Chars(c) => {
                let inner_iter = c
                    .on_sources_par_stream(sources, channel_size)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(ChunkerIterator {
                    inner: Box::new(inner_iter),
                })
            }
        }
    }
}

#[pymethods]
impl ChunkerBuilderWrapper {
    /// Chunk a single string input.
    ///
    /// Args:
    ///     text (str): The input text to chunk.
    ///
    /// Returns:
    ///     ChunkerIterator: An iterator over the chunks.
    ///
    /// Raises:
    ///     ValueError: If the input cannot be processed.
    fn on_string(&self, text: String) -> PyResult<ChunkerIterator> {
        let source = Source::Text(text);
        self.on_source_internal(source)
    }

    /// Chunk a single file from a local path.
    ///
    /// Args:
    ///     path (str): The path to the file (e.g., "path/to/file.txt").
    ///
    /// Returns:
    ///     ChunkerIterator: An iterator over the chunks.
    ///
    /// Raises:
    ///     ValueError: If the file cannot be read (e.g., does not exist).
    fn on_file(&self, path: String) -> PyResult<ChunkerIterator> {
        let source = Source::File(path);
        self.on_source_internal(source)
    }

    /// Chunk content from an HTTP/HTTPS URL.
    ///
    /// Args:
    ///     url (str): The URL to fetch content from (e.g., "http://example.com/text").
    ///
    /// Returns:
    ///     ChunkerIterator: An iterator over the chunks.
    ///
    /// Raises:
    ///     ValueError: If the URL cannot be fetched or content cannot be processed.
    fn on_http(&self, url: String) -> PyResult<ChunkerIterator> {
        let source = Source::Http(url);
        self.on_source_internal(source)
    }

    /// Chunk multiple sources specified as strings with prefixes.
    ///
    /// Supported prefixes:
    /// - `file://` for local files (e.g., "file://path/to/file.txt").
    /// - `http://` or `https://` for URLs (e.g., "http://example.com/text").
    /// - `text://` for raw text (e.g., "text://Hello world").
    /// - `glob://` for glob patterns (e.g., "glob://*.txt").
    /// - No prefix assumes a local file path (e.g., "path/to/file.txt").
    ///
    /// Args:
    ///     source_strings (List[str]): A list of source strings with optional prefixes.
    ///
    /// Returns:
    ///     ChunkerIterator: An iterator over the chunks from all sources.
    ///
    /// Raises:
    ///     ValueError: If any source has an invalid prefix, uses an unsupported type (e.g., "sitemap://"),
    ///                 or cannot be processed (e.g., file not found, invalid glob).
    fn on_sources(&self, source_strings: Vec<String>) -> PyResult<ChunkerIterator> {
        if source_strings.is_empty() {
            // Return an empty iterator for empty input
            return Ok(ChunkerIterator {
                inner: Box::new(std::iter::empty()),
            });
        }

        let higher_order_sources = parse_source_strings(source_strings)?;

        let sources = HigherOrderSource::into_flattened_sources(higher_order_sources)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        self.on_sources_internal(sources)
    }

    /// Chunk multiple sources in parallel, specified as strings with prefixes.
    ///
    /// Supported prefixes:
    /// - `file://` for local files (e.g., "file://path/to/file.txt").
    /// - `http://` or `https://` for URLs (e.g., "http://example.com/text").
    /// - `text://` for raw text (e.g., "text://Hello world").
    /// - `glob://` for glob patterns (e.g., "glob://*.txt").
    /// - No prefix assumes a local file path (e.g., "path/to/file.txt").
    ///
    /// Args:
    ///     source_strings (List[str]): A list of source strings with optional prefixes.
    ///     channel_size (Optional[int]): Number of chunks to buffer in the channel (default: 100).
    ///
    /// Returns:
    ///     ChunkerIterator: An iterator over the chunks from all sources.
    ///
    /// Raises:
    ///     ValueError: If any source has an invalid prefix, uses an unsupported type (e.g., "sitemap://"),
    ///                 or cannot be processed (e.g., file not found, invalid glob).
    fn on_sources_par(
        &self,
        source_strings: Vec<String>,
        channel_size: Option<usize>,
    ) -> PyResult<ChunkerIterator> {
        if source_strings.is_empty() {
            // Return an empty iterator for empty input
            return Ok(ChunkerIterator {
                inner: Box::new(std::iter::empty()),
            });
        }

        let higher_order_sources = parse_source_strings(source_strings)?;

        let sources = HigherOrderSource::into_flattened_sources(higher_order_sources)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        self.on_sources_par_internal(sources, channel_size.unwrap_or(1000))
    }
}

#[pymethods]
impl ChunkerIterator {
    /// Collect all chunks into a list.
    ///
    /// Returns:
    ///     List[str]: A list of all chunks.
    fn all(mut slf: PyRefMut<Self>) -> Vec<String> {
        slf.inner.by_ref().collect()
    }

    /// Return an iterator over the chunks.
    ///
    /// Returns:
    ///     ChunkerIterator: The iterator itself.
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Get the next chunk.
    ///
    /// Returns:
    ///     Optional[str]: The next chunk, or None if exhausted.
    fn __next__(mut slf: PyRefMut<Self>) -> Option<String> {
        slf.inner.next()
    }
}

// ============================================================================
// Python Module
// ============================================================================

#[pymodule]
fn kiru(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Chunker>()?;
    m.add_class::<ChunkerBuilderWrapper>()?;
    m.add_class::<ChunkerIterator>()?;
    Ok(())
}
