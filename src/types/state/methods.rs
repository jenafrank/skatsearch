use crate::core_functions::get_mapped_hash::get_mapped_hash;
use crate::types::state::State;

impl State {

    pub fn is_not_root_state(&self) -> bool {
        self.is_root_state == false
    }

    pub fn get_mapped_hash(&self) -> usize {
        get_mapped_hash(self.player, self.left_cards, self.right_cards, self.declarer_cards)
    }

    pub fn is_new_trick(&self) -> bool {
        self.trick_cards_count == 0
    }
    
}