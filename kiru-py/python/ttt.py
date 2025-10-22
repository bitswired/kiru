from time import perf_counter

from kiru import Chunker

CHUNK_SIZE = 4096
OVERLAP = 128
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
    "glob://../test-data/*.txt",
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


def a() -> None:
    start = perf_counter()
    chunker = Chunker.by_bytes(chunk_size=CHUNK_SIZE, overlap=OVERLAP).on_sources_par(
        SOURCES,
        10_000,
    )
    chunks = chunker.all()
    print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
    size = 0
    for chunk in chunks:
        size += len(chunk)
    print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


def b() -> None:
    start = perf_counter()
    chunker = Chunker.by_bytes(chunk_size=CHUNK_SIZE, overlap=OVERLAP).on_sources(
        SOURCES,
    )
    chunks = chunker.all()
    print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
    size = 0
    for chunk in chunks:
        size += len(chunk)
    print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


def c() -> None:
    start = perf_counter()
    chunker = Chunker.by_characters(
        chunk_size=CHUNK_SIZE, overlap=OVERLAP
    ).on_sources_par(
        SOURCES,
        10_000,
    )
    chunks = chunker.all()
    print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
    size = 0
    for chunk in chunks:
        size += len(chunk)
    print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


def d() -> None:
    start = perf_counter()
    chunker = Chunker.by_characters(chunk_size=CHUNK_SIZE, overlap=OVERLAP).on_sources(
        SOURCES,
    )
    chunks = chunker.all()
    print(f"Elapsed time: {perf_counter() - start:.4f} seconds")
    size = 0
    for chunk in chunks:
        size += len(chunk)
    print(f"Total size of chunks: {size / 1024 / 1024:.2f} MB")


a()
b()
c()
d()
