/// Calculates the total set of not played cards, combining all hands
/// (declarer, left player and right player)
///
/// * __In__: Bitboard-encoded set of cards of each player (declarer, left player and right player)
/// * __Out__: Combined set of all cards, bitboard-encoded.

pub fn get_all_unplayed_cards(declarer_cards: u32, left_cards: u32, right_cards: u32) -> u32 {
    declarer_cards | left_cards | right_cards
}