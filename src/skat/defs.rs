//! # Core Definitions
//!
//! Contains basic types (`Game`, `Player`) and bitboard constants used throughout the engine.
pub use crate::consts::bitboard::{
    ACES, ALLCARDS, CARDS, CLUBS, CONNECTION_BREAKER, DIAMONDS, EIGHTOFCLUBS, EIGHTOFDIAMONDS,
    EIGHTOFHEARTS, EIGHTOFSPADES, EIGHTS, FARB_CONN, FARB_CONN_EQ, GRAND_CONN, GRAND_CONN_EQ,
    HEARTS, JACKOFCLUBS, JACKOFDIAMONDS, JACKOFHEARTS, JACKOFSPADES, JACKS, KINGOFCLUBS,
    KINGOFDIAMONDS, KINGOFHEARTS, KINGOFSPADES, KINGS, NINEOFCLUBS, NINEOFDIAMONDS, NINEOFHEARTS,
    NINEOFSPADES, NINES, NULL_CLUBS, NULL_CONN_EQ, NULL_DIAMONDS, NULL_HEARTS, NULL_SPADES,
    QUEENOFCLUBS, QUEENOFDIAMONDS, QUEENOFHEARTS, QUEENOFSPADES, QUEENS, RANGE, RANGE_INV,
    SEVENOFCLUBS, SEVENOFDIAMONDS, SEVENOFHEARTS, SEVENOFSPADES, SEVENS, SPADES, TENOFCLUBS,
    TENOFDIAMONDS, TENOFHEARTS, TENOFSPADES, TENS, TRUMP_FARBE, TRUMP_GRAND, TRUMP_NULL,
};

// -----------------------------------------------------------------------------
// PLAYER
// -----------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum Player {
    #[default]
    Declarer = 0,
    Left = 1,
    Right = 2,
}

impl Player {
    pub fn inc(&self) -> Player {
        match self {
            Player::Declarer => Player::Left,
            Player::Left => Player::Right,
            Player::Right => Player::Declarer,
        }
    }

    pub fn dec(&self) -> Player {
        self.inc().inc()
    }

    pub fn str(&self) -> &str {
        match self {
            Player::Declarer => "D",
            Player::Left => "L",
            Player::Right => "R",
        }
    }

    pub fn is_team(&self) -> bool {
        !self.is_declarer()
    }

    pub fn is_declarer(&self) -> bool {
        matches!(self, Player::Declarer)
    }

    pub fn is_same_team_as(&self, player: Player) -> bool {
        self.is_team() == player.is_team()
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::Declarer => write!(f, "Declarer"),
            Player::Left => write!(f, "Left"),
            Player::Right => write!(f, "Right"),
        }
    }
}

// -----------------------------------------------------------------------------
// GAME
// -----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Game {
    Farbe,
    Grand,
    Null,
}

impl Game {
    pub fn convert_to_string(&self) -> String {
        match self {
            Game::Farbe => "Farbe".to_string(),
            Game::Grand => "Grand".to_string(),
            Game::Null => "Null".to_string(),
        }
    }

    pub fn get_trump(&self) -> u32 {
        match self {
            Game::Farbe => TRUMP_FARBE,
            Game::Grand => TRUMP_GRAND,
            Game::Null => TRUMP_NULL,
        }
    }

    pub fn get_unequal_sequence(&self) -> &[(u32, u8)] {
        match self {
            Game::Farbe => &FARB_CONN,
            Game::Grand => &GRAND_CONN,
            Game::Null => panic!("Not allowed for Null because Null has only equal values."),
        }
    }

    pub fn get_equal_sequence(&self) -> &[u32] {
        match self {
            Game::Farbe => &FARB_CONN_EQ,
            Game::Grand => &GRAND_CONN_EQ,
            Game::Null => &NULL_CONN_EQ,
        }
    }
}

// Keep the first definition and impls.
// Removing duplicate definition lines 75-80.
// Keeping impl Player lines 82-114.

// Constants are imported from crate::consts::bitboard
