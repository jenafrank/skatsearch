mod search;
mod constructors;
mod methods;
mod traits;

use super::game::Game;
use super::player::Player;
use super::state::State;
use crate::types::tt_table::TtTable;

pub struct Problem {

    // Primary values
    pub declarer_cards_all: u32,
    pub left_cards_all: u32,
    pub right_cards_all: u32,
    pub game_type: Game,
    pub start_player: Player,
    pub points_to_win: u8,
    
    // Primary values for intertrick problems
    pub trick_cards: u32,
    pub trick_suit: u32,    
 
    // Derived values
    pub augen_total: u8,
    pub nr_of_cards: u8,

    // Transposition table and counters for statistics
    pub transposition_table: TtTable
}

impl Problem {

    pub fn new_state(&self, alpha: u8, beta: u8) -> State {

        let player_cards = match self.start_player {
            Player::Declarer => self.declarer_cards_all,
            Player::Left => self.left_cards_all,
            Player::Right => self.right_cards_all,
        };

        let trick_cards_count = self.trick_cards.count_ones() as u8;

        State {
            played_cards: self.trick_cards,
            player: self.start_player,
            trick_cards: self.trick_cards,
            trick_suit: self.trick_suit,
            augen_declarer: 0, // by definition
            augen_team: 0,
            augen_future: self.augen_total,
            declarer_cards: self.declarer_cards_all,
            left_cards: self.left_cards_all,
            right_cards: self.right_cards_all,
            player_cards: player_cards,
            trick_cards_count,
            alpha: alpha,
            beta: beta,
            mapped_hash: 0,
            is_root_state: true,            
        }.add_hash()
    }
}