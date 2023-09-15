#![feature(test)]

extern crate test;
extern crate skat_aug23;

use std::time::Instant;
use skat_aug23::traits::{StringConverter, Augen};
use skat_aug23::types::problem::Problem;
use skat_aug23::types::solver::Solver;

mod problems;

fn assert_solution((p, s): (Problem, u8)) {
    let now = Instant::now();
    let mut solver = Solver::create(p);    
    let res = solver.solve_double_dummy();

    assert_eq!(res.best_value, s);
    let elapsed = now.elapsed();
    println!("Transtable duration = {} µs",elapsed.as_micros());
    println!("NPS: {} kN", (res.counters.iters as f32)/((elapsed.as_micros() as f32)/1e6)/1000f32);
}

#[test]
fn one_trick_rank_in_one_suit() { assert_solution(problems::one_trick_rank_in_one_suit()); }

#[test]
fn one_trick_suit_wins_decl() { assert_solution(problems::one_trick_suit_wins_decl()); }

#[test]
fn one_trick_suit_wins_left() { assert_solution(problems::one_trick_suit_wins_left()); }

#[test]
fn one_trick_suit_wins_right() { assert_solution(problems::one_trick_suit_wins_right()); }

#[test]
fn two_tricks_suit_wins_decl() { assert_solution(problems::two_tricks_suit_wins_decl()); }

#[test]
fn two_tricks_suit_wins_left() { assert_solution(problems::two_tricks_suit_wins_left()); }

#[test]
fn two_tricks_suit_wins_right() { assert_solution(problems::two_tricks_suit_wins_right()); }

#[test]
fn two_tricks_forking_decl_all() { assert_solution(problems::two_tricks_forking_decl_all()); }

#[test]
fn two_tricks_forking_decl_part() { assert_solution(problems::two_tricks_forking_decl_part()); }

#[test]
fn two_tricks_forking_team_part() { assert_solution(problems::two_tricks_forking_team_part()); }

#[test]
fn two_tricks_not_allowed_to_trump() { assert_solution(problems::two_tricks_not_allowed_to_trump()); }

#[test]
fn five_tricks() { assert_solution(problems::five_tricks()); }

#[test]
fn six_tricks() { assert_solution(problems::six_tricks()); }

#[test]
fn seven_tricks() { assert_solution(problems::seven_tricks()); }

#[test]
fn eight_tricks() { assert_solution(problems::eight_tricks()); }

#[test]
fn ten_tricks() { assert_solution(problems::ten_tricks()); }

// #[test]
// fn ten_grand_hard() { assert_solution_all(problems::ten_grand_hard())}}

/// Checks playout capabilities. We do not have access to a principal variation.
/// Therefore, we play out a game to see sequence of moves w.r.t. best play.
#[test]
fn play_out () {
    let now = Instant::now();
    let problem_set = problems::ten_tricks();

    let mut solver = Solver::create(problem_set.0);

    let result = solver.playout();

    for (i, el) in result.iter().enumerate() {

        if i % 3 == 0 {
            println!();
            println!("D: {}", el.declarer_cards.__str());
            println!("L: {}", el.left_cards.__str());
            println!("R: {}", el.right_cards.__str());
            println!();
        }

        println!("{} _ {} _ {} ({}) | #: {}, ab: {} ms: {}",
                 el.player.str(),
                 el.card.__str(),
                 el.card,
                 el.augen_declarer,
                 el.cnt_iters,
                 el.cnt_breaks,
                 el.time
        );

    }

    let elapsed = now.elapsed();

    println!();
    println!("Final score: {}", problem_set.1);
    println!("TT duration playout = {} ms",elapsed.as_millis());
}

#[test]
fn allvalues () {
    let now = Instant::now();
    let pset = problems::ten_tricks();
    let mut solver = Solver::create(pset.0);

    let res = solver.solve_all_cards();

    for el in res.card_list {
        println!("{} -> {} ({})", 
            el.investigated_card.__str(), 
            el.best_follow_up_card.__str(), 
            el.value);
    }

    let elapsed = now.elapsed();

    println!();
    println!("Final score: {}", pset.1);
    println!("TT duration playout = {} ms",elapsed.as_millis());
}

#[test]
fn allvalues_playout () {
    let now = Instant::now();
    let pset = problems::ten_tricks();

    let mut solver = Solver::create(pset.0);

    let res = solver.playout_all_cards();

    for (i, el) in res.iter().enumerate() {

        let card = el.best_card;
        let player = el.player;
        let pnts = el.augen_declarer;
        let allvals = &el.all_cards;

        if i%3 != 2 {
            print!("{} {}    | ", player.str(), card.__str());
        } else {
            print!("{} {} {:2} | ", player.str(), card.__str(), pnts);
        }

        for el2 in &allvals.card_list {
            print!("[{}, {} {}] ", el2.investigated_card.__str(), el2.best_follow_up_card.__str(), el2.value);
        }

        println!();

        if i%3 == 2 {
            println!();
        }
    }

    let elapsed = now.elapsed();

    println!();
    println!("Final score: {}", pset.1);
    println!("TT duration playout = {} ms",elapsed.as_millis());
}

#[test]
pub fn search_if_winning () {
    let pset = problems::ten_tricks();
    
    let mut solver = Solver::create(pset.0);

    let start = Instant::now();
    let result = solver.solve_win_10tricks();
    let is_winning = result.declarer_wins;
    let time = start.elapsed().as_micros();

    println!("Consumed time: {} µs",time);
    println!("Declarer is winning: {}", is_winning);

    // The ten tricks would give 59 points, together with the 7 points from skat, this hand is won.
    assert!(is_winning);
}

#[test]
pub fn all_skat_values () {
    let mut solver = Solver::create(problems::ten_tricks().0);
    
    let start = Instant::now();
    let result = solver.solve_with_skat(false,false);
    let time = start.elapsed().as_micros();

    let mut vec = result.all_skats;
    vec.sort_by(|a,b| b.value.cmp(&a.value));

    println!("Consumed time: {} µs",time);
    println!();

    println!("All twelve cards:");
    let p = &solver.problem;
    let allcards: u32 = (!0u32) ^ p.left_cards_all ^ p.right_cards_all;
    let skat: u32 = (!0u32) ^ p.left_cards_all ^ p.right_cards_all ^ p.declarer_cards_all;
    println!("{} | {}", p.declarer_cards_all.__str() , skat.__str());
    println!();

    let best_skat = result.best_skat.unwrap();
    println!("One of best skat drueckungs found:");
    println!("{} {}", best_skat.skat_card_1, best_skat.skat_card_2);
    println!();

    for el in vec {        
        let skat_value = el.skat_card_1.__get_value() + el.skat_card_2.__get_value();
        println!("{} {} : {:3} + {:3} = {:3} | {}", 
        el.skat_card_1.__str(), 
        el.skat_card_2.__str(), 
        el.value - skat_value,
        skat_value, 
        el.value, 
        (allcards ^ el.skat_card_1 ^ el.skat_card_2).__str());
    }
}
