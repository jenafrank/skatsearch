#![feature(test)]

use skat_aug23::{traits::StringConverter, uncertain::uproblem_builder::UProblemBuilder};
use skat_aug23::types::player::Player;
use skat_aug23::uncertain::estimator::Estimator;

extern crate test;
extern crate skat_aug23;

#[ignore]
#[test]
fn test_uproblem_eight_cards() {  

    let uproblem = UProblemBuilder::new_farbspiel()
    .cards(Player::Declarer, "CJ HJ CA CT SA S7 HT H7")
    .remaining_cards("SJ DJ CK CQ ST SK SQ S9 S8 DA DT DK DQ D9 D8 D7")
    .threshold_half()
    .build();

    let estimator = Estimator::new(uproblem, 1000);
    let (probability, _) = estimator.estimate_win(false);

    println!("Probability of win: {}", probability);
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_grand() {

    let uproblem = UProblemBuilder::new_grand()
    .cards(Player::Declarer, "CJ SJ DJ HJ C8 C7 HK HQ DA DK")
    .skat_cards("S7 D7")
    .threshold_half()
    .build();

    assert!(uproblem.threshold_upper() == 61);

    let estimator = Estimator::new(uproblem, 100);
    let (probability, _) = estimator.estimate_win(true);

    println!("Probability of win: {}", probability);
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_null() {   

    let uproblem = UProblemBuilder::new_null()
    .cards(Player::Declarer, "SJ S9 S7 HJ H9 H7 CJ C9 C7 D8")
    .skat_cards("CA SA")
    .turn(Player::Left)
    .build();

    let estimator = Estimator::new(uproblem, 100);
    let (probability, _) = estimator.estimate_win(true);

    println!("Probability of win: {}", probability);
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_null_all_cards() {

    let uproblem = UProblemBuilder::new_null()
    .cards(Player::Declarer, "SJ S9 S7 HJ H9 H7 CK CJ C9 C7")
    .skat_cards("CA SA")
    .build();

    let estimator = Estimator::new(uproblem, 100);
    let res = estimator.estimate_probability_of_all_cards(true);

    show_probabilites(res);    
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_grand_all_cards() {

    let uproblem = UProblemBuilder::new_grand()
    .cards(Player::Declarer, "SJ DJ CA CT CK CQ C9 C8 DA H7")
    .skat_cards("D7 S7")
    .build();

    let estimator = Estimator::new(uproblem, 100);
    let res = estimator.estimate_probability_of_all_cards(true);

    show_probabilites(res);
}

fn show_probabilites(probabilities: Vec<(u32, f32)>) {    
    for (key, value) in probabilities {
        println!("{}: {:.2}", key.__str(), value);        
    }
}