#![feature(test)]

use skat_aug23::{traits::{BitConverter, Augen, StringConverter}, consts::bitboard::ALLCARDS};
use skat_aug23::types::{player::Player, game::Game};
use skat_aug23::uncertain::{uncertain_problem::{UncertainProblem, Facts}, estimator::Estimator};

extern crate test;
extern crate skat_aug23;

#[ignore]
#[test]
fn test_uproblem_eight_cards() {

    let my_cards = "CJ HJ CA CT SA S7 HT H7".__bit();
    let other_cards = "SJ DJ CK CQ ST SK SQ S9 S8 DA DT DK DQ D9 D8 D7".__bit(); 
    assert!(my_cards & other_cards == 0);
    let all_cards = my_cards ^ other_cards;

    let uproblem = UncertainProblem {
        game_type: Game::Farbe,
        my_player: Player::Declarer,
        next_player: Player::Declarer,
        my_cards: my_cards,
        cards_on_table: 0,
        all_cards: all_cards,
        active_suit: 0,
        upper_bound_of_null_window: all_cards.__get_value() / 2,
        facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
    };

    let estimator = Estimator::new(uproblem, 1000);
    let (probability, _) = estimator.estimate_win(false);

    println!("Probability of win: {}", probability);
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_grand() {

    let my_cards = "CJ SJ DJ HJ C8 C7 HK HQ DA DK".__bit();
    let skat_cards = "S7 D7".__bit(); 

    let other_cards = ALLCARDS ^ my_cards ^ skat_cards;
    assert!(my_cards & other_cards == 0);
    let all_cards = my_cards ^ other_cards;

    let uproblem = UncertainProblem {
        game_type: Game::Grand,
        my_player: Player::Declarer,
        next_player: Player::Declarer,
        my_cards: my_cards,
        cards_on_table: 0,
        all_cards: all_cards,
        active_suit: 0,
        upper_bound_of_null_window: 61,
        facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
    };

    let estimator = Estimator::new(uproblem, 100);
    let (probability, _) = estimator.estimate_win(true);

    println!("Probability of win: {}", probability);
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_null() {

    let my_cards = "SJ S9 S7 HJ H9 H7 CJ C9 C7 D8".__bit();
    let skat_cards = "CA SA".__bit(); 

    let other_cards = ALLCARDS ^ my_cards ^ skat_cards;
    assert!(my_cards & other_cards == 0);
    let all_cards = my_cards ^ other_cards;

    let uproblem = UncertainProblem {
        game_type: Game::Null,
        my_player: Player::Declarer,
        next_player: Player::Left,
        my_cards: my_cards,
        cards_on_table: 0,
        all_cards: all_cards,
        active_suit: 0,
        upper_bound_of_null_window: 1,
        facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
    };

    let estimator = Estimator::new(uproblem, 100);
    let (probability, _) = estimator.estimate_win(true);

    println!("Probability of win: {}", probability);
}

#[ignore]
#[test]
fn test_uproblem_ten_cards_null_all_cards() {

    let my_cards = "SJ S9 S7 HJ H9 H7 CK CJ C9 C7".__bit();
    let skat_cards = "CA SA".__bit(); 

    let other_cards = ALLCARDS ^ my_cards ^ skat_cards;
    assert!(my_cards & other_cards == 0);
    let all_cards = my_cards ^ other_cards;

    let uproblem = UncertainProblem {
        game_type: Game::Null,
        my_player: Player::Declarer,
        next_player: Player::Declarer,
        my_cards: my_cards,
        cards_on_table: 0,
        all_cards: all_cards,
        active_suit: 0,
        upper_bound_of_null_window: 1,
        facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
    };

    let estimator = Estimator::new(uproblem, 100);
    let res = estimator.estimate_probability_of_all_cards(true);

    let mut sorted_entries = res.iter().collect::<Vec<_>>();
    sorted_entries.sort_by(|a,b| b.1.partial_cmp(a.1).unwrap());

    for (key, value) in sorted_entries {
        println!("{}: {:.2}", (*key).__str(), value);        }
    
}