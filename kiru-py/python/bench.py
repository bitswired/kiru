import itertools as it
import json
import subprocess
import time
import tracemalloc
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Tuple

import pandas as pd
from bench_plot import plot_benchmark_results
from kiru import Chunker
from langchain.text_splitter import CharacterTextSplitter
from tqdm import tqdm


def rust_native_chunking(
    strategy: str,  # "bytes" or "chars"
    source_type: str,  # "file" or "string"
    path: str,
    chunk_size: int,
    overlap: int,
) -> Tuple[float, float]:
    """
    Run pure Rust benchmark via subprocess.

    Returns:
        Tuple[float, float]: (elapsed_time_seconds, memory_usage_mb)
        Note: memory_usage_mb is always 0 as we can't measure it from subprocess
    """
    benchmark_bin = Path("../target/release/benchmark")

    if not benchmark_bin.exists():
        raise FileNotFoundError(
            f"Benchmark binary not found at {benchmark_bin}.\n"
            "Run: cd kiru-core && cargo build --release --bin benchmark"
        )

    result = subprocess.run(
        [
            str(benchmark_bin),
            strategy,
            source_type,
            path,
            str(chunk_size),
            str(overlap),
        ],
        capture_output=True,
        text=True,
        # check=True,
        timeout=300,  # 5 minute timeout
    )

    bench_result = json.loads(result.stdout)

    res = (
        bench_result["elapsed_secs"],
        float("nan"),
    )  # Can't measure memory from subprocess

    return res


def ensure_rust_benchmark_built():
    """Ensure the Rust benchmark binary is built."""
    kiru_core = Path(__file__).parent.parent.parent / "kiru-core"
    benchmark_bin = kiru_core / "target" / "release" / "benchmark"

    if benchmark_bin.exists():
        print("✓ Rust benchmark binary already exists")
        return

    print("Building Rust benchmark binary...")
    try:
        subprocess.run(
            ["cargo", "build", "--release", "--bin", "benchmark"],
            cwd=kiru_core,
            check=True,
            capture_output=True,
        )
        print("✓ Rust benchmark binary built successfully")
    except subprocess.CalledProcessError as e:
        print(f"Failed to build Rust benchmark binary: {e.stderr.decode()}")
        raise


def kiru_native_chunking_bytes_file(path: str, chunk_size: int, overlap: int):
    """Run kiru byte-based chunking from file using Rust native benchmark."""
    return rust_native_chunking(
        strategy="bytes",
        source_type="file",
        path=path,
        chunk_size=chunk_size,
        overlap=overlap,
    )


def kiru_native_chunking_chars_file(path: str, chunk_size: int, overlap: int):
    """Run kiru character-based chunking from file using Rust native benchmark."""
    return rust_native_chunking(
        strategy="chars",
        source_type="file",
        path=path,
        chunk_size=chunk_size,
        overlap=overlap,
    )


def kiru_native_chunking_bytes_string(text: str, chunk_size: int, overlap: int):
    """Run kiru byte-based chunking from string using Rust native benchmark."""
    return rust_native_chunking(
        strategy="bytes",
        source_type="string",
        path=text,
        chunk_size=chunk_size,
        overlap=overlap,
    )


def kiru_native_chunking_chars_string(text: str, chunk_size: int, overlap: int):
    """Run kiru character-based chunking from string using Rust native benchmark."""
    return rust_native_chunking(
        strategy="chars",
        source_type="string",
        path=text,
        chunk_size=chunk_size,
        overlap=overlap,
    )


def kiru_chunking_bytes_file(path: str, chunk_size: int, overlap: int) -> None:
    """Run kiru byte-based chunking from file."""
    chunker = Chunker.by_bytes(chunk_size=chunk_size, overlap=overlap).on_file(path)
    for chunk in chunker:
        _ = len(chunk)


def kiru_chunking_chars_file(path: str, chunk_size: int, overlap: int) -> None:
    """Run kiru character-based chunking from file."""
    chunker = Chunker.by_characters(chunk_size=chunk_size, overlap=overlap).on_file(
        path
    )
    for chunk in chunker:
        _ = len(chunk)


def kiru_chunking_bytes_string(text: str, chunk_size: int, overlap: int) -> None:
    """Run kiru byte-based chunking from string."""
    chunker = Chunker.by_bytes(chunk_size=chunk_size, overlap=overlap).on_string(text)
    for chunk in chunker:
        _ = len(chunk)


def kiru_chunking_chars_string(text: str, chunk_size: int, overlap: int) -> None:
    """Run kiru character-based chunking from string."""
    chunker = Chunker.by_characters(chunk_size=chunk_size, overlap=overlap).on_string(
        text
    )
    for chunk in chunker:
        _ = len(chunk)


def langchain_chunking_string(text: str, chunk_size: int, overlap: int) -> None:
    """Run LangChain chunking from string."""
    splitter = CharacterTextSplitter(
        chunk_size=chunk_size,
        chunk_overlap=overlap,
        separator="",
        is_separator_regex=False,
        keep_separator=True,
        strip_whitespace=False,
    )
    splitter.split_text(text)


def langchain_chunking_file(path: str, chunk_size: int, overlap: int) -> None:
    """Run LangChain chunking from string."""
    splitter = CharacterTextSplitter(
        chunk_size=chunk_size,
        chunk_overlap=overlap,
        separator="",
        is_separator_regex=False,
        keep_separator=True,
        strip_whitespace=False,
    )
    with open(path, "r", encoding="utf-8") as file:
        data = file.read()
    splitter.split_text(data)


def benchmark_function(func: Callable, *args) -> Tuple[float, float]:
    """Measure execution time and memory usage of a function."""
    tracemalloc.start()
    start_time = time.perf_counter()

    try:
        func(*args)
    except MemoryError:
        print(f"{func.__name__} encountered OOM error")
        return float("inf"), float("inf")
    except Exception as e:
        print(f"{func.__name__} encountered error: {e}")
        return float("inf"), float("inf")

    end_time = time.perf_counter()
    current, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()

    return end_time - start_time, peak / (1024 * 1024)  # Convert bytes to MB


@dataclass
class BenchmarkConfig:
    library: str
    strategy: str
    source: str
    file_path: str
    file_size: int
    chunk_size: int
    overlap: int
    run: int


def create_benchmark_config() -> list[BenchmarkConfig]:
    """Create benchmark configuration."""
    chunk_sizes = [1024, 4096, 8192, 16384]
    overlaps = [0]
    runs = list(range(3))

    file_paths = ["../test-data/realistic-1.0mb.txt"]
    file_sizes = [int(1 * 1024 * 1024)]

    source = ["string", "file"]

    kiru_strategies = ["chars", "bytes"]
    langchain_strategies = ["chars"]

    def base():
        return it.product(file_paths, file_sizes, chunk_sizes, overlaps, runs)

    def l():
        return it.product(["langchain"], langchain_strategies, source)

    def k():
        return it.product(["kiru"], kiru_strategies, source)

    def kn():
        return it.product(["kiru-native"], kiru_strategies, source)

    kiru_configs = it.product(k(), base())
    langchain_configs = it.product(l(), base())
    kiru_native_configs = it.product(kn(), base())

    configs_gen = it.chain(kiru_native_configs, kiru_configs, langchain_configs)

    res = [
        BenchmarkConfig(
            library=lib_strat[0],
            source=lib_strat[2],
            strategy=lib_strat[1],
            file_path=params[0],
            file_size=params[1],
            chunk_size=params[1],
            overlap=params[2],
            run=params[3],
        )
        for lib_strat, params in configs_gen
    ]

    return res


def run_benchmarks(configs: list[BenchmarkConfig]):
    # load all file in memory for the string benchmarks
    files_paths = set(c.file_path for c in configs)
    file_content_map = {x: open(x, "r", encoding="utf-8").read() for x in files_paths}

    res = []

    pbar = tqdm(configs)

    func_map = {
        ("kiru-native", "chars", "string"): kiru_native_chunking_chars_string,
        ("kiru-native", "chars", "file"): kiru_native_chunking_chars_file,
        ("kiru-native", "bytes", "string"): kiru_native_chunking_bytes_string,
        ("kiru-native", "bytes", "file"): kiru_native_chunking_bytes_file,
        ("kiru", "chars", "string"): kiru_chunking_chars_string,
        ("kiru", "chars", "file"): kiru_chunking_chars_file,
        ("kiru", "bytes", "string"): kiru_chunking_bytes_string,
        ("kiru", "bytes", "file"): kiru_chunking_bytes_file,
        ("langchain", "chars", "string"): langchain_chunking_string,
        ("langchain", "chars", "file"): langchain_chunking_file,
    }

    for config in pbar:
        pbar.set_description_str(
            f"{config.library} | {config.strategy} | {config.source} | {config.file_size // (1024 * 1024)}MB | chunk: {config.chunk_size} | overlap: {config.overlap} | run: {config.run}"
        )

        func = func_map[(config.library, config.strategy, config.source)]

        if config.source == "string":
            x = file_content_map[config.file_path]
        else:
            x = config.file_path

        if config.library == "kiru-native":
            (exec_time, mem_usage) = func(x, config.chunk_size, config.overlap)
        else:
            exec_time, mem_usage = benchmark_function(
                func, x, config.chunk_size, config.overlap
            )

        throughput = (
            config.file_size / exec_time / (1024 * 1024) if exec_time > 0 else 0
        )

        res.append(
            {
                "library": config.library,
                "strategy": config.strategy,
                "source": config.source,
                "file_size_mb": config.file_size / (1024 * 1024),
                "chunk_size": config.chunk_size,
                "overlap": config.overlap,
                "run": config.run,
                "time_s": exec_time,
                "memory_mb": mem_usage,
                "throughput_mb_s": throughput,
            }
        )

    return res


def main():
    ensure_rust_benchmark_built()

    configs = create_benchmark_config()

    df = pd.DataFrame(run_benchmarks(configs))

    print(df)

    a = df.groupby(
        ["library", "strategy", "source", "file_size_mb", "chunk_size", "overlap"]
    ).agg(
        {
            "time_s": ["mean", "std"],
            "memory_mb": ["mean", "std"],
            "throughput_mb_s": ["mean", "std"],
        }
    )
    print(a)

    df.to_csv("benchmark_results.csv", index=False)

    plot_benchmark_results(df)


if __name__ == "__main__":
    main()
