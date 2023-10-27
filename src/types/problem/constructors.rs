use crate::traits::BitConverter;

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
        assert!(declarer_cards_all & left_cards_all == 0);
        assert!(declarer_cards_all & right_cards_all == 0);
        assert!(left_cards_all & right_cards_all == 0);
        
        Problem {
            declarer_cards: declarer_cards_all,
            left_cards: left_cards_all,
            right_cards: right_cards_all,
            game_type,
            start_player,
            trick_cards: 0,
            trick_suit: 0,
            threshold_upper: 1,
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
            declarer_cards: 0u32,
            left_cards: 0u32,
            right_cards: 0u32,
            game_type: Game::Farbe,
            start_player: Player::Declarer,
            trick_cards: 0,
            trick_suit: 0,
            threshold_upper: 0
        }
    }

}
