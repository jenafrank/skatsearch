use skat_aug23::types::game::Game;
use skat_aug23::types::player::Player;
use skat_aug23::types::problem::Problem;
use skat_aug23::traits::BitConverter;

pub fn one_trick_rank_in_one_suit() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("SA", "ST", "SK");
    (p, 25)
}

pub fn one_trick_suit_wins_decl() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("SA", "HA", "DA");
    (p, 33)
}

pub fn one_trick_suit_wins_left() -> (Problem, u8) {
    let p = Problem::create_farbe_left_problem("SA", "HA", "DA");
    (p, 0)
}

pub fn one_trick_suit_wins_right() -> (Problem, u8) {
    let p = Problem::create_farbe_right_problem("SA", "HA", "DA");
    (p, 0)
}

pub fn two_tricks_suit_wins_decl() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("SA S7", "HA D7", "DA H7");
    (p, 33)
}

pub fn two_tricks_suit_wins_left() -> (Problem, u8) {
    let p = Problem::create_farbe_left_problem("SA S7", "HA D7", "DA H7");
    (p, 0)
}

pub fn two_tricks_suit_wins_right() -> (Problem, u8) {
    let p = Problem::create_farbe_right_problem("SA S7", "HA D7", "DA H7");
    (p, 0)
}

pub fn two_tricks_forking_decl_all() -> (Problem, u8) {
    let p = Problem::create_farbe_right_problem("DJ CT", "HA DA", "CA CK");
    (p, 49)
}

pub fn two_tricks_forking_decl_part() -> (Problem, u8) {
    let p = Problem::create_farbe_right_problem("DJ CT", "HA DA", "HJ CA");
    (p, 24)
}

pub fn two_tricks_forking_team_part() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("DJ CT", "HA D7", "CA CK");
    (p, 6)
}

pub fn two_tricks_not_allowed_to_trump() -> (Problem, u8) {
    let p = Problem::create_farbe_right_problem("CJ HT", "HA H7", "HK H8");
    (p, 2)
}

pub fn five_tricks() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("CA SA HA ST HT", "CT SK SQ HK HQ", "CK S9 S8 H9 H8");
    (p, 81)
}

pub fn six_tricks() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("CJ CA SA HA ST HT", "SJ CT SK SQ HK HQ", "HJ CK S9 S8 H9 H8");
    (p, 64)
}

pub fn seven_tricks() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem("CJ CA SA HA ST HT DA", "SJ CT SK SQ HK HQ D7", "HJ CK S9 S8 H9 H8 D8");
    (p, 75)
}

pub fn eight_tricks() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem(
        "CJ CA C7 HA HT HK G8 G7",
        "SJ HJ G9 H9 H8 DA DT DK",
        "DJ CT HQ H7 DQ D9 D8 D7"
    );
    (p, 59)
}

pub fn ten_tricks() -> (Problem, u8) {
    let p = Problem::create_farbe_declarer_problem(
        "CJ CA C9 C8 C7 HA HT HK H7 S8",
        "SJ HJ SA ST SK S9 H9 H8 DA DT",
        "DJ CT CK CQ HQ S7 DQ D9 D8 D7"
    );

    assert_eq!(p.nr_of_cards, 30);

    (p, 59)
}

#[allow(dead_code)]
pub fn ten_grand_hard() -> (Problem, u8) {
    let p = Problem::create(
        "HJ DJ CA CT CK CQ C9 D9 D8 D7".__bit(),
        "CJ SJ SA ST SK SQ S9 S8 S7 DA".__bit(),
        "HA HT HK HQ H9 H8 H7 DT DK DQ".__bit(),
        Game::Grand,
        Player::Declarer
    );
    (p,38)
}

#[allow(dead_code)]
pub fn eight_grand_hard() -> (Problem, u8) {
    let p = Problem::create(
        "HJ DJ CA CT CK CQ C9 D9".__bit(),
        "CJ SJ SA ST SK SQ S9 S8".__bit(),
        "HA HT HK HQ H9 H8 H7 DT".__bit(),
        Game::Grand,
        Player::Declarer
    );
    (p,42)
}

#[allow(dead_code)]
pub fn nine_grand_hard() -> (Problem, u8) {
    let p = Problem::create(
        "HJ DJ CA CT CK CQ C9 D9 D8".__bit(),
        "CJ SJ SA ST SK SQ S9 S8 S7".__bit(),
        "HA HT HK HQ H9 H8 H7 DT DK".__bit(),
        Game::Grand,
        Player::Declarer
    );
    (p,39)
}

pub fn intermediate_state() -> (Problem, u8, u32, u32) {
    let p = Problem::create(
        "CA SA".__bit(),
        "CT ST".__bit(),
        "C9 S9".__bit(),
        Game::Farbe,
        Player::Declarer
    );
    (p, 42, "CA".__bit(), "CT".__bit())
}

pub fn bench_three_tricks()-> Problem {
    Problem::create_farbe_declarer_problem("DJ CT C7", "HA D8 D7", "CA CK C8")
}