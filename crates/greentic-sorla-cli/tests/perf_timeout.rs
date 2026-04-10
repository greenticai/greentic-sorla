use std::time::{Duration, Instant};

#[test]
fn workload_should_finish_quickly() {
    let start = Instant::now();

    // TODO: replace with a real operation from the critical path.
    let schema = greentic_sorla_cli::default_schema();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "workload too slow: {:?}",
        elapsed
    );
    assert!(!schema.sections.is_empty());
}
