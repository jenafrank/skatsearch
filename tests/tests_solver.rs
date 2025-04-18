extern crate skat_aug23;

use std::time::Instant;
use skat_aug23::traits::{StringConverter, Augen};
use skat_aug23::types::solver::withskat::acceleration_mode::AccelerationMode;
use skat_aug23::types::solver::Solver;

mod problems;

#[ignore]
#[test]
fn solve_all_cards () {
    let now = Instant::now();
    let pset = problems::null_shrinked_1();
    let mut solver = Solver::new(pset.0, None);

    let res = solver.solve_all_cards(0, 120);

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

#[ignore]
#[test]
pub fn solve_win_10tricks () {
    let pset = problems::ten_tricks();
    
    let mut solver = Solver::new(pset.0, None);

    let start = Instant::now();
    let result = solver.solve_win_10tricks();
    let is_winning = result.declarer_wins;
    let time = start.elapsed().as_micros();

    println!("Consumed time: {} µs",time);
    println!("Declarer is winning: {}", is_winning);

    // The ten tricks would give 59 points, together with the 7 points from skat, this hand is won.
    assert!(is_winning);
}

#[ignore]
#[test]
pub fn solve_with_skat () {
    let (p, _) = problems::ten_tricks();
    
    let start = Instant::now();
    let result = Solver::solve_with_skat(
        p.left_cards(), 
        p.right_cards(),
        p.declarer_cards(), 
        p.game_type(), 
        p.start_player(),
        AccelerationMode::NotAccelerating
    );

    let time = start.elapsed().as_micros();
    let mut vec = result.all_skats;
    vec.sort_by(|a,b| b.value.cmp(&a.value));

    println!("Consumed time: {} µs",time);
    println!();

    println!("All twelve cards:");    
    let allcards: u32 = (!0u32) ^ p.left_cards() ^ p.right_cards();
    let skat: u32 = (!0u32) ^ p.left_cards() ^ p.right_cards() ^ p.declarer_cards();
    println!("{} | {}", p.declarer_cards().__str() , skat.__str());
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
