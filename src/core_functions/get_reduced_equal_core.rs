use crate::consts::bitboard::CONNECTION_BREAKER;
use crate::traits::Bitboard;

pub fn get_reduced_equal_core(moves: u32, all_moves: u32, connection_list: &[u32]) -> u32 {
    let mut ret = moves;
    let mut precseen = false;

    for &mov in connection_list {
        if mov == CONNECTION_BREAKER {
            precseen = false;
            continue;
        }

        // Only regard unplayed cards
        if all_moves.__contain(mov) {

            // Card on hand under consideration
            if moves.__contain(mov) {
                if precseen {
                    ret &= !mov;
                }
                precseen = true;
            } else {
                precseen = false;
            }
        }
    }

    ret
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::consts::bitboard::{ALLCARDS, FARB_CONN_EQ, NULL_CONN_EQ};
    use crate::traits::BitConverter;

    /// Tests reducing of four to one jack
    #[test]
    fn test_easiest() {
        let res = get_reduced_equal_core(
            "CJ SJ HJ DJ CA CT CK CQ".__bit(),
            ALLCARDS,
            &FARB_CONN_EQ);
        assert_eq!(res, "CJ CA CT CK CQ".__bit());
    }

    /// Moves one check from player card to already played cards
    /// Result must be the same.
    #[test]
    fn test_movecard_to_hole() {
        let res = get_reduced_equal_core(
            "CJ HJ DJ CA CT CK CQ".__bit(),
            ALLCARDS & !"SJ".__bit(),
            &FARB_CONN_EQ);
        assert_eq!(res, "CJ CA CT CK CQ".__bit());
    }

    /// Test null reducing. Reducing of low cards in connection.
    #[test]
    fn test_null_easy() {
        let res = get_reduced_equal_core(
            "CA CK C9 C8 C7".__bit(),
            ALLCARDS,
            &NULL_CONN_EQ);
        assert_eq!(res, "CA C9".__bit());
    }

    /// Test null reducing. Add club ten.
    #[test]
    fn test_null_add_ten() {
        let res = get_reduced_equal_core(
            "CA CT CK C9 C8 C7".__bit(),
            ALLCARDS,
            &NULL_CONN_EQ);
        assert_eq!(res, "CA CT".__bit());
    }

}
