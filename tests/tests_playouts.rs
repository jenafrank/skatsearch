extern crate skat_aug23;

use skat_aug23::extensions::playout::{playout, playout_all_cards};
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::traits::StringConverter;
use std::time::Instant;

mod problems;

#[ignore]
#[test]
fn playout_test() {
    let now = Instant::now();
    let problem_set = problems::null_2();

    let mut solver = SkatEngine::new(problem_set.0, None);

    let result = playout(&mut solver);

    for (i, el) in result.iter().enumerate() {
        if i % 3 == 0 {
            println!();
            println!("D: {}", el.declarer_cards.__str());
            println!("L: {}", el.left_cards.__str());
            println!("R: {}", el.right_cards.__str());
            println!();
        }

        println!(
            "{} _ {} _ {} ({}) | #: {}, ab: {} ms: {}",
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
    println!("TT duration playout = {} ms", elapsed.as_millis());
}

#[ignore]
#[test]
fn playout_all_cards_test() {
    let now = Instant::now();
    let pset = problems::ten_tricks();

    let mut solver = SkatEngine::new(pset.0, None);

    let res = playout_all_cards(&mut solver);

    for (i, el) in res.iter().enumerate() {
        let card = el.best_card;
        let player = el.player;
        let pnts = el.augen_declarer;
        let allvals = &el.all_cards;

        if i % 3 != 2 {
            print!("{} {}    | ", player.str(), card.__str());
        } else {
            print!("{} {} {:2} | ", player.str(), card.__str(), pnts);
        }

        for el2 in allvals.results.iter() {
            print!("[{}, {} {}] ", el2.0.__str(), el2.1.__str(), el2.2);
        }

        println!();

        if i % 3 == 2 {
            println!();
        }
    }

    let elapsed = now.elapsed();

    println!();
    println!("Final score: {}", pset.1);
    println!("TT duration playout = {} ms", elapsed.as_millis());
}
