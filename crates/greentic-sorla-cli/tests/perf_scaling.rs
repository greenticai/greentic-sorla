use std::time::{Duration, Instant};

fn run_workload(threads: usize) -> Duration {
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            std::thread::spawn(|| {
                // TODO: replace with real workload once the compiler pipeline exists.
                let schema = greentic_sorla_cli::default_schema();
                schema.sections.len()
            })
        })
        .collect();

    for handle in handles {
        let _: usize = handle.join().expect("thread should finish");
    }

    start.elapsed()
}

#[test]
fn scaling_should_not_degrade_badly() {
    let t1 = run_workload(1);
    let t4 = run_workload(4);
    let t8 = run_workload(8);

    assert!(
        t4 <= t1.mul_f64(12.0),
        "4 threads slower than expected: t1={:?}, t4={:?}",
        t1,
        t4
    );

    assert!(
        t8 <= t4.mul_f64(12.0),
        "8 threads slower than expected: t4={:?}, t8={:?}",
        t4,
        t8
    );
}
