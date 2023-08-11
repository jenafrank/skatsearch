use crate::types::player::Player;

pub fn get_cards_for_player(declarer_cards: u32, left_cards: u32, right_cards: u32, player: Player) -> u32 {
    match player {
        Player::Declarer => declarer_cards,
        Player::Left => left_cards,
        Player::Right => right_cards
    }
}