use crate::types::state::State;
use crate::types::tt_entry::TtEntry;

impl TtEntry {
    pub fn matches(&self, state: &State) -> bool {
            self.player == state.player &&
            self.cards == state.get_all_unplayed_cards()
    }
}
