use crate::types::connections::Connections;
use crate::consts::bitboard::CONNECTION_BREAKER;
use crate::traits::Bitboard;

///
/// Connection format:
///
/// row 0 : nr of connections found, single move pattern, single move pattern
/// row 1 : connection pattern, max value card in connection, min value card in connection,
/// ...
///
/// max 5 connections => +1 for row 0 and +1 for terminating row (all zero as closing signature) = 7
///
pub fn get_connections(moves: u32, all_not_played_cards: u32, connection_list: &[(u32, u8)]) -> Connections {
    let mut nr_connections = 0usize;
    let mut single_moves = 0u32;
    let mut connections: Connections = [(0, 0, 0); 7];

    let mut conn_length = 0u8;
    let mut conn_all = 0u32;
    let mut conn_high = (0u32, 0u8);
    let mut conn_low = (0u32, 0u8);

    for &(mov, aug) in connection_list {
        if mov == CONNECTION_BREAKER {
            (single_moves, conn_length) = invalidate_connection(single_moves, conn_length, conn_all);
        }

        // Only regard unplayed cards
        if all_not_played_cards.__contain(mov) {

            // Card on hand under consideration
            if moves.__contain(mov) {
                conn_length += 1;

                // Init vals
                if conn_length == 1 {
                    conn_high = (mov, aug);
                    conn_low = (mov, aug);
                    conn_all = mov;
                }

                // Add to connection pattern
                conn_all |= mov;

                // Update high card
                if aug > conn_high.1
                {
                    conn_high = (mov, aug);
                }

                // Update low card
                if aug <= conn_low.1 {
                    conn_low = (mov, aug);
                }

                // Register as valid connection (increase number)
                if conn_length == 2 {
                    nr_connections += 1;
                }

                // Update in list
                if conn_length >= 2 {
                    connections[nr_connections] = (conn_all, conn_high.0, conn_low.0);
                }
            } else {
                (single_moves, conn_length) = invalidate_connection(single_moves, conn_length, conn_all);
            }
        }
    }

    connections[0] = (nr_connections as u32, single_moves, single_moves);

    connections
}

#[inline(always)]
fn invalidate_connection(single_moves: u32, conn_length: u8, conn_all: u32)
                         -> (u32, u8)
{
    // Save to single move, if connection pattern breaks after first entry.
    let new_single_moves = if conn_length == 1 {
        single_moves | conn_all
    } else {
        single_moves
    };

    // Invalidate connection
    (new_single_moves, 0)
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::consts::bitboard::{FARB_CONN, GRAND_CONN};
    use crate::traits::BitConverter;

    /// Check hand with one single connection
    #[test]
    fn easy() {
        let x = get_connections("CA CT CK".__bit(),
                                "CA CT CK CQ C9 C8 C7 SA ST".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 1);
        assert_eq!(x[0].1, 0);
        assert_eq!(x[0].2, 0);

        assert_eq!(x[1].0, "CA CT CK".__bit());
        assert_eq!(x[1].1, "CA".__bit());
        assert_eq!(x[1].2, "CK".__bit());

        assert_eq!(x[2].0, 0);
        assert_eq!(x[2].1, 0);
        assert_eq!(x[2].2, 0);
    }

    /// Check if ace is high card even if a jack would make the trick.
    #[test]
    fn advanced() {
        let x = get_connections("DJ CA CT".__bit(),
                                "DJ CA CT CQ C9 C8 C7 SA ST".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 1);
        assert_eq!(x[0].1, 0);
        assert_eq!(x[0].2, 0);

        assert_eq!(x[1].0, "DJ CA CT".__bit());
        assert_eq!(x[1].1, "CA".__bit());
        assert_eq!(x[1].2, "DJ".__bit());

        assert_eq!(x[2].0, 0);
        assert_eq!(x[2].1, 0);
        assert_eq!(x[2].2, 0);
    }

    /// Check if connection allows 'holes' if those holes _have been_ already
    /// played, i.e. they are not part of the cards that _will be_ played.
    #[test]
    fn advanced_2() {
        let x = get_connections("DJ CA CK".__bit(),
                                "DJ CA CK CQ C9 C8 C7 SA ST".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 1);
        assert_eq!(x[0].1, 0);
        assert_eq!(x[0].2, 0);

        assert_eq!(x[1].0, "DJ CA CK".__bit());
        assert_eq!(x[1].1, "CA".__bit());
        assert_eq!(x[1].2, "DJ".__bit());

        assert_eq!(x[2].0, 0);
        assert_eq!(x[2].1, 0);
        assert_eq!(x[2].2, 0);
    }

    /// Destroy the connection by adding the single Club Ten to the
    /// available cards (at hand of another player).
    #[test]
    fn advanced_3() {
        let x = get_connections("DJ CA CK".__bit(),
                                "\
                    DJ CA CK | \
                    CT C9 C8 | \
                    C7 SA ST".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 1);
        assert_eq!(x[0].1, "CK".__bit());
        assert_eq!(x[0].2, x[0].1);

        assert_eq!(x[1].0, "DJ CA".__bit());
        assert_eq!(x[1].1, "CA".__bit());
        assert_eq!(x[1].2, "DJ".__bit());

        assert_eq!(x[2].0, 0);
        assert_eq!(x[2].1, 0);
        assert_eq!(x[2].2, 0);
    }

    /// Add a little more cards and identify two connections.
    #[test]
    fn advanced_4() {
        let x = get_connections("CK CQ C7 SA ST".__bit(),
                                "\
                    CK CQ C7 SA ST | \
                    H9 H8 H7 DA DT | \
                    S9 S8 S7 HA HT".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 2);
        assert_eq!(x[0].1, 0);
        assert_eq!(x[0].2, x[0].1);

        assert_eq!(x[1].0, "CK CQ C7".__bit());
        assert_eq!(x[1].1, "CK".__bit());
        assert_eq!(x[1].2, "C7".__bit());

        assert_eq!(x[2].0, "SA ST".__bit());
        assert_eq!(x[2].1, "SA".__bit());
        assert_eq!(x[2].2, "ST".__bit());

        assert_eq!(x[3].0, 0);
        assert_eq!(x[3].1, 0);
        assert_eq!(x[3].2, 0);
    }

    /// Check if rank order is replicated even though the cards
    /// C9 C8 C7 are completely equivalent here. Rank order
    /// is that C9 is above C8 or C7, and C7 is the lowest card from those three.
    /// All cards have value of zero here and can not make any difference in the
    /// outcome of the game here if played out. Game(C9) = Game(C8) = Game(C7).
    /// Reason for this is to just replicate common sense, anticipating debugging
    /// or calculation of game courses. That will be easier if you get not
    /// distracted by violating common sense even though it does not affect the
    /// game outcome.
    #[test]
    fn advanced_5() {
        let x = get_connections("C9 C8 C7 SA ST".__bit(),
                                "\
                    C9 C8 C7 SA ST | \
                    H9 H8 H7 DA DT | \
                    S9 S8 S7 HA HT".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 2);
        assert_eq!(x[0].1, 0);
        assert_eq!(x[0].2, x[0].1);

        assert_eq!(x[1].0, "C9 C8 C7".__bit());
        assert_eq!(x[1].1, "C9".__bit());
        assert_eq!(x[1].2, "C7".__bit());

        assert_eq!(x[2].0, "SA ST".__bit());
        assert_eq!(x[2].1, "SA".__bit());
        assert_eq!(x[2].2, "ST".__bit());

        assert_eq!(x[3].0, 0);
        assert_eq!(x[3].1, 0);
        assert_eq!(x[3].2, 0);
    }

    /// Destroy previous connection again and check result.
    #[test]
    fn advanced_6() {
        let x = get_connections("CK CQ C7 SA ST".__bit(),
                                "\
                    CK CQ C7 SA ST | \
                    C9 H8 H7 DA DT | \
                    S9 S8 S7 HA HT".__bit(),
                                &FARB_CONN);

        assert_eq!(x[0].0, 2);
        assert_eq!(x[0].1, "C7".__bit());
        assert_eq!(x[0].2, x[0].1);

        assert_eq!(x[1].0, "CK CQ".__bit());
        assert_eq!(x[1].1, "CK".__bit());
        assert_eq!(x[1].2, "CQ".__bit());

        assert_eq!(x[2].0, "SA ST".__bit());
        assert_eq!(x[2].1, "SA".__bit());
        assert_eq!(x[2].2, "ST".__bit());

        assert_eq!(x[3].0, 0);
        assert_eq!(x[3].1, 0);
        assert_eq!(x[3].2, 0);
    }

    #[test]
    fn grand_1() {
        let x = get_connections("SJ DJ CT CQ D7".__bit(),
                                "\
                    SJ DJ CT CQ D7| \
                    HA HT H9 H8 D8| \
                    SA ST S9 S8 D9".__bit(),
                                &GRAND_CONN);

        assert_eq!(x[0].0, 2);
        assert_eq!(x[0].1, "D7".__bit());
        assert_eq!(x[0].2, x[0].1);

        assert_eq!(x[1].0, "SJ DJ".__bit());
        assert_eq!(x[1].1, "SJ".__bit());
        assert_eq!(x[1].2, "DJ".__bit());

        assert_eq!(x[2].0, "CT CQ".__bit());
        assert_eq!(x[2].1, "CT".__bit());
        assert_eq!(x[2].2, "CQ".__bit());

        assert_eq!(x[3].0, 0);
        assert_eq!(x[3].1, 0);
        assert_eq!(x[3].2, 0);
    }

}