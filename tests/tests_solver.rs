extern crate skat_aug23;

use skat_aug23::extensions::skat_solving::{solve_with_skat, AccelerationMode};
use skat_aug23::extensions::solver::{solve_all_cards, solve_win};
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::traits::{Points, StringConverter};
use std::time::Instant;

mod problems;

#[ignore]
#[test]
fn solve_all_cards_test() {
    let now = Instant::now();
    let pset = problems::null_shrinked_1();
    let mut solver = SkatEngine::new(pset.0, None);

    let res = solve_all_cards(&mut solver, 0, 120);

    for el in res.results {
        println!("{} -> {} ({})", el.0.__str(), el.1.__str(), el.2);
    }

    let elapsed = now.elapsed();

    println!();
    println!("Final score: {}", pset.1);
    println!("TT duration playout = {} ms", elapsed.as_millis());
}

#[ignore]
#[test]
pub fn solve_win_10tricks() {
    let pset = problems::ten_tricks();
    let (mut problem, _expected_points) = pset;

    // Logic to replicate old solve_win_10tricks behavior:
    // It adjusted threshold based on skat value.
    let skat_value = problem.get_skat().points();
    let threshold = 61 - skat_value;
    problem.set_threshold_upper(threshold);

    let mut solver = SkatEngine::new(problem, None);

    let start = Instant::now();
    let result = solve_win(&mut solver);
    let is_winning = result.declarer_wins;
    let time = start.elapsed().as_micros();

    println!("Consumed time: {} µs", time);
    println!("Declarer is winning: {}", is_winning);

    // The ten tricks would give 59 points, together with the 7 points from skat, this hand is won.
    assert!(is_winning);
}

#[ignore]
#[test]
pub fn solve_with_skat_test() {
    let (p, _) = problems::ten_tricks();

    let start = Instant::now();
    let result = solve_with_skat(
        p.left_cards,
        p.right_cards,
        p.declarer_cards,
        p.game_type,
        p.start_player,
        AccelerationMode::NotAccelerating,
    );

    let time = start.elapsed().as_micros();
    let mut vec = result.all_skats;
    vec.sort_by(|a, b| b.value.cmp(&a.value));

    println!("Consumed time: {} µs", time);
    println!();

    println!("All twelve cards:");
    let allcards: u32 = (!0u32) ^ p.left_cards ^ p.right_cards;
    let skat: u32 = (!0u32) ^ p.left_cards ^ p.right_cards ^ p.declarer_cards;
    println!("{} | {}", p.declarer_cards.__str(), skat.__str());
    println!();

    let best_skat = result.best_skat.unwrap();
    println!("One of best skat drueckungs found:");
    println!("{} {}", best_skat.skat_card_1, best_skat.skat_card_2);
    println!();

    for el in vec {
        let skat_value = el.skat_card_1.points() + el.skat_card_2.points();
        println!(
            "{} {} : {:3} + {:3} = {:3} | {}",
            el.skat_card_1.__str(),
            el.skat_card_2.__str(),
            el.value - skat_value,
            skat_value,
            el.value,
            (allcards ^ el.skat_card_1 ^ el.skat_card_2).__str()
        );
    }
}
