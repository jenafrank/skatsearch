extern crate skat_aug23;
use skat_aug23::extensions::solver::solve;
use skat_aug23::skat::engine::SkatEngine;

mod problems;

#[test]
fn bench_ten_tricks() {
    let (problem, _) = problems::ten_tricks();
    let mut solver = SkatEngine::new(problem, None);

    println!("Starting benchmark ten_tricks...");
    let result = solve(&mut solver);

    println!("Benchmark Result:");
    println!("Iters: {}", result.counters.iters);
    // println!("Collisions: {}", result.counters.collisions); // old counters had collisions?
    // SkatEngine Counters has: iters, writes, cuts, ...
    // Assuming simple counters output
    println!("Value: {}", result.best_value);
}
