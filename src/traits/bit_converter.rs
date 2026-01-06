use crate::consts::bitboard::{CARDS, RANGE};
use crate::traits::BitConverter;

impl BitConverter for str {

    fn __bit(&self) -> u32 {
        let cards_string = self.to_string();
        let mut ret = 0u32;

        for &i in &RANGE {
            ret <<= 1;
            if cards_string.contains(CARDS[i]) {
                ret += 1;
            }
        }

        ret
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bit() {
        assert_eq!("[]".__bit(), 0);
        assert_eq!("[D7]".__bit(), 1u32);
        assert_eq!("[CJ]".__bit(), 0b_1000_0000_0000_0000_0000_0000_0000_0000u32);
        assert_eq!("[CJ HJ CA CK D7]".__bit(), 0b_1010_1010_0000_0000_0000_0000_0000_0001u32);
    }
}
