use crate::traits::{Augen, BitConverter};

use super::*;

impl Problem {

    pub fn create(
        declarer_cards_all: u32,
        left_cards_all: u32,
        right_cards_all: u32,
        game_type: Game,
        start_player: Player,
    ) -> Problem {
        
        let allcards = declarer_cards_all | left_cards_all | right_cards_all;

        Problem {
            declarer_cards_all,
            left_cards_all,
            right_cards_all,
            game_type,
            start_player,
            trick_cards: 0,
            trick_suit: 0,
            points_to_win: 0,

            augen_total: allcards.__get_value(),
            nr_of_cards: allcards.__get_number_of_bits(),
        }
    }

    pub fn create_farbe_declarer_problem(
        declarer_cards_all: &str,
        left_cards_all: &str,
        right_cards_all: &str,
    ) -> Problem 
    {
        Problem::create(
            declarer_cards_all.__bit(),
            left_cards_all.__bit(),
            right_cards_all.__bit(),
            Game::Farbe,
            Player::Declarer,
        )
    }

    pub fn create_farbe_left_problem(
        declarer_cards_all: &str,
        left_cards_all: &str,
        right_cards_all: &str,
    ) -> Problem {
        Problem::create(
            declarer_cards_all.__bit(),
            left_cards_all.__bit(),
            right_cards_all.__bit(),
            Game::Farbe,
            Player::Left,
        )
    }

    pub fn create_farbe_right_problem(
        declarer_cards_all: &str,
        left_cards_all: &str,
        right_cards_all: &str,
    ) -> Problem {
        Problem::create(
            declarer_cards_all.__bit(),
            left_cards_all.__bit(),
            right_cards_all.__bit(),
            Game::Farbe,
            Player::Right,
        )
    }

    pub fn new() -> Self {
        Problem {
            declarer_cards_all: 0u32,
            left_cards_all: 0u32,
            right_cards_all: 0u32,
            game_type: Game::Farbe,
            start_player: Player::Declarer,
            augen_total: 0u8,
            nr_of_cards: 0u8,
            trick_cards: 0,
            trick_suit: 0,
            points_to_win: 0
        }
    }

}
