use crate::consts::bitboard::{JACKOFCLUBS, RANGE};
use crate::consts::general::{AUGENLIST, REVERSED_AUGENLIST};
use crate::traits::Augen;
use crate::traits::Bitboard;

impl Augen for u32 {

    fn __get_value(&self) -> u8 {
        let mut augen = 0u8;
        let mut current_card = JACKOFCLUBS;

        for &i in &RANGE {
            if self.__contain(current_card) {
                augen += AUGENLIST[i];
            }
            current_card >>= 1;
        }

        augen
    }

    fn __get_value_of_three_cards(&self) -> u8 {
        let mut x: u32 = *self;

        let i1 = x.trailing_zeros() as usize;
        x &= x - 1;
        let i2 = x.trailing_zeros() as usize;
        x &= x - 1;
        let i3 = x.trailing_zeros() as usize;

        REVERSED_AUGENLIST[i1] + REVERSED_AUGENLIST[i2] + REVERSED_AUGENLIST[i3]
    }

    fn __get_number_of_bits(&self) -> u8 {
        self.count_ones() as u8
    }

    fn __get_from_one_card(&self) -> u8 {
        let i1 = self.trailing_zeros() as usize;
        REVERSED_AUGENLIST[i1]
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::BitConverter;
    use super::*;

    #[test]
    fn test_get() {
        assert_eq!(
            0b1000_0000000_0000000_0000000_0000000u32.__get_value(),
            2
        );
        assert_eq!(
            0b1000_1000000_0000000_0000000_0000000u32.__get_value(),
            13
        );
        assert_eq!(
            0b0000_0000000_0000000_0000000_0001111u32.__get_value(),
            3
        );

        assert_eq!("[CA SA HA DA]".__bit().__get_value(), 44);
    }

    #[test]
    fn test_get_from_one_card() {
        assert_eq!("CJ".__bit().__get_from_one_card(), 2);
    }

    #[test]
    fn test_get_from_three_cards() {
        assert_eq!(
            "[CA SA HA]".__bit().__get_value_of_three_cards(),
            33
        );
        assert_eq!(
            "[CA SA HA D7]".__bit().__get_value_of_three_cards(),
            22
        );

        // if more than three cards exist, the remaining will be skipped
        // regarding standard order, here: CA
        assert_eq!(
            "[CA D7 HK SQ]".__bit().__get_value_of_three_cards(),
            7
        );
        assert_eq!(
            "[HK DQ CK]".__bit().__get_value_of_three_cards(),
            11
        );

        // multiples of one card are mapped to one single
        assert_eq!(
            "[HK HK CJ SJ]".__bit().__get_value_of_three_cards(),
            8
        );
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 32 but the index is 32")]
    fn test_get_from_three_cards_panic_1() {
        let _x = "[CJ SJ]".__bit().__get_value_of_three_cards();
    }

    #[test]
    #[should_panic]
    fn test_get_from_three_cards_panic_2() {
        let _x = "[CJ]".__bit().__get_value_of_three_cards();
    }
}
