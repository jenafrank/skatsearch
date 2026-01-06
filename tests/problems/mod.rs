#![allow(dead_code)]

use skat_aug23::skat::builder::GameContextBuilder;
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::defs::Game;
use skat_aug23::skat::defs::Player;
use skat_aug23::traits::BitConverter;

pub fn one_trick_rank_in_one_suit() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA")
        .cards(Player::Left, "ST")
        .cards(Player::Right, "SK")
        .turn(Player::Declarer)
        .build();
    (p, 25)
}

pub fn one_trick_suit_wins_decl() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA")
        .cards(Player::Left, "HA")
        .cards(Player::Right, "DA")
        .turn(Player::Declarer)
        .build();
    (p, 33)
}

pub fn one_trick_suit_wins_left() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA")
        .cards(Player::Left, "HA")
        .cards(Player::Right, "DA")
        .turn(Player::Left)
        .build();
    (p, 0)
}

pub fn one_trick_suit_wins_right() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA")
        .cards(Player::Left, "HA")
        .cards(Player::Right, "DA")
        .turn(Player::Right)
        .build();
    (p, 0)
}

pub fn two_tricks_suit_wins_decl() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA S7")
        .cards(Player::Left, "HA D7")
        .cards(Player::Right, "DA H7")
        .turn(Player::Declarer)
        .build();
    (p, 33)
}

pub fn two_tricks_suit_wins_left() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA S7")
        .cards(Player::Left, "HA D7")
        .cards(Player::Right, "DA H7")
        .turn(Player::Left)
        .build();
    (p, 0)
}

pub fn two_tricks_suit_wins_right() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA S7")
        .cards(Player::Left, "HA D7")
        .cards(Player::Right, "DA H7")
        .turn(Player::Right)
        .build();
    (p, 0)
}

pub fn two_tricks_forking_decl_all() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "DJ CT")
        .cards(Player::Left, "HA DA")
        .cards(Player::Right, "CA CK")
        .turn(Player::Right)
        .build();
    (p, 49)
}

pub fn two_tricks_forking_decl_part() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "DJ CT")
        .cards(Player::Left, "HA DA")
        .cards(Player::Right, "HJ CA")
        .turn(Player::Right)
        .build();
    (p, 24)
}

pub fn two_tricks_forking_team_part() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "DJ CT")
        .cards(Player::Left, "HA D7")
        .cards(Player::Right, "CA CK")
        .turn(Player::Declarer)
        .build();
    (p, 6)
}

pub fn two_tricks_not_allowed_to_trump() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "CJ HT")
        .cards(Player::Left, "HA H7")
        .cards(Player::Right, "HK H8")
        .turn(Player::Right)
        .build();
    (p, 2)
}

pub fn five_tricks() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "CA SA HA ST HT")
        .cards(Player::Left, "CT SK SQ HK HQ")
        .cards(Player::Right, "CK S9 S8 H9 H8")
        .turn(Player::Declarer)
        .build();
    (p, 81)
}

pub fn six_tricks() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "CJ CA SA HA ST HT")
        .cards(Player::Left, "SJ CT SK SQ HK HQ")
        .cards(Player::Right, "HJ CK S9 S8 H9 H8")
        .turn(Player::Declarer)
        .build();
    (p, 64)
}

pub fn seven_tricks() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "CJ CA SA HA ST HT DA")
        .cards(Player::Left, "SJ CT SK SQ HK HQ D7")
        .cards(Player::Right, "HJ CK S9 S8 H9 H8 D8")
        .turn(Player::Declarer)
        .build();
    (p, 75)
}

pub fn eight_tricks() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "CJ CA C7 HA HT HK S8 S7")
        .cards(Player::Left, "SJ HJ S9 H9 H8 DA DT DK")
        .cards(Player::Right, "DJ CT HQ H7 DQ D9 D8 D7")
        .turn(Player::Declarer)
        .build();
    (p, 59)
}

pub fn ten_tricks() -> (GameContext, u8) {
    let p = GameContextBuilder::new_farbspiel()
        .cards(Player::Declarer, "CJ CA C9 C8 C7 HA HT HK H7 S8")
        .cards(Player::Left, "SJ HJ SA ST SK S9 H9 H8 DA DT")
        .cards(Player::Right, "DJ CT CK CQ HQ S7 DQ D9 D8 D7")
        .turn(Player::Declarer)
        .build();

    // assert_eq!(p.number_of_cards(), 30); // Not available in GameContext directly?

    (p, 59)
}

pub fn null_1() -> (GameContext, u8) {
    let p = GameContextBuilder::new_null()
        .cards_all(
            "SJ C9 C7 ST S9 S8 H9 DT D9 D8",
            "HJ DJ CA CT CK C8 SA SK HA H8",
            "CJ CQ SQ HT HK HQ H7 DA DK DQ",
        )
        .turn(Player::Declarer)
        .build();
    (p, 1)
}

pub fn null_2() -> (GameContext, u8) {
    let p = GameContextBuilder::new_null()
        .cards_all(
            "SJ C9 C7 ST S9 S8 H9 DT D9 D8",
            "DJ CA CQ SA SK SQ HT H8 H7 DK",
            "CJ HJ CT CK C8 HA HK HQ DA DQ",
        )
        .turn(Player::Declarer)
        .build();
    (p, 0)
}

pub fn null_3() -> (GameContext, u8) {
    let p = GameContextBuilder::new_null()
        .cards_all(
            "SJ C9 C7 ST S9 S8 H9 DT D9 D8",
            "DJ CA CQ SA SK SQ HT H8 H7 DK",
            "CJ HJ CT CK C8 HA HK HQ DA DQ",
        )
        .turn(Player::Declarer)
        .build();
    (p, 0)
}

pub fn null_shrinked_1() -> (GameContext, u8) {
    let p = GameContextBuilder::new_null()
        .cards_all("S9 S7 H8 D7", "S8 H9 D8 D9", "SA SK H7 DJ")
        .turn(Player::Right)
        .build();
    (p, 1)
}

pub fn null_1_debug() -> (GameContext, u8) {
    let p = GameContextBuilder::new_null()
        .cards_all("D9 CJ", "D8 SJ", "D7 DT")
        .turn(Player::Declarer)
        .build();
    (p, 1)
}
