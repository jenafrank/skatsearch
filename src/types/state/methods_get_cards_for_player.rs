use super::*;
use crate::types::player::Player;

impl State {
    pub(crate) fn get_cards_for_player(&self, player: Player) -> u32 {
        crate::core_functions::get_cards_for_player::get_cards_for_player(
            self.declarer_cards,
            self.left_cards,
            self.right_cards,
            player
        )
    }
}
