use crate::types::game::Game;
use crate::types::player::Player;
use crate::consts::bitboard::{ACES, EIGHTS, JACKS, KINGS, NINES, QUEENS, SEVENS, TENS};
use crate::traits::Bitboard;

/// Determines the winner of a trick based on the played cards, the trump suit, and the game type.
///
/// # Arguments
/// * `trick_cards` - A bitmask representing the cards played in the trick.
/// * `trick_suit` - The suit of the leading card in the trick.
/// * `game_type` - The type of game being played (`Game::Farbe`, `Game::Grand`, or `Game::Null`).
/// * `declarer_cards_all` - A bitmask representing all cards of the declarer.
/// * `left_cards_all` - A bitmask representing all cards of the left player.
/// * `right_cards_all` - A bitmask representing all cards of the right player.
///
/// # Returns
/// The player who won the trick (`Player::Declarer`, `Player::Left`, or `Player::Right`).
pub fn get_trick_winner(
    trick_cards: u32,
    trick_suit: u32,
    game_type: Game,
    declarer_cards_all: u32,
    left_cards_all: u32,
    right_cards_all: u32,
) -> Player {
    let trump = game_type.get_trump();

    let is_trump_played = (trick_cards & trump) > 0;
    let effective_suit = if is_trump_played { trump } else { trick_suit };
    let lead_cards = effective_suit & trick_cards;

    let mut lead_declarer = lead_cards & declarer_cards_all;
    let mut lead_left = lead_cards & left_cards_all;
    let mut lead_right = lead_cards & right_cards_all;

    if let Game::Null = game_type {
        lead_declarer = nullmap(lead_declarer).unwrap_or(0);
        lead_left = nullmap(lead_left).unwrap_or(0);
        lead_right = nullmap(lead_right).unwrap_or(0);
    }

    determine_winner(lead_declarer, lead_left, lead_right)
}

/// Determines the winner based on the leading cards of each player.
fn determine_winner(lead_declarer: u32, lead_left: u32, lead_right: u32) -> Player {
    if lead_left > lead_declarer || lead_right > lead_declarer {
        if lead_left < lead_right {
            Player::Right
        } else {
            Player::Left
        }
    } else {
        Player::Declarer
    }
}

/// Maps a card to its rank in a Null game.
fn nullmap(card: u32) -> Option<u32> {
    match card {
        x if SEVENS.__contain(x) => Some(1),
        x if EIGHTS.__contain(x) => Some(2),
        x if NINES.__contain(x) => Some(3),
        x if TENS.__contain(x) => Some(4),
        x if JACKS.__contain(x) => Some(5),
        x if QUEENS.__contain(x) => Some(6),
        x if KINGS.__contain(x) => Some(7),
        x if ACES.__contain(x) => Some(8),
        x if x == 0 => Some(0),
        _ => None, // Invalid card
    }
}

// Tests -----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests_farbe {

    use super::*;

    use crate::consts::bitboard::{TRUMP_FARBE, SPADES, DIAMONDS, HEARTS};
    use crate::types::game::*;
    use crate::types::player::Player;

    #[test]
    fn standard() {
        let winner = get_trick_winner(
            0b_1110_0000000_0000000_0000000_0000000u32,
            TRUMP_FARBE,
            Game::Farbe,
            0b_1000_0000000_0000000_0000000_0000000u32,
            0b_0100_0000000_0000000_0000000_0000000u32,
            0b_0010_0000000_0000000_0000000_0000000u32,
        );
        assert_eq!(winner, Player::Declarer);
    }

    #[test]
    fn trump() {
        let winner = get_trick_winner(
            0b_0000_0000001_1100000_0000000_0000000u32,
            SPADES,
            Game::Farbe,
            0b_0000_0000000_1000000_0000000_0000000u32,
            0b_0000_0000001_0000000_0000000_0000000u32,
            0b_0000_0000000_0100000_0000000_0000000u32,
        );
        assert_eq!(winner, Player::Left);
    }

    #[test]
    fn rotate() {
        let arr = [
            (SPADES, Player::Declarer),
            (HEARTS, Player::Left),
            (DIAMONDS, Player::Right)];

        for el in &arr {
            let winner = get_trick_winner(
                0b_0000_0000000_1000000_0001000_0000001u32,
                el.0,
                Game::Farbe,
                0b_0000_0000000_1000000_0000000_0000000u32,
                0b_0000_0000000_0000000_0001000_0000000u32,
                0b_0000_0000000_0000000_0000000_0000001u32,
            );
            assert_eq!(winner, el.1);
        }
    }

    #[test]
    fn ueberstich() {
        let winner = get_trick_winner(
            0b_0000_0000011_1000000_0000000_0000000u32,
            SPADES,
            Game::Farbe,
            0b_0000_0000001_0000000_0000000_0000000u32,
            0b_0000_0000010_0000000_0000000_0000000u32,
            0b_0000_0000000_1000000_0000000_0000000u32,
        );
        assert_eq!(winner, Player::Left);
    }
}

#[cfg(test)]
mod tests_grand {

    use super::*;

    use crate::consts::bitboard::{CLUBS, SPADES};
    use crate::types::game::*;
    use crate::types::player::Player;

    #[test]
    fn ueberstich() {
        let winner = get_trick_winner(
            0b_0011_1000000_0000000_0000000_0000000u32,
            CLUBS,
            Game::Grand,
            0b_0001_0000000_0000000_0000000_0000000u32,
            0b_0000_1000000_0000000_0000000_0000000u32,
            0b_0010_0000000_0000000_0000000_0000000u32,
        );
        assert_eq!(winner, Player::Right);
    }

    #[test]
    fn clubs_no_trump() {
        let winner = get_trick_winner(
            0b_0000_1000000_1100000_0000000_0000000u32,
            SPADES,
            Game::Grand,
            0b_0000_1000000_0000000_0000000_0000000u32,
            0b_0000_0000000_1000000_0000000_0000000u32,
            0b_0000_0000000_0100000_0000000_0000000u32,
        );
        assert_eq!(winner, Player::Left);
    }
}

#[cfg(test)]
mod tests_null {

    use super::*;

    use crate::consts::bitboard::NULL_CLUBS;
    use crate::types::game::Game;
    use crate::types::player::Player;

    #[test]
    fn einreihung_1() {
        let winner = get_trick_winner(
            0b_1000_0101000_0000000_0000000_0000000u32,
            NULL_CLUBS,
            Game::Null,
            0b_1000_0000000_0000000_0000000_0000000u32,
            0b_0000_0100000_0000000_0000000_0000000u32,
            0b_0000_0001000_0000000_0000000_0000000u32,
        );
        assert_eq!(winner, Player::Right);
    }
}
