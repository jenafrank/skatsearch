extern crate skat_aug23;

use std::time::Instant;

use skat_aug23::extensions::solver::solve_win;
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::traits::StringConverter;

pub mod problems;

#[test]
pub fn null_1() {
    assert_solution_null(problems::null_1());
}

#[test]
pub fn null_2() {
    assert_solution_null(problems::null_2());
}

#[test]
pub fn null_3() {
    assert_solution_null(problems::null_3());
}

#[test]
pub fn null_shrinked_1() {
    assert_solution_null(problems::null_shrinked_1());
}

#[test]
pub fn null_1_debug() {
    assert_solution_null(problems::null_1_debug());
}

fn assert_solution_null((p, s): (GameContext, u8)) {
    let now = Instant::now();
    let mut solver = SkatEngine::new(p, None);
    let res = solve_win(&mut solver);

    let elapsed = now.elapsed();
    println!("Best card: {}", res.best_card.__str());
    println!("Transtable duration = {} µs", elapsed.as_micros());
    println!(
        "NPS: {} kN",
        (res.counters.iters as f32) / ((elapsed.as_micros() as f32) / 1e6) / 1000f32
    );

    assert_eq!(res.declarer_wins, s == 0);
}
