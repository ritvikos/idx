use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn get_test_corpus() -> Vec<String> {
    vec![
        "The quick brown fox jumps over the lazy dog.",
        "The quick brown fox.",
        "The quick brown fox jumps.",
        "The quick brown fox jumps over.",
        "The quick brown fox jumps over the lazy dog again.",
        "The lazy dog lies in the sun.",
        "The dog is lazy.",
        "Foxes are quick and brown.",
        "Foxes jump over lazy dogs.",
        "A fast brown fox leaps over lazy hounds.",
        "Speedy brown foxes jump over sluggish dogs.",
        "Foxes are cunning and quick.",
        "Dogs are loyal and lazy.",
        "A fox is quicker than a dog.",
        "Jumping foxes and sleeping dogs.",
        "The sun shines on the lazy dog.",
        "Quick thinking foxes outsmart lazy dogs.",
        "The fox and the hound.",
        "A quick brown fox outpaces a lazy brown dog.",
        "Clever foxes evade the lazy dogs.",
    ]
    .iter()
    .map(|document| document.to_string())
    .collect::<Vec<_>>()
}

fn token_entry_old(tokens: &mut Vec<String>) {
    tokens.iter_mut().for_each(|token| {
        let _ = token.to_string();
    });
}

fn token_entry_new(tokens: &mut Vec<String>) {
    tokens.iter_mut().for_each(|token| {
        let _ = std::mem::take(token);
    });
}

fn bench_token_entry_conversion(c: &mut Criterion) {
    let mut tokens = black_box(get_test_corpus());

    c.bench_function("token-string-conversion", |b| {
        b.iter(|| token_entry_old(&mut tokens))
    });

    c.bench_function("token-mem-take", |b| {
        b.iter(|| token_entry_new(&mut tokens))
    });
}

criterion_group!(benches, bench_token_entry_conversion);
criterion_main!(benches);
