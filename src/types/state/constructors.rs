use super::*;

use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::traits::Augen;

impl State {

    pub fn create(
        played_cards: u32,
        trick_cards: u32,
        trick_suit: u32,
        augen_declarer: u8,
        player: Player,
        problem: &Problem,
    ) -> State {

        let declarer_cards: u32 = problem.declarer_cards_all & !played_cards;
        let left_cards: u32 = problem.left_cards_all & !played_cards;
        let right_cards: u32 = problem.right_cards_all & !played_cards;
        let augen_future = problem.augen_total - (played_cards & !trick_cards).__get_value();
        let player_cards = match player {
            Player::Declarer => declarer_cards,
            Player::Left => left_cards,
            Player::Right => right_cards,
        };
        let trick_cards_count = trick_cards.count_ones() as u8;
        let augen_team = problem.augen_total - augen_future - augen_declarer;

        State {
            // Primary values
            player,
            played_cards,
            trick_cards,
            trick_suit,
            augen_declarer,

            // Derived values (~ cached values)
            declarer_cards,
            left_cards,
            right_cards,
            player_cards,
            trick_cards_count,
            augen_future,
            augen_team,
        }
    }

    pub fn create_initial_state_from_problem(problem: &Problem) -> State {
        State::create(0u32, 0u32, 0u32, 0u8, problem.start_player, problem)
    }
}
