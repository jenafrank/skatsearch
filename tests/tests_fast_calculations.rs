extern crate skat_aug23;

use skat_aug23::extensions::solver::solve_double_dummy;
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::engine::SkatEngine;
use std::time::Instant;

mod problems;

#[test]
fn one_trick_rank_in_one_suit() {
    assert_solution(problems::one_trick_rank_in_one_suit());
}

#[test]
fn one_trick_suit_wins_decl() {
    assert_solution(problems::one_trick_suit_wins_decl());
}

#[test]
fn one_trick_suit_wins_left() {
    assert_solution(problems::one_trick_suit_wins_left());
}

#[test]
fn one_trick_suit_wins_right() {
    assert_solution(problems::one_trick_suit_wins_right());
}

#[test]
fn two_tricks_suit_wins_decl() {
    assert_solution(problems::two_tricks_suit_wins_decl());
}

#[test]
fn two_tricks_suit_wins_left() {
    assert_solution(problems::two_tricks_suit_wins_left());
}

#[test]
fn two_tricks_suit_wins_right() {
    assert_solution(problems::two_tricks_suit_wins_right());
}

#[test]
fn two_tricks_forking_decl_all() {
    assert_solution(problems::two_tricks_forking_decl_all());
}

#[test]
fn two_tricks_forking_decl_part() {
    assert_solution(problems::two_tricks_forking_decl_part());
}

#[test]
fn two_tricks_forking_team_part() {
    assert_solution(problems::two_tricks_forking_team_part());
}

#[test]
fn two_tricks_not_allowed_to_trump() {
    assert_solution(problems::two_tricks_not_allowed_to_trump());
}

#[test]
fn five_tricks() {
    assert_solution(problems::five_tricks());
}

#[test]
fn six_tricks() {
    assert_solution(problems::six_tricks());
}

#[test]
fn seven_tricks() {
    assert_solution(problems::seven_tricks());
}

#[test]
fn eight_tricks() {
    assert_solution(problems::eight_tricks());
}

#[test]
fn ten_tricks() {
    assert_solution(problems::ten_tricks());
}

/// Checks playout capabilities. We do not have access to a principal variation.
/// Therefore, we play out a game to see sequence of moves w.r.t. best play.

fn assert_solution((p, s): (GameContext, u8)) {
    let now = Instant::now();
    let mut solver = SkatEngine::new(p, None);
    let res = solve_double_dummy(&mut solver, 0, 120, 5);

    assert_eq!(res.best_value, s);
    let elapsed = now.elapsed();
    println!("Transtable duration = {} µs", elapsed.as_micros());
    println!(
        "NPS: {} kN",
        (res.counters.iters as f32) / ((elapsed.as_micros() as f32) / 1e6) / 1000f32
    );
}
