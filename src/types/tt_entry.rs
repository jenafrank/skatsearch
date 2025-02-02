use crate::types::player::Player;
use crate::types::tt_flag::TtFlag;

pub mod implementation;
pub mod traits;

#[derive(Copy, Clone, Debug)]
pub struct TtEntry {
    // occupied or empty slot -> for statistics
    pub occupied: bool,
        
    // full state
    pub player: Player,
    pub left_cards: u32,
    pub right_cards: u32,
    pub declarer_cards: u32,
    pub trick_cards: u32,

    // value
    pub value: u8,

    // for alpha-beta functions
    pub flag: TtFlag,
    pub bestcard: u32
}
