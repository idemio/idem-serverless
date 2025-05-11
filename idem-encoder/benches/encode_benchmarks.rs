use criterion::{criterion_group, criterion_main, Criterion};
use idem_encoder::encoders::java_script_encoder;
use idem_encoder::java_script_encoder::{JavaScriptEncoder, Mode};

fn java_script_encode_benches(c: &mut Criterion) {
    let bench_data = std::fs::read_to_string("./benches/data/benchmark-data-1.txt").unwrap();
    c.bench_function("js - encoderv1", |b| b.iter(|| {
        let encoder = JavaScriptEncoder::new(Mode::Attribute, true);
        encoder.encode(std::hint::black_box(&bench_data))
    }));
    c.bench_function("js - encoderv2", |b| b.iter(|| {
        let encoder = java_script_encoder();
        encoder.encode(std::hint::black_box(&bench_data))
    }));


}

criterion_group!(benches, java_script_encode_benches);
criterion_main!(benches);