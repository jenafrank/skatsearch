use crate::types::game::Game;
use crate::consts::bitboard::{CLUBS, DIAMONDS, HEARTS, NULL_CLUBS, NULL_DIAMONDS, NULL_HEARTS, NULL_SPADES, SPADES, TRUMP_FARBE, TRUMP_GRAND};
use crate::traits::Bitboard;

pub fn get_suit_for_card(card: u32, game_type: Game) -> u32 {
    match game_type {

        Game::Farbe => {
            match card {
                x if TRUMP_FARBE.__contain(x) => TRUMP_FARBE,
                x if SPADES.__contain(x) => SPADES,
                x if HEARTS.__contain(x) => HEARTS,
                x if DIAMONDS.__contain(x) => DIAMONDS,
                _ => 0u32
            }
        }

        Game::Grand => {
            match card {
                x if TRUMP_GRAND.__contain(x) => TRUMP_GRAND,
                x if CLUBS.__contain(x) => CLUBS,
                x if SPADES.__contain(x) => SPADES,
                x if HEARTS.__contain(x) => HEARTS,
                x if DIAMONDS.__contain(x) => DIAMONDS,
                _ => 0u32
            }
        }

        Game::Null => {
            match card {
                x if NULL_CLUBS.__contain(x) => NULL_CLUBS,
                x if NULL_SPADES.__contain(x) => NULL_SPADES,
                x if NULL_HEARTS.__contain(x) => NULL_HEARTS,
                x if NULL_DIAMONDS.__contain(x) => NULL_DIAMONDS,
                _ => 0u32
            }
        }
    }
}
