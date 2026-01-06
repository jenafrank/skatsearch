use crate::types::player::Player;
use crate::consts::general::TT_SIZE_U64;
use crate::core_functions::get_hash::get_hash;

pub fn get_mapped_hash(player: Player, left_cards: u32, right_cards: u32, declarer_cards: u32, trick_cards: u32) -> usize {
    let hash = get_hash(player, left_cards, right_cards, declarer_cards, trick_cards);
    let mapped_hash = hash % TT_SIZE_U64;
    mapped_hash as usize
}
