use crate::consts::bitboard::{CARDS, RANGE_INV};
use crate::traits::{Bitboard, StringConverter};

impl StringConverter for u32 {

    fn __str(&self) -> String {
        let mut x = *self;
        let mut ret = String::new();

        for &i in &RANGE_INV {
            if x.__is_odd() {
                ret.insert(0, ' ');
                ret.insert_str(0, CARDS[i]);
            }
            x >>= 1;
        }

        ret = ret.trim().to_string();

        if ret.len() == 2 {
            return ret;
        }

        ret.push(']');
        ret.insert(0, '[');

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        assert_eq!(0u32.__str(),"[]");
        assert_eq!(0b1u32.__str(),"D7");
        assert_eq!(0b11u32.__str(),"[D8 D7]");
        assert_eq!(0b_1111u32.__str(),"[DQ D9 D8 D7]");
        assert_eq!(0b_1111u32.__str(),"[DQ D9 D8 D7]");
        assert_eq!(0b_1000_0000_0000_0000_0000_0000_0000_0000u32.__str(),"CJ");
        assert_eq!(0b_1010_1010_0000_0000_0000_0000_0000_0000u32.__str(),"[CJ HJ CA CK]");
    }
}
