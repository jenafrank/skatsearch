mod search;
pub mod counters;
mod constructors;
mod methods;
mod traits;

use super::game::Game;
use super::player::Player;
use crate::types::problem::counters::Counters;
use crate::types::tt_table::TtTable;

pub struct Problem {

    // Primary values
    pub declarer_cards_all: u32,
    pub left_cards_all: u32,
    pub right_cards_all: u32,
    pub game_type: Game,
    pub start_player: Player,
 
    // Derived values
    pub augen_total: u8,
    pub nr_of_cards: u8,

    // Transposition table and counters for statistics
    pub transposition_table: TtTable,
    pub counters: Counters

}

