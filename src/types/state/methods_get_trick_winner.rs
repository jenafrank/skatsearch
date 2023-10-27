use super::*;

use crate::types::player::Player;
use crate::types::problem::Problem;

impl State {
    pub fn get_trick_winner(
        &self, 
        trick_cards: u32, 
        trick_suit: u32, 
        problem: &Problem) -> Player 
        {
            crate::core_functions::get_trick_winner::get_trick_winner(
                trick_cards,
                trick_suit,
                problem.game_type(),
                problem.declarer_cards(),
                problem.left_cards(),
                problem.right_cards(),
            )
    }
}