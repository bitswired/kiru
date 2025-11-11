// kiru-core/src/bin/benchmark.rs

use kiru::{ChunkerBuilder, Source};
use serde::Serialize;
use std::env;
use std::time::Instant;

#[derive(Serialize)]
struct BenchmarkResult {
    elapsed_secs: f64,
    num_chunks: usize,
    total_bytes: usize,
    throughput_mb_s: f64,
}

#[derive(Serialize)]
struct BenchmarkError {
    error: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 6 {
        let error = BenchmarkError {
            error: "Usage: benchmark <strategy> <source_type> <path> <chunk_size> <overlap>"
                .to_string(),
        };
        eprintln!("{}", serde_json::to_string(&error).unwrap());
        std::process::exit(1);
    }

    let strategy = &args[1]; // "bytes" or "chars"
    let source_type = &args[2]; // "string" or "file" or "http" or "glob"
    let path = &args[3];
    let chunk_size: usize = match args[4].parse() {
        Ok(v) => v,
        Err(e) => {
            let error = BenchmarkError {
                error: format!("Invalid chunk_size: {}", e),
            };
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            std::process::exit(1);
        }
    };
    let overlap: usize = match args[5].parse() {
        Ok(v) => v,
        Err(e) => {
            let error = BenchmarkError {
                error: format!("Invalid overlap: {}", e),
            };
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            std::process::exit(1);
        }
    };

    let result = run_benchmark(strategy, source_type, path, chunk_size, overlap);

    match result {
        Ok(bench_result) => {
            println!("{}", serde_json::to_string(&bench_result).unwrap());
        }
        Err(e) => {
            let error = BenchmarkError {
                error: format!("Benchmark failed: {}", e),
            };
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            std::process::exit(1);
        }
    }
}

fn run_benchmark(
    strategy: &str,
    source_type: &str,
    path: &str,
    chunk_size: usize,
    overlap: usize,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    // Parse the source based on source_type
    let source = match source_type {
        "file" => Source::File(path.to_string()),
        "http" | "https" => Source::Http(path.to_string()),
        "string" => Source::Text(path.to_string()),
        _ => {
            return Err(format!(
                "Invalid source_type '{}'. Use 'file', 'string', 'http', 'text', or 'glob'",
                source_type
            )
            .into());
        }
    };

    // Create the chunker using ChunkerBuilder
    match strategy {
        "bytes" => {
            let chunker = ChunkerBuilder::by_bytes(chunk_size, overlap)?;
            bench_with(chunker, source)
        }
        "chars" => {
            let chunker = ChunkerBuilder::by_characters(chunk_size, overlap)?;
            bench_with(chunker, source)
        }
        _ => Err(format!("Invalid strategy '{}'. Use 'bytes' or 'chars'", strategy).into()),
    }
}

// Generic benchmarking body specialized for the concrete chunker type.
fn bench_with<C>(
    chunker: kiru::ChunkerWithStrategy<C>,
    source: Source,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>>
where
    C: kiru::Chunker,
{
    let start = Instant::now();
    let mut num_chunks = 0usize;
    let mut total_bytes = 0usize;

    let iterator = chunker.on_source(source)?;
    for chunk in iterator {
        num_chunks += 1;
        total_bytes += chunk.len();
        std::hint::black_box(chunk.len());
    }

    let elapsed_secs = start.elapsed().as_secs_f64();
    let throughput_mb_s = (total_bytes as f64) / (1024.0 * 1024.0) / elapsed_secs;

    Ok(BenchmarkResult {
        elapsed_secs,
        num_chunks,
        total_bytes,
        throughput_mb_s,
    })
}
