use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use kiru::{BytesChunker, CharactersChunker, Chunker, Source, StreamType};
use std::fs;
use std::hint::black_box;
use std::time::Duration;

const LARGE_FILE_PATH: &str = "../test-data/realistic-5.0mb.txt";

fn benchmark_file_chunking_by_bytes(c: &mut Criterion) {
    // Check if file exists
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        println!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        println!("   Skipping file benchmarks");
        return;
    }

    let file_size = fs::metadata(LARGE_FILE_PATH).map(|m| m.len()).unwrap_or(0);

    let mut group = c.benchmark_group("file_chunking_by_bytes");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Bytes(file_size));

    // Benchmark different chunk sizes (in BYTES)
    for chunk_size in [512, 1024, 2048, 4096, 8192] {
        let overlap = chunk_size / 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb_chunk", chunk_size / 1024)),
            &(chunk_size, overlap),
            |b, &(chunk_size, overlap)| {
                b.iter(|| {
                    let chunker =
                        BytesChunker::new(black_box(chunk_size), black_box(overlap)).unwrap();
                    let stream = StreamType::from_source(&Source::File(black_box(
                        LARGE_FILE_PATH.to_string(),
                    )))
                    .unwrap();
                    let chunks: Vec<_> = chunker.chunk_stream(stream).collect();
                    black_box(chunks)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_file_chunking_by_characters(c: &mut Criterion) {
    // Check if file exists
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        println!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        println!("   Skipping file benchmarks");
        return;
    }

    let file_size = fs::metadata(LARGE_FILE_PATH).map(|m| m.len()).unwrap_or(0);

    let mut group = c.benchmark_group("file_chunking_by_characters");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Bytes(file_size)); // Still measure bytes/sec for comparison

    // Benchmark different chunk sizes (in CHARACTERS)
    for chunk_size in [512, 1024, 2048, 4096, 8192] {
        let overlap = chunk_size / 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}k_chars", chunk_size / 1024)),
            &(chunk_size, overlap),
            |b, &(chunk_size, overlap)| {
                b.iter(|| {
                    let chunker =
                        CharactersChunker::new(black_box(chunk_size), black_box(overlap)).unwrap();
                    let stream = StreamType::from_source(&Source::File(black_box(
                        LARGE_FILE_PATH.to_string(),
                    )))
                    .unwrap();
                    let chunks: Vec<_> = chunker.chunk_stream(stream).collect();
                    black_box(chunks)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_file_chunking_by_bytes,
    benchmark_file_chunking_by_characters,
);
criterion_main!(benches);
