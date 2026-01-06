use crate::traits::Augen;

use super::Problem;

impl Problem {

    pub fn get_skat(&self) -> u32 {
        (!0u32) ^ self.declarer_cards ^ self.left_cards ^ self.right_cards
    }

    pub fn all_cards(&self) -> u32 {
        self.left_cards | self.right_cards | self.declarer_cards
    }

    pub fn augen_total(&self) -> u8 {
        self.all_cards().__get_value()
    }

    pub fn number_of_cards(&self) -> u32 {
        self.all_cards().count_ones()
    }

}
