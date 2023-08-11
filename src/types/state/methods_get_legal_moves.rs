use super::*;

impl State {
    pub fn get_legal_moves(&self) -> u32 {
        if self.trick_suit == 0 {
            self.player_cards
        } else {
            crate::core_functions::get_legal_moves::get_legal_moves(self.trick_suit, self.player_cards)
        }
    }
}
