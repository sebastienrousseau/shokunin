<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Bounded-Memory Batch Compilation

For sites with 10,000+ pages, bounded-memory batch compilation caps
peak memory by processing content in fixed-size batches. It is a
batch pipeline — content is compiled batch-by-batch to disk, not
incrementally streamed — so peak memory is bounded by the batch
size, not the site size. (The file name `streaming.md` is kept for
link stability; the mechanism was historically named after streaming
but has always been batch-based.)

## When Batch Mode Activates

Batch mode activates when:

- `--max-memory MB` flag is set, OR
- Content exceeds the default batch size (512 MB budget)

## Memory Budget

| Setting | Value |
|---------|-------|
| Default budget | 512 MB |
| Estimated per page | 64 KB |
| Default batch size | ~8,000 pages |

## CLI Usage

```sh
# Use default 512 MB budget
ssg --content ./content --output ./public

# Constrain to 256 MB for CI environments
ssg --content ./content --output ./public --max-memory 256
```

## How It Works

1. Content files are divided into batches based on the memory budget
2. Each batch is compiled independently to a temporary directory
3. After all batches, a merge pass unifies cross-page artefacts
4. Temporary batch directories are cleaned up automatically

## Performance

Bounded-memory batch compilation adds ~10% overhead vs. in-memory
compilation but enables sites that would otherwise exceed available
RAM.

Note: the `lol_html`-based HTML rewriting stage is genuinely
streaming (it rewrites HTML as a byte stream without building a full
DOM); the term "streaming" in this codebase is reserved for that
path and for chunked file I/O.
