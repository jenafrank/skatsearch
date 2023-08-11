use std::fmt;
use crate::types::state::State;

// Implementing PartialEq manually
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.played_cards        ==  other.played_cards
            && self.trick_cards         ==  other.trick_cards
            && self.trick_suit          ==  other.trick_suit
            && self.augen_declarer      ==  other.augen_declarer
            && self.player              ==  other.player
            && self.declarer_cards      ==  other.declarer_cards
            && self.left_cards          ==  other.left_cards
            && self.right_cards         ==  other.right_cards
            && self.player_cards        ==  other.player_cards
            && self.trick_cards_count   ==  other.trick_cards_count
            && self.augen_future        ==  other.augen_future
            && self.augen_team          ==  other.augen_team
    }
}

// Implementing Eq manually
impl Eq for State {}

// Implementing Debug manually
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("played cards", &self.played_cards)
            .field("trick cards", &self.trick_cards)
            .finish()
    }
}
