use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use kiru::{ChunkerBuilder, ChunkerEnum, Source};
use std::fs;
use std::hint::black_box;
use std::time::Duration;

const LARGE_FILE_PATH: &str = "../test-data/realistic-5.0mb.txt";
const CHUNK_SIZE: usize = 4096;
const OVERLAP: usize = 512;
const CHANNEL_SIZE: usize = 1000;

fn prepare_sources(n: usize) -> Vec<Source> {
    (0..n)
        .map(|_| Source::File(LARGE_FILE_PATH.to_string()))
        .collect()
}

fn benchmark_bytes_chunking(c: &mut Criterion) {
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        println!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        println!("   Skipping parallel benchmarks");
        return;
    }

    let file_size = fs::metadata(LARGE_FILE_PATH).map(|m| m.len()).unwrap_or(0);

    let mut group = c.benchmark_group("bytes_chunking_parallel");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    // Serial single source
    group.throughput(Throughput::Bytes(file_size));
    group.bench_function("serial_single", |b| {
        let source = Source::File(LARGE_FILE_PATH.to_string());
        b.iter(|| {
            let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
                chunk_size: CHUNK_SIZE,
                overlap: OVERLAP,
            });
            let iter = chunker.on_source(source.clone()).unwrap();
            let chunks: Vec<_> = iter.collect();
            black_box(chunks);
        });
    });

    for n in [1, 10, 50, 100] {
        let sources = prepare_sources(n);
        let total_size = file_size * n as u64;
        group.throughput(Throughput::Bytes(total_size));

        // Serial multi
        group.bench_with_input(
            BenchmarkId::new("serial_multi", n),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let iter = chunker.on_sources(sources.clone()).unwrap();
                    let chunks: Vec<_> = iter.collect();
                    black_box(chunks);
                });
            },
        );

        // Par multi collect
        group.bench_with_input(
            BenchmarkId::new("par_multi_collect", n),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let chunks: Vec<_> = chunker.on_sources_par(sources.clone()).unwrap();
                    black_box(chunks);
                });
            },
        );

        // Par multi stream
        group.bench_with_input(
            BenchmarkId::new("par_multi_stream", n),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let iter = chunker
                        .on_sources_par_stream(sources.clone(), CHANNEL_SIZE)
                        .unwrap();
                    let chunks: Vec<_> = iter.collect();
                    black_box(chunks);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_characters_chunking(c: &mut Criterion) {
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        println!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        println!("   Skipping parallel benchmarks");
        return;
    }

    let file_size = fs::metadata(LARGE_FILE_PATH).map(|m| m.len()).unwrap_or(0);

    let mut group = c.benchmark_group("characters_chunking_parallel");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    // Serial single source
    group.throughput(Throughput::Bytes(file_size));
    group.bench_function("serial_single", |b| {
        let source = Source::File(LARGE_FILE_PATH.to_string());
        b.iter(|| {
            let chunker = ChunkerBuilder::by_characters(ChunkerEnum::Characters {
                chunk_size: CHUNK_SIZE,
                overlap: OVERLAP,
            });
            let iter = chunker.on_source(source.clone()).unwrap();
            let chunks: Vec<_> = iter.collect();
            black_box(chunks);
        });
    });

    for n in [1, 10, 50, 100] {
        let sources = prepare_sources(n);
        let total_size = file_size * n as u64;
        group.throughput(Throughput::Bytes(total_size));

        // Serial multi
        group.bench_with_input(
            BenchmarkId::new("serial_multi", n),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_characters(ChunkerEnum::Characters {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let iter = chunker.on_sources(sources.clone()).unwrap();
                    let chunks: Vec<_> = iter.collect();
                    black_box(chunks);
                });
            },
        );

        // Par multi collect
        group.bench_with_input(
            BenchmarkId::new("par_multi_collect", n),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_characters(ChunkerEnum::Characters {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let chunks: Vec<_> = chunker.on_sources_par(sources.clone()).unwrap();
                    black_box(chunks);
                });
            },
        );

        // Par multi stream
        group.bench_with_input(
            BenchmarkId::new("par_multi_stream", n),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_characters(ChunkerEnum::Characters {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let iter = chunker
                        .on_sources_par_stream(sources.clone(), CHANNEL_SIZE)
                        .unwrap();
                    let chunks: Vec<_> = iter.collect();
                    black_box(chunks);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_channel_size(c: &mut Criterion) {
    if !std::path::Path::new(LARGE_FILE_PATH).exists() {
        println!("⚠️  Large test file not found at: {}", LARGE_FILE_PATH);
        println!("   Skipping channel size benchmarks");
        return;
    }

    const N_SOURCES: usize = 50;

    let file_size = fs::metadata(LARGE_FILE_PATH).map(|m| m.len()).unwrap_or(0);
    let total_size = file_size * N_SOURCES as u64;

    let mut group = c.benchmark_group("channel_size_benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Bytes(total_size));

    let sources = prepare_sources(N_SOURCES);

    for channel_size in [100, 1000, 10000] {
        // Bytes chunking
        group.bench_with_input(
            BenchmarkId::new(format!("bytes_channel_{}", channel_size), N_SOURCES),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_bytes(ChunkerEnum::Bytes {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let iter = chunker
                        .on_sources_par_stream(sources.clone(), channel_size)
                        .unwrap();
                    let chunks: Vec<_> = iter.collect();
                    black_box(chunks);
                });
            },
        );

        // Characters chunking
        group.bench_with_input(
            BenchmarkId::new(format!("characters_channel_{}", channel_size), N_SOURCES),
            &sources,
            |b, sources| {
                b.iter(|| {
                    let chunker = ChunkerBuilder::by_characters(ChunkerEnum::Characters {
                        chunk_size: CHUNK_SIZE,
                        overlap: OVERLAP,
                    });
                    let iter = chunker
                        .on_sources_par_stream(sources.clone(), channel_size)
                        .unwrap();
                    let chunks: Vec<_> = iter.collect();
                    black_box(chunks);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_bytes_chunking,
    benchmark_characters_chunking,
    benchmark_channel_size
);
criterion_main!(benches);
