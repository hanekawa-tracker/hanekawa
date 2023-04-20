use criterion::{black_box, criterion_group, criterion_main, Criterion};
use include_dir::{include_dir, Dir};

use hanekawa::bencode;

const TORRENT_SAMPLES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/benches/samples/");

pub fn parse_torrents(c: &mut Criterion) {
    for sample in TORRENT_SAMPLES.files() {
        let name = sample.path()
            .file_name()
            .unwrap()
            .to_string_lossy();

        c.bench_function(&format!("parse {}", name), |b| {
            b.iter(|| {
                bencode::parse(black_box(sample.contents())).unwrap();
            })
        });
    }
}

pub fn encode_torrents(c: &mut Criterion) {
    for sample in TORRENT_SAMPLES.files() {
        let name = sample.path()
            .file_name()
            .unwrap()
            .to_string_lossy();

        let parsed = bencode::parse(sample.contents())
            .unwrap();

        c.bench_function(&format!("encode {}", name), |b| {
            b.iter(|| {
                bencode::encode(&parsed);
            })
        });
    }
}

pub fn encode_torrents_serde(c: &mut Criterion) {
    for sample in TORRENT_SAMPLES.files() {
        let name = sample.path()
            .file_name()
            .unwrap()
            .to_string_lossy();

        let parsed = bencode::parse(sample.contents())
            .unwrap();

        c.bench_function(&format!("encode(serde) {}", name), |b| {
            b.iter(|| {
                bencode::to_bytes(&parsed)
                    .unwrap();
            })
        });
    }
}

criterion_group!(benches, parse_torrents, encode_torrents, encode_torrents_serde);
criterion_main!(benches);
