use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

/// Create a temporary file with given content
pub fn create_temp_file(content: &str) -> (TempDir, String) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = dir.path().join("test.txt");
    let mut file = File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    drop(file);
    (dir, path.to_string_lossy().to_string())
}

/// Verify all chunks are valid UTF-8 (universal for all chunkers)
pub fn assert_all_valid_utf8(chunks: &[String]) {
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(
            std::str::from_utf8(chunk.as_bytes()).is_ok(),
            "Chunk {} contains invalid UTF-8",
            i
        );
    }
}
