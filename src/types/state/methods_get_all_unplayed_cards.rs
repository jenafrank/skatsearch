use super::*;

impl State {
    pub fn get_all_unplayed_cards(&self) -> u32 {
        crate::core_functions::get_all_unplayed_cards::get_all_unplayed_cards(
            self.declarer_cards,
            self.left_cards,
            self.right_cards
        )
    }
}
