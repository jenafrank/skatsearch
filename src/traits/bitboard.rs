use crate::consts::bitboard::JACKOFCLUBS;
use crate::traits::Bitboard;

impl Bitboard for u32 {

    fn __contain(&self, card: u32) -> bool {
        (self & card) > 0
    }

    fn __is_odd(&self) -> bool { self % 2 == 1 }

    fn __decompose(&self) -> ([u32; 32], usize) {
        let mut ret = [0; 32];
        let mut i = 0;
        let mut x = JACKOFCLUBS;

        while x > 0 {
            if self & x > 0 {
                ret[i] = x;
                i += 1;
            }
            x >>= 1;
        }

        (ret,i)
    }

    fn __decompose_twelve(&self) -> [u32; 12] {
        let mut ret = [0; 12];
        let mut i = 0;
        let mut x = JACKOFCLUBS;

        while x > 0 {
            if self & x > 0 {
                ret[i] = x;
                i += 1;
            }
            x >>= 1;
        }

        ret
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn is_odd() {
        assert!(3u32.__is_odd());
        assert!(!0b_1010_1010_0000_0000_0000_0000_0000_0000u32.__is_odd());
        assert!(0b_1010_1010_0000_0000_0000_0000_0000_0001u32.__is_odd());
    }

}