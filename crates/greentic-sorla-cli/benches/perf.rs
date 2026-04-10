use criterion::{Criterion, criterion_group, criterion_main};

fn bench_example(c: &mut Criterion) {
    c.bench_function("wizard_schema_hot_path", |b| {
        b.iter(|| {
            // TODO: replace with real hot path once wizard generation is implemented.
            let schema = greentic_sorla_cli::default_schema();
            assert!(!schema.sections.is_empty());
        })
    });
}

criterion_group!(benches, bench_example);
criterion_main!(benches);
