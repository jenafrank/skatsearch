use crate::types::tt_entry::TtEntry;

impl PartialEq for TtEntry {
    fn eq(&self, other: &Self) -> bool {
            self.player == other.player &&
            self.left_cards == other.left_cards &&
            self.right_cards == other.right_cards &&
            self.declarer_cards == other.declarer_cards &&
            self.trick_cards == other.trick_cards
    }
}
