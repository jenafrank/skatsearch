extern crate skat_aug23;

use std::time::Instant;
use skat_aug23::traits::{Augen, StringConverter};
use skat_aug23::types::problem::Problem;
use skat_aug23::types::state::State;

mod problems;

fn assert_solution_transposition_table((p, s): (Problem, u8)) {
    let now = Instant::now();
    let res = Problem::search_with_problem_using_double_dummy_solver(p);
    assert_eq!(res.0, s);
    let elapsed = now.elapsed();
    println!("Transtable duration = {} µs",elapsed.as_micros());
    println!("NPS: {} kN", (res.1 as f32)/((elapsed.as_micros() as f32)/1e6)/1000f32);
}

fn assert_solution_all((p, s): (Problem, u8)) {
    assert_solution_transposition_table((p, s));
}

#[test]
fn one_trick_rank_in_one_suit() { assert_solution_all(problems::one_trick_rank_in_one_suit()); }

#[test]
fn one_trick_suit_wins_decl() { assert_solution_all(problems::one_trick_suit_wins_decl()); }

#[test]
fn one_trick_suit_wins_left() { assert_solution_all(problems::one_trick_suit_wins_left()); }

#[test]
fn one_trick_suit_wins_right() { assert_solution_all(problems::one_trick_suit_wins_right()); }

#[test]
fn two_tricks_suit_wins_decl() { assert_solution_all(problems::two_tricks_suit_wins_decl()); }

#[test]
fn two_tricks_suit_wins_left() { assert_solution_all(problems::two_tricks_suit_wins_left()); }

#[test]
fn two_tricks_suit_wins_right() { assert_solution_all(problems::two_tricks_suit_wins_right()); }

#[test]
fn two_tricks_forking_decl_all() { assert_solution_all(problems::two_tricks_forking_decl_all()); }

#[test]
fn two_tricks_forking_decl_part() { assert_solution_all(problems::two_tricks_forking_decl_part()); }

#[test]
fn two_tricks_forking_team_part() { assert_solution_all(problems::two_tricks_forking_team_part()); }

#[test]
fn two_tricks_not_allowed_to_trump() { assert_solution_all(problems::two_tricks_not_allowed_to_trump()); }

#[test]
fn five_tricks() { assert_solution_all(problems::five_tricks()); }

#[test]
fn six_tricks() { assert_solution_all(problems::six_tricks()); }

#[test]
fn seven_tricks() { assert_solution_all(problems::seven_tricks()); }

#[test]
fn eight_tricks() { assert_solution_all(problems::eight_tricks()); }

#[test]
fn ten_tricks() { assert_solution_all(problems::ten_tricks()); }

// #[test]
// fn ten_grand_hard() { assert_solution_all(problems::ten_grand_hard())}}

/// Checks playout capabilities. We do not have access to a principal variation.
/// Therefore, we play out a game to see sequence of moves w.r.t. best play.
#[test]
fn play_out () {
    let now = Instant::now();
    let pset = problems::ten_tricks();

    let res = Problem::get_playout(pset.0);

    for (i, el) in res.iter().flatten().enumerate() {

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
    println!("Final score: {}", pset.1);
    println!("TT duration playout = {} ms",elapsed.as_millis());
}

#[test]
fn allvalues () {
    let now = Instant::now();
    let pset = problems::ten_tricks();

    let mut problem = pset.0;
    let state = State::create_initial_state_from_problem(&problem);

    let res = problem.get_allvalues(&state);

    for el in res.iter().flatten() {
        let card = el.0;
        let follow_up_card = el.1;
        let value = el.2;

        println!("{} -> {} ({})", card.__str(), follow_up_card.__str(), value);
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

    let problem = pset.0;

    let res = Problem::get_allvalues_playout(problem);

    for (i, el) in res.iter().enumerate() {

        let card = el.0;
        let player = el.1;
        let pnts = el.2;
        let allvals = el.3;

        if i%3 != 2 {
            print!("{} {}    | ", player.str(), card.__str());
        } else {
            print!("{} {} {:2} | ", player.str(), card.__str(), pnts);
        }

        for el2 in allvals.iter().flatten() {
            print!("[{}, {} {}] ", el2.0.__str(), el2.1.__str(), el2.2);
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
    
    let mut problem = pset.0;
    let mut state = State::create_initial_state_from_problem(&problem);

    let start = Instant::now();
    let result = problem.search_if_declarer_is_winning(&mut state);
    let time = start.elapsed().as_micros();

    println!("Consumed time: {} µs",time);
    println!("Declarer is winning: {}", result);

    assert!(!result);
}

#[test]
pub fn all_skat_values () {
    let mut p = problems::ten_tricks().0;
    
    let start = Instant::now();
    let result = p.get_all_skat_values(false,false);
    let time = start.elapsed().as_micros();

    let mut vec = result.2.to_vec();
    vec.sort_by(|a,b| b.1.cmp(&a.1));

    println!("Consumed time: {} µs",time);
    println!();

    println!("All twelve cards:");
    let allcards: u32 = (!0u32) ^ p.left_cards_all ^ p.right_cards_all;
    let skat: u32 = (!0u32) ^ p.left_cards_all ^ p.right_cards_all ^ p.declarer_cards_all;
    println!("{} | {}", p.declarer_cards_all.__str() , skat.__str());
    println!();

    println!("One of best skat drueckungs found:");
    println!("{} {}",result.0.__str(), result.1.__str());
    println!();

    for el in vec {
        let skat_value = el.0.0.__get_value() + el.0.1.__get_value();
        println!("{} {} : {:3} + {:3} = {:3} | {}", el.0.0.__str(), el.0.1.__str(), el.1 - skat_value,
            skat_value, el.1, (allcards ^ el.0.0 ^ el.0.1).__str());
    }
}
