# Benchmarks

Benchmarks run with [Criterion](https://github.com/bheisler/criterion.rs) on a 4-core AMD VPS (16GB RAM, Ubuntu Linux 6.8).

## Encryption (age, x25519)

Single recipient:

| Payload | Encrypt | Decrypt | Roundtrip | Throughput (encrypt) |
|---------|---------|---------|-----------|---------------------|
| 32B | 105µs | 135µs | 258µs | 289 KiB/s |
| 256B | 116µs | 149µs | 263µs | 2.1 MiB/s |
| 1KB | 116µs | 146µs | 264µs | 8.5 MiB/s |
| 4KB | 113µs | 154µs | 271µs | 34.5 MiB/s |
| 16KB | 138µs | 195µs | 355µs | 113 MiB/s |

## Recipient Scaling

256B payload, varying team size:

| Recipients | Encrypt Time |
|-----------|-------------|
| 1 | 109µs |
| 3 | 322µs |
| 5 | 542µs |
| 10 | 978µs |

Encryption scales linearly with recipient count. A 10-person team adds less than 1ms per secret.

## Methodology

- Criterion with 50 samples per benchmark
- Release profile with `opt-level = 3`
- Results are median times from 50 samples
- Run locally: `cargo bench`
