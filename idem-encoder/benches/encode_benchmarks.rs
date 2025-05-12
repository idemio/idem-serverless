use criterion::{Criterion, criterion_group, criterion_main};
use idem_encoder::encoder::{Encoder, ValidAsciiRange};
use idem_encoder::java_script_encoder::{
    JavaScriptEncoder, JavaScriptEncoderMode, JavaScriptEncoderSettings,
};

fn java_script_encode_benches(c: &mut Criterion) {
    let bench_data1 = std::fs::read_to_string("./benches/data/benchmark-data-1.txt").unwrap();

    // Javascript Block Encoding
    let encoder = JavaScriptEncoder::create(
        '\\',
        ValidAsciiRange::ASCII,
        JavaScriptEncoderSettings {
            mode: JavaScriptEncoderMode::Block,
        },
    );
    c.bench_function("js - encoder", |b| {
        b.iter(|| encoder.encode(std::hint::black_box(&bench_data1)))
    });
}

criterion_group!(benches, java_script_encode_benches);
criterion_main!(benches);
