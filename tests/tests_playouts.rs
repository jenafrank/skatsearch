extern crate skat_aug23;

use std::time::Instant;
use skat_aug23::traits::StringConverter;
use skat_aug23::types::solver::Solver;

mod problems;

#[ignore]
#[test]
fn playout () {
    let now = Instant::now();
    let problem_set = problems::null_2();

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

#[ignore]
#[test]
fn playout_all_cards () {
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
