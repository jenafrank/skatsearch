extern crate skat_aug23;
use skat_aug23::types::solver::Solver;
mod problems;

#[test]
fn bench_ten_tricks() {
    let (problem, _) = problems::ten_tricks();
    let mut solver = Solver::new(problem, None);

    println!("Starting benchmark ten_tricks...");
    let result = solver.solve_classic();

    println!("Benchmark Result:");
    println!("Iters: {}", result.counters.iters);
    println!("Collisions: {}", result.counters.collisions);
    println!("Value: {}", result.best_value);
}
