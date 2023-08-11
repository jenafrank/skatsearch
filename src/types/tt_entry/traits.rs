use crate::types::tt_entry::TtEntry;

impl PartialEq for TtEntry {
    fn eq(&self, other: &Self) -> bool {
        self.player == other.player &&
            self.cards == other.cards
    }
}
