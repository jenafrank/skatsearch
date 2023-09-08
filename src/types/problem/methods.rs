use super::Problem;

impl Problem {

    pub fn get_skat(&self) -> u32 {
        (!0u32) ^ self.declarer_cards_all ^ self.left_cards_all ^ self.right_cards_all

    }

}