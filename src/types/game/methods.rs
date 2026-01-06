use super::*;
use crate::consts::bitboard::*;

impl Game {
    pub fn get_trump(&self) -> u32 {
        match self {
            Game::Farbe => TRUMP_FARBE,
            Game::Grand => TRUMP_GRAND,
            Game::Null => TRUMP_NULL
        }
    }

    pub fn get_unequal_sequence(&self) -> &[(u32, u8)] {
        match self {
            Game::Farbe => &FARB_CONN,
            Game::Grand => &GRAND_CONN,
            Game::Null => panic!("Not allowed for Null because Null has only equal values.")
        }
    }

    pub fn get_equal_sequence(&self) -> &[u32] {
        match self {
            Game::Farbe => &FARB_CONN_EQ,
            Game::Grand => &GRAND_CONN_EQ,
            Game::Null => &NULL_CONN_EQ
        }
    }
}
