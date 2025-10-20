use pyo3::prelude::*;

mod chunker;
pub use chunker::*;

#[pyclass]
pub struct BytesChunkerBuilder {
    chunk_size: usize,
    overlap: usize,
}

#[pyclass]
pub struct CharactersChunkerBuilder {
    chunk_size: usize,
    overlap: usize,
}

#[pyclass]
pub struct Chunker {
    inner: Box<dyn Iterator<Item = String> + Send + Sync>,
}

#[pymethods]
impl Chunker {
    #[staticmethod]
    fn by_bytes(chunk_size: usize, overlap: usize) -> PyResult<BytesChunkerBuilder> {
        if overlap >= chunk_size {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "overlap ({}) must be less than chunk_size ({})",
                overlap, chunk_size
            )));
        }
        Ok(BytesChunkerBuilder {
            chunk_size,
            overlap,
        })
    }

    #[staticmethod]
    fn by_characters(chunk_size: usize, overlap: usize) -> PyResult<CharactersChunkerBuilder> {
        if overlap >= chunk_size {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "overlap ({}) must be less than chunk_size ({})",
                overlap, chunk_size
            )));
        }
        Ok(CharactersChunkerBuilder {
            chunk_size,
            overlap,
        })
    }

    fn all(mut slf: PyRefMut<Self>) -> Vec<String> {
        slf.inner.by_ref().collect()
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<String> {
        slf.inner.next()
    }
}

#[pymethods]
impl BytesChunkerBuilder {
    fn from_text(&self, text: String) -> PyResult<Chunker> {
        let iterator = chunk_string_by_bytes(text, self.chunk_size, self.overlap)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Chunker {
            inner: Box::new(iterator),
        })
    }

    fn from_file(&self, path: String) -> PyResult<Chunker> {
        let iterator = chunk_file_by_bytes(path, self.chunk_size, self.overlap)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Chunker {
            inner: Box::new(iterator),
        })
    }
}

#[pymethods]
impl CharactersChunkerBuilder {
    fn from_text(&self, text: String) -> PyResult<Chunker> {
        let iterator = chunk_string_by_characters(text, self.chunk_size, self.overlap)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Chunker {
            inner: Box::new(iterator),
        })
    }

    fn from_file(&self, path: String) -> PyResult<Chunker> {
        let iterator = chunk_file_by_characters(path, self.chunk_size, self.overlap)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(Chunker {
            inner: Box::new(iterator),
        })
    }
}

#[pymodule]
fn kiru(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Chunker>()?;
    m.add_class::<BytesChunkerBuilder>()?;
    m.add_class::<CharactersChunkerBuilder>()?;
    Ok(())
}
