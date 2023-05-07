use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use include_dir::{include_dir, Dir};

use hanekawa_bencode;

const TORRENT_SAMPLES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/benches/samples/");

pub fn parse_torrents(c: &mut Criterion) {
    for sample in TORRENT_SAMPLES.files() {
        let name = sample.path().file_name().unwrap().to_string_lossy();

        let mut group = c.benchmark_group(name.clone());
        group.throughput(Throughput::BytesDecimal(sample.contents().len() as u64));

        group.bench_function("parse", |b| {
            b.iter(|| {
                hanekawa_bencode::parse(black_box(sample.contents())).unwrap();
            })
        });
    }
}

pub fn encode_torrents(c: &mut Criterion) {
    for sample in TORRENT_SAMPLES.files() {
        let name = sample.path().file_name().unwrap().to_string_lossy();

        let parsed = hanekawa_bencode::parse(sample.contents())
            .unwrap()
            .into_value();

        let mut group = c.benchmark_group(name.clone());
        group.throughput(Throughput::BytesDecimal(sample.contents().len() as u64));

        group.bench_function("encode", |b| {
            b.iter(|| {
                hanekawa_bencode::encode(&parsed);
            })
        });
    }
}

pub fn encode_torrents_serde(c: &mut Criterion) {
    for sample in TORRENT_SAMPLES.files() {
        let name = sample.path().file_name().unwrap().to_string_lossy();

        let mut group = c.benchmark_group(name.clone());
        group.throughput(Throughput::BytesDecimal(sample.contents().len() as u64));

        let parsed = hanekawa_bencode::parse(sample.contents()).unwrap();

        group.bench_function("encode(serde)", |b| {
            b.iter(|| {
                hanekawa_bencode::to_bytes(&parsed).unwrap();
            })
        });
    }
}

criterion_group!(
    benches,
    parse_torrents,
    encode_torrents,
    encode_torrents_serde
);
criterion_main!(benches);
