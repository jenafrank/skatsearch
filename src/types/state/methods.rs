use super::*;

impl State {
    pub fn calc_hash(&self) -> usize {
        get_mapped_hash(self.player, self.get_all_unplayed_cards(), self.trick_cards)
    }
    pub fn is_new_trick(&self) -> bool {
        self.trick_cards_count == 0
    }
}

