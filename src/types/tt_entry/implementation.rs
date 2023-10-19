use crate::types::state::State;
use crate::types::tt_entry::TtEntry;

impl TtEntry {
    pub fn matches(&self, state: &State) -> bool {
            self.player == state.player &&
            self.left_cards == state.left_cards &&
            self.right_cards == state.right_cards &&
            self.declarer_cards == state.declarer_cards &&
            self.trick_cards == state.trick_cards
    }
}
