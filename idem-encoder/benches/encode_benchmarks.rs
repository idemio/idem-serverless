use criterion::{criterion_group, criterion_main, Criterion};
use idem_encoder::encoders::java_script_encoder;
use idem_encoder::{OldEncoder, Mode};

fn java_script_encode_benches(c: &mut Criterion) {
    let chunk = "0123456789abcdef\u{0009}123456789abcdef\u{12345}12345\u{0009}789abc\u{0008}ef0123456789abcde\u{00ff}";
    let mut full_block = String::with_capacity(16384);
    for _ in 0..254 {
        full_block.push_str(chunk);
    }
    full_block.push_str("&");
    c.bench_function("js - encoderv1", |b| b.iter(|| {
        let encoder = OldEncoder::new(Mode::Attribute, true);
        encoder.encode(std::hint::black_box(&full_block))
    }));
    c.bench_function("js - encoderv2", |b| b.iter(|| {
        let encoder = java_script_encoder();
        encoder.encode(std::hint::black_box(&full_block))
    }));


}

criterion_group!(benches, java_script_encode_benches);
criterion_main!(benches);