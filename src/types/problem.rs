use crate::types::game::Game;
use crate::types::player::Player;

pub mod constructors;
mod methods;

#[derive(Default, Copy, Debug, Clone)]
pub struct Problem {
    // Primary values
    pub declarer_cards_all: u32,
    pub left_cards_all: u32,
    pub right_cards_all: u32,
    pub game_type: Game,
    pub start_player: Player,

    // Derived values
    pub augen_total: u8,
    pub nr_of_cards: u8
}