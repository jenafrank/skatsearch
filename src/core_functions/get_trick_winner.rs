use crate::types::game::Game;
use crate::types::player::Player;
use crate::consts::bitboard::{ACES, EIGHTS, JACKS, KINGS, NINES, QUEENS, SEVENS, TENS};
use crate::traits::Bitboard;

pub fn get_trick_winner(
    trick_cards: u32,
    trick_suit: u32,
    game_type: Game,
    declarer_cards_all: u32,
    left_cards_all: u32,
    right_cards_all: u32,
) -> Player {
    let trump = game_type.get_trump();

    let is_trump = (trick_cards & trump) > 0;
    let lead_suit = if is_trump { trump } else { trick_suit };
    let lead_trick = lead_suit & trick_cards;

    let mut lead_declarer = lead_trick & declarer_cards_all;
    let mut lead_left = lead_trick & left_cards_all;
    let mut lead_right = lead_trick & right_cards_all;

    if let Game::Null = game_type {
        lead_declarer = nullmap(lead_declarer);
        lead_left = nullmap(lead_left);
        lead_right = nullmap(lead_right);
    }

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

fn nullmap(card: u32) -> u32 {
    match card {
        x if SEVENS.__contain(x) => 1,
        x if EIGHTS.__contain(x) => 2,
        x if NINES.__contain(x) => 3,
        x if TENS.__contain(x) => 4,
        x if JACKS.__contain(x) => 5,
        x if QUEENS.__contain(x) => 6,
        x if KINGS.__contain(x) => 7,
        x if ACES.__contain(x) => 8,
        x if x == 0 => 0,
        _ => panic!("Impossible single card '{}'", card)
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