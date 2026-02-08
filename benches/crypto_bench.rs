use age::x25519;
use dugout::bench::{Age, Cipher};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Generate a payload of given size.
fn generate_payload(size: usize) -> String {
    "x".repeat(size)
}

/// Benchmark encrypt/decrypt roundtrip with varying payload sizes.
fn bench_encrypt_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_decrypt");
    group.sample_size(50);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let cipher = Age;
    let sizes = [32, 256, 1024, 4096, 16384];

    for size in sizes {
        let payload = generate_payload(size);
        let identity = x25519::Identity::generate();
        let recipients = vec![identity.to_public()];

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("roundtrip", format!("{}B", size)),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let encrypted = cipher
                        .encrypt(black_box(payload), black_box(&recipients))
                        .unwrap();
                    let decrypted = cipher
                        .decrypt(black_box(&encrypted), black_box(&identity))
                        .unwrap();
                    black_box(decrypted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark encryption only.
fn bench_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt");
    group.sample_size(50);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let cipher = Age;
    let sizes = [32, 256, 1024, 4096, 16384];

    for size in sizes {
        let payload = generate_payload(size);
        let identity = x25519::Identity::generate();
        let recipients = vec![identity.to_public()];

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("age", format!("{}B", size)),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let encrypted = cipher
                        .encrypt(black_box(payload), black_box(&recipients))
                        .unwrap();
                    black_box(encrypted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark decryption only with pre-encrypted data.
fn bench_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt");
    group.sample_size(50);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let cipher = Age;
    let sizes = [32, 256, 1024, 4096, 16384];
    let identity = x25519::Identity::generate();
    let recipients = vec![identity.to_public()];

    for size in sizes {
        let payload = generate_payload(size);
        let encrypted = cipher.encrypt(&payload, &recipients).unwrap();

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("age", format!("{}B", size)),
            &encrypted,
            |b, encrypted| {
                b.iter(|| {
                    let decrypted = cipher
                        .decrypt(black_box(encrypted), black_box(&identity))
                        .unwrap();
                    black_box(decrypted);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark multi-recipient encryption scaling.
fn bench_recipient_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("recipient_scaling");
    group.sample_size(30);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    let cipher = Age;
    let payload = generate_payload(256);
    let recipient_counts = [1, 3, 5, 10];

    for count in recipient_counts {
        let recipients: Vec<_> = (0..count)
            .map(|_| x25519::Identity::generate().to_public())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("encrypt_256B", format!("{}_recipients", count)),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let encrypted = cipher
                        .encrypt(black_box(payload), black_box(&recipients))
                        .unwrap();
                    black_box(encrypted);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encrypt_decrypt,
    bench_encrypt,
    bench_decrypt,
    bench_recipient_scaling,
);
criterion_main!(benches);
