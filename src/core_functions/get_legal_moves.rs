pub fn get_legal_moves(trick_suit: u32, player_cards: u32) -> u32 {
    let mapped_player_cards = trick_suit & player_cards;
    if mapped_player_cards > 0 {
        mapped_player_cards
    } else {
        player_cards
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::types::player::Player;
    use crate::types::state::State;
    use crate::consts::bitboard::{CLUBS, DIAMONDS, HEARTS, NULL_CLUBS, NULL_DIAMONDS, NULL_HEARTS, NULL_SPADES, SPADES, TRUMP_FARBE, TRUMP_GRAND};
    use crate::traits::BitConverter;

    #[test]
    fn farb() {
        let hand = get_hand();
        let trump = get_legal_moves(TRUMP_FARBE, hand);
        let spades = get_legal_moves(SPADES, hand);
        let hearts = get_legal_moves(HEARTS, hand);
        let diamonds = get_legal_moves(DIAMONDS, hand);

        assert_eq!(trump, "[CJ DJ CA CT CQ]".__bit());
        assert_eq!(spades, "[SA ST]".__bit());
        assert_eq!(hearts, "[HK HQ]".__bit());
        assert_eq!(diamonds, hand);
    }

    #[test]
    fn grand() {
        let hand = get_hand();
        let trumpgrand = get_legal_moves(TRUMP_GRAND, hand);
        let clubs = get_legal_moves(CLUBS, hand);

        assert_eq!(trumpgrand, "[CJ DJ]".__bit());
        assert_eq!(clubs, "[CA CT CQ]".__bit());
    }

    #[test]
    fn null() {
        let hand = get_hand();
        let clubs = get_legal_moves(NULL_CLUBS, hand);
        let spades = get_legal_moves(NULL_SPADES, hand);
        let hearts = get_legal_moves(NULL_HEARTS, hand);
        let diamonds = get_legal_moves(NULL_DIAMONDS, hand);

        assert_eq!(clubs, "[CJ CA CT CQ]".__bit());
        assert_eq!(spades, "[SA ST]".__bit());
        assert_eq!(hearts, "[HK HQ]".__bit());
        assert_eq!(diamonds, "[DJ]".__bit());
    }

    #[test]
    fn from_state() {
        let state_empty = get_state_trick_empty();
        let state_not_empty = get_state_trick_not_empty();

        let trick_empty = state_empty.get_legal_moves();
        let trick_not_empty = state_not_empty.get_legal_moves();

        assert_eq!(trick_empty, get_hand());
        assert_eq!(trick_not_empty, "[CJ DJ]".__bit());
    }

    fn get_hand() -> u32 {
        "[CJ DJ CA CT CQ SA ST HK HQ]".__bit()
    }

    fn get_state_trick_not_empty() -> State {
        State {
            played_cards: 0,
            declarer_cards: 0,
            left_cards: 0,
            right_cards: 0,
            trick_cards: 0,
            trick_suit: TRUMP_GRAND,
            trick_cards_count: 0,
            augen_future: 0,
            augen_declarer: 0,
            augen_team: 0,
            player: Player::Declarer,
            player_cards: get_hand(),
        }
    }

    fn get_state_trick_empty() -> State {
        State {
            played_cards: 0,
            declarer_cards: 0,
            left_cards: 0,
            right_cards: 0,
            trick_cards: 0,
            trick_suit: 0,
            trick_cards_count: 0,
            augen_future: 0,
            augen_declarer: 0,
            augen_team: 0,
            player: Player::Declarer,
            player_cards: get_hand(),
        }
    }

}