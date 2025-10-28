use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use kiru::{BytesChunker, CharactersChunker, Chunker};
use std::fs;
use std::hint::black_box;
use std::time::Duration;

const LARGE_FILE_PATH: &str = "../test-data/realistic-5.0mb.txt";

fn benchmark_string_chunking_by_bytes(c: &mut Criterion) {
    // Check if file exists and load content
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        eprintln!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        eprintln!("   Skipping string_chunking_by_bytes benchmark");
        return;
    }

    let content = match fs::read_to_string(LARGE_FILE_PATH) {
        Ok(content) => content,
        Err(e) => {
            println!("⚠️  Failed to read file: {}", e);
            return;
        }
    };

    let content_size = content.len() as u64;

    let mut group = c.benchmark_group("string_chunking_by_bytes");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Bytes(content_size));

    // Benchmark different chunk sizes (in BYTES)
    for chunk_size in [512, 1024, 2048, 4096, 8192] {
        let overlap = chunk_size / 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb_chunk", chunk_size / 1024)),
            &(chunk_size, overlap),
            |b, &(chunk_size, overlap)| {
                b.iter_batched(
                    || content.clone(),
                    |content| {
                        let chunker =
                            BytesChunker::new(black_box(chunk_size), black_box(overlap)).unwrap();
                        let chunks: Vec<_> = chunker.chunk_string(black_box(content)).collect();
                        black_box(chunks)
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

fn benchmark_string_chunking_by_characters(c: &mut Criterion) {
    // Check if file exists and load content
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        eprintln!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        eprintln!("   Create it with: head -c 100M /dev/urandom | base64 > test-data/100mb.txt");
        eprintln!("   Skipping string_chunking_by_characters benchmark");
        return;
    }

    let content = match fs::read_to_string(LARGE_FILE_PATH) {
        Ok(content) => content,
        Err(e) => {
            println!("⚠️  Failed to read file: {}", e);
            return;
        }
    };

    let content_size = content.len() as u64;

    let mut group = c.benchmark_group("string_chunking_by_characters");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));
    group.throughput(Throughput::Bytes(content_size)); // Still measure bytes/sec for comparison

    // Benchmark different chunk sizes (in CHARACTERS)
    for chunk_size in [512, 1024, 2048, 4096, 8192] {
        let overlap = chunk_size / 10;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}k_chars", chunk_size / 1024)),
            &(chunk_size, overlap),
            |b, &(chunk_size, overlap)| {
                b.iter_batched(
                    || content.clone(),
                    |content| {
                        let chunker =
                            CharactersChunker::new(black_box(chunk_size), black_box(overlap))
                                .unwrap();
                        let chunks: Vec<_> = chunker.chunk_string(black_box(content)).collect();
                        black_box(chunks)
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_string_chunking_by_bytes,
    benchmark_string_chunking_by_characters,
);
criterion_main!(benches);
