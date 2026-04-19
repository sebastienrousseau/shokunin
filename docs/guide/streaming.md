<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Streaming Compilation

For sites with 10,000+ pages, streaming compilation caps peak memory
by processing content in batches.

## When Streaming Activates

Streaming activates when:

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

Streaming adds ~10% overhead vs. in-memory compilation but enables
sites that would otherwise exceed available RAM.
