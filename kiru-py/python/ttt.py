from time import perf_counter

import pandas as pd
from kiru import Chunker
from tqdm import tqdm

CHUNK_SIZE = 4096
OVERLAP = 0
# SOURCES = ["glob://../test-data/*.txt"] * 10
SOURCES = [
    "https://en.wikipedia.org/wiki/World_War_II",
    "https://en.wikipedia.org/wiki/United_States",
    "https://en.wikipedia.org/wiki/China",
    "https://en.wikipedia.org/wiki/India",
    "https://en.wikipedia.org/wiki/Christianity",
    "https://en.wikipedia.org/wiki/Islam",
    "https://en.wikipedia.org/wiki/Byzantine_Empire",
    "https://en.wikipedia.org/wiki/Ancient_Rome",
    "https://en.wikipedia.org/wiki/French_Revolution",
    "https://en.wikipedia.org/wiki/Industrial_Revolution",
    "https://en.wikipedia.org/wiki/Soviet_Union",
    "https://en.wikipedia.org/wiki/British_Empire",
    "https://en.wikipedia.org/wiki/Ottoman_Empire",
    "https://en.wikipedia.org/wiki/European_Union",
    "https://en.wikipedia.org/wiki/Catholic_Church",
    "https://en.wikipedia.org/wiki/New_York_City",
    "https://en.wikipedia.org/wiki/London",
    "https://en.wikipedia.org/wiki/American_Civil_War",
    "https://en.wikipedia.org/wiki/Nazi_Germany",
    "https://en.wikipedia.org/wiki/Cold_War",
]


bytes_chunker = Chunker.by_bytes(chunk_size=CHUNK_SIZE, overlap=OVERLAP)
chars_chunker = Chunker.by_characters(chunk_size=CHUNK_SIZE, overlap=OVERLAP)


def compute(iterator_fn: callable, name: str, runs: int = 1):
    data = []
    for run in tqdm(list(range(runs)), leave=False):
        start = perf_counter()
        chunks = list(iterator_fn())
        elapsed = perf_counter() - start
        size = sum(len(chunk.encode("utf-8")) for chunk in chunks)
        data.append(
            {
                "run": run,
                "elapsed": elapsed,
                "size": size / 1024 / 1024,
                "throughput": size / 1024 / 1024 / elapsed,
            }
        )
    df = pd.DataFrame(data)
    avg_elapsed = df["elapsed"].mean()
    avg_size = df["size"].mean()
    avg_throughput = df["throughput"].mean()

    print(f"--- {name} ---")
    print(f"Elapsed time: {avg_elapsed:.4f} seconds")
    print(f"Total size of chunks: {avg_size:.2f} MB")
    print(f"Throughput: {avg_throughput:.2f} MB/s")
    print("--------------------------------------------")
    print()


# sources = ["../test-data/realistic-1.0mb.txt"]
# compute(
#     lambda: bytes_chunker.on_sources(["file://../test-data/realistic-1.0mb.txt"]),
#     "bytes on_sources",
#     100,
# )
# compute(
#     lambda: bytes_chunker.on_file("../test-data/realistic-1.0mb.txt"),
#     "bytes on_file",
#     100,
# )
# compute(
#     lambda: bytes_chunker.on_sources_par(
#         ["file://../test-data/realistic-1.0mb.txt"], 10_000
#     ),
#     "bytes on_sources_par",
#     100,
# )


glob = "glob://../test-data/realistic-*"
compute(
    lambda: bytes_chunker.on_sources(
        [*SOURCES],
    ),
    "bytes on_sources glob",
    5,
)

compute(
    lambda: bytes_chunker.on_sources_par([*SOURCES], 10_000),
    "bytes on_sources_par glob",
    5,
)


# def a() -> None:
#     start = perf_counter()
#     chunker = Chunker.by_bytes(chunk_size=CHUNK_SIZE, overlap=OVERLAP).on_sources_par(
#         SOURCES,
#         10_000,
#     )
#     chunks = chunker.all()
#     print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
#     size = 0
#     for chunk in chunks:
#         size += len(chunk)
#     print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


# def b() -> None:
#     start = perf_counter()
#     chunker = Chunker.by_bytes(chunk_size=CHUNK_SIZE, overlap=OVERLAP).on_sources(
#         SOURCES,
#     )
#     chunks = chunker.all()
#     print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
#     size = 0
#     for chunk in chunks:
#         size += len(chunk)
#     print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


# def c() -> None:
#     start = perf_counter()
#     chunker = Chunker.by_characters(
#         chunk_size=CHUNK_SIZE, overlap=OVERLAP
#     ).on_sources_par(
#         SOURCES,
#         10_000,
#     )
#     chunks = chunker.all()
#     print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
#     size = 0
#     for chunk in chunks:
#         size += len(chunk)
#     print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


# def d() -> None:
#     start = perf_counter()
#     chunker = Chunker.by_characters(chunk_size=CHUNK_SIZE, overlap=OVERLAP).on_sources(
#         SOURCES,
#     )
#     chunks = chunker.all()
#     print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
#     size = 0
#     for chunk in chunks:
#         size += len(chunk)
#     print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


# a()
# b()
# c()
# d()
