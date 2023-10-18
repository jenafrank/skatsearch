use super::State;

use crate::types::game::Game;
use crate::types::problem::Problem;

use crate::core_functions::get_connections::get_connections;
use crate::core_functions::get_reduced_equal_core::get_reduced_equal_core;

impl State {

    pub fn get_reduced(&self, problem: &Problem) -> u32 {
        let mut moves = self.get_legal_moves();
        match problem.game_type {
            Game::Farbe | Game::Grand => {
                moves = self.reduce_unequal(moves, problem);
                moves = self.reduce_equal(moves, problem.game_type);
            },
            Game::Null => {
                moves = self.reduce_equal(moves, problem.game_type);
            }
        }
        moves
    }

    fn reduce_unequal(&self, moves: u32, problem: &Problem) -> u32 {
        let mut ret = 0u32;

        let player = self.player;
        let all_moves = self.left_cards | self.right_cards | self.declarer_cards;

        let connections = get_connections(
            moves, all_moves, problem.game_type.get_unequal_sequence());

        let mut i = 1;
        while connections[i].0 > 0 {
            let repr = connections[i].1; // could be any card from connection
            let winner = self.get_forecasted_winner_of_current_trick(repr, problem);

            if let Some(x) = winner {
                if x.is_same_team_as(player) {
                    ret |= connections[i].1;
                } else {
                    ret |= connections[i].2;
                }
            } else {
                ret |= connections[i].0;
            }

            i += 1;
        }

        ret | connections[0].1
    }

    fn reduce_equal(&self, moves: u32, game: Game) -> u32 {
        // todo: add self.trick_cards ?
        let all_moves = self.declarer_cards | self.left_cards | self.right_cards | self.trick_cards;
        let connection_list = game.get_equal_sequence();

        get_reduced_equal_core(moves, all_moves, connection_list)
    }
}