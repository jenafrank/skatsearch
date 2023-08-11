mod constructors;
mod methods;
mod traits;
mod methods_create_child_state;
mod methods_get_all_unplayed_cards;
mod methods_get_cards_for_player;
mod methods_get_forecasted_winner_of_current_trick;
mod methods_get_legal_moves;
mod methods_get_trick_winner;
mod methods_reducer;

use crate::types::player::Player;
use crate::core_functions::get_mapped_hash::get_mapped_hash;

#[derive(Copy, Clone)]
pub struct State {

    // Primary values

    pub player: Player,
    pub played_cards: u32,
    pub trick_cards: u32,
    pub trick_suit: u32,
    pub augen_declarer: u8,


    // Derived (cached) values

    pub declarer_cards: u32,
    pub left_cards: u32,
    pub right_cards: u32,
    pub player_cards: u32,
    pub trick_cards_count: u8,
    pub augen_future: u8,
    pub augen_team: u8,
}



#[cfg(test)]
mod tests_derived_state {
    use crate::types::game::Game;
    use crate::types::state::State;
    use crate::types::problem::Problem;
    use crate::types::player::Player;
    use crate::consts::bitboard::TRUMP_FARBE;
    use crate::traits::{Augen, BitConverter};

    #[test]
    fn test_initial_state_from_problem() {
        let p = Problem::create(
            "CJ SJ".__bit(),
            "SA ST".__bit(),
            "CA CT".__bit(),
            Game::Farbe,
            Player::Declarer,
        );

        let x = State::create_initial_state_from_problem(&p);

        let y = State {
            player: Player::Declarer,
            played_cards: 0,
            declarer_cards: "CJ SJ".__bit(),
            left_cards: "SA ST".__bit(),
            right_cards: "CA CT".__bit(),
            player_cards: "CJ SJ".__bit(),
            trick_cards: 0,
            trick_suit: 0,
            trick_cards_count: 0,
            augen_future: "CJ SJ SA ST CA CT".__bit().__get_value(),
            augen_declarer: 0,
            augen_team: 0,
        };

        assert_eq!(x, y);
    }

    #[test]
    fn test_from() {
        let p = Problem::create(
            "CJ SJ".__bit(),
            "SA ST".__bit(),
            "CA CT".__bit(),
            Game::Farbe,
            Player::Declarer,
        );

        let x = State::create(
            "CJ SA".__bit(),
            "CJ SA".__bit(),
            TRUMP_FARBE,
            0,
            Player::Right,
            &p,
        );

        let y = State {
            player: Player::Right,
            played_cards: "CJ SA".__bit(),
            declarer_cards: "SJ".__bit(),
            left_cards: "ST".__bit(),
            right_cards: "CA CT".__bit(),
            player_cards: "CA CT".__bit(),
            trick_cards: "CJ SA".__bit(),
            trick_suit: TRUMP_FARBE,
            trick_cards_count: 2,
            augen_future: "CJ SJ SA ST CA CT".__bit().__get_value(),
            augen_declarer: 0,
            augen_team: 0,
        };

        assert_eq!(x, y);
    }

}

#[cfg(test)]
mod tests_evaluation {
    use crate::types::game::*;
    use crate::types::problem::Problem;
    use crate::types::player::Player;
    use crate::types::state::State;
    use crate::traits::BitConverter;

    #[test]
    fn test_trick_evaluation_1() {
        let p = Problem::create(
            "CJ SJ".__bit(),
            "SA ST".__bit(),
            "CA CT".__bit(),
            Game::Farbe,
            Player::Declarer,
        );

        let s = State::create_initial_state_from_problem(&p);

        let s1 = s.create_child_state("CJ".__bit(), &p);
        let s2 = s1.create_child_state("SA".__bit(), &p);
        let sa = s2.create_child_state("CA".__bit(), &p);

        let es = State::create("CJ SA CA".__bit(), 0, 0, 24, Player::Declarer, &p);

        assert_eq!(sa, es);
    }

}

#[cfg(test)]
mod tests_no_evaluation {
    use crate::types::game::*;
    use crate::types::state::State;
    use crate::types::problem::Problem;
    use crate::types::player::Player;
    use crate::consts::bitboard::TRUMP_FARBE;
    use crate::traits::{Augen, BitConverter};

    #[test]
    fn test_advance_from_trick_beginning() {
        let problem = Problem {
            declarer_cards_all: "[CJ ST]".__bit(),
            left_cards_all: "[SJ SA]".__bit(),
            right_cards_all: "[CA S7]".__bit(),
            game_type: Game::Farbe,
            augen_total: "CJ ST SJ SA CA S7".__bit().__get_value(),
            start_player: Player::Declarer,
            nr_of_cards: 6,
        };

        let state = State {
            played_cards: 0,
            declarer_cards: problem.declarer_cards_all,
            left_cards: problem.left_cards_all,
            right_cards: problem.right_cards_all,
            trick_cards: 0,
            trick_suit: 0,
            trick_cards_count: 0,
            augen_future: "CJ ST SJ SA CA S7".__bit().__get_value(),
            augen_declarer: 0,
            augen_team: 0,
            player: Player::Declarer,
            player_cards: "[CJ ST]".__bit(),
        };


        let next_state = state.create_child_state("CJ".__bit(),&problem);

        let expected_next_state = State {
            played_cards: "CJ".__bit(),
            declarer_cards: "[ST]".__bit(),
            left_cards: "[SJ SA]".__bit(),
            right_cards: "[CA S7]".__bit(),
            trick_cards: "CJ".__bit(),
            trick_suit: TRUMP_FARBE,
            trick_cards_count: 1,
            augen_declarer: 0,
            augen_team: 0,
            augen_future: "CJ ST SJ SA CA S7".__bit().__get_value(),
            player: Player::Left,
            player_cards: "[SJ SA]".__bit(),
        };

        assert_eq!(next_state, expected_next_state);
    }

    #[test]
    fn test_advance_from_within_trick() {
        let problem = Problem {
            declarer_cards_all: "[ST]".__bit(),
            left_cards_all: "[SJ SA]".__bit(),
            right_cards_all: "[CA S7]".__bit(),
            game_type: Game::Farbe,
            augen_total: "CJ ST SJ SA CA S7".__bit().__get_value(),
            start_player: Player::Declarer,
            nr_of_cards: 6,
        };

        let state = State {
            played_cards: "CJ".__bit(),
            declarer_cards: problem.declarer_cards_all, // cache var 1, ToDo: Check if speed gain when del cached vars
            left_cards: problem.left_cards_all,  // cache var 2
            right_cards: problem.right_cards_all, // cache var 3
            trick_cards: "CJ".__bit(),
            trick_cards_count: 1, // cache var 4
            trick_suit: TRUMP_FARBE,
            augen_future: "CJ ST SJ SA CA S7".__bit().__get_value(),
            augen_declarer: 0,
            augen_team: 0, // cache var 5: augen_team = 120 - augen_declarer - augen_future
            player: Player::Left,
            player_cards: "[SJ SA]".__bit(), // cache var 6
            // cache var 7: augen_future ??
        };

        let next_state = state.create_child_state("SA".__bit(), &problem);

        let expected_next_state = State {
            played_cards: "[CJ SA]".__bit(),
            declarer_cards: "[ST]".__bit(),
            left_cards: "[SJ]".__bit(),
            right_cards: "[CA S7]".__bit(),
            trick_cards: "[CJ SA]".__bit(),
            trick_suit: TRUMP_FARBE,
            trick_cards_count: 2,
            augen_declarer: 0,
            augen_team: 0,
            augen_future: "CJ ST SJ SA CA S7".__bit().__get_value(),
            player: Player::Right,
            player_cards: "[CA S7]".__bit(),
        };

        assert_eq!(next_state, expected_next_state);
    }

}