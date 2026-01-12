//! # Transposition Table
//!
//! A hash map to store visited game positions, preventing redundant calculations.
//! Stores exact scores or alpha/beta bounds.

use crate::consts::general::TT_SIZE;
use crate::skat::counters::Counters;
use crate::skat::defs::Player;
use crate::skat::position::Position;

// -----------------------------------------------------------------------------
// TT FLAG
// -----------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TranspositionFlag {
    Exact,
    Upper,
    Lower,
}

// -----------------------------------------------------------------------------
// TT ENTRY
// -----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct TranspositionEntry {
    // occupied or empty slot -> for statistics
    pub occupied: bool,

    // full state key
    pub player: Player,
    pub left_cards: u32,
    pub right_cards: u32,
    pub declarer_cards: u32,
    pub trick_cards: u32, // Used for collision check

    // value
    pub value: i16,

    // for alpha-beta functions
    pub flag: TranspositionFlag,
    pub bestcard: u32,
}

impl TranspositionEntry {
    pub fn matches(&self, position: &Position) -> bool {
        self.player == position.player
            && self.left_cards == position.left_cards
            && self.right_cards == position.right_cards
            && self.declarer_cards == position.declarer_cards
            && self.trick_cards == position.trick_cards
    }
}

// -----------------------------------------------------------------------------
// TT TABLE
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct TranspositionTable {
    pub data: Vec<TranspositionEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(TT_SIZE);
        for _ in 0..TT_SIZE {
            data.push(TranspositionEntry {
                occupied: false,
                player: Player::Declarer, // Default
                left_cards: 0,
                right_cards: 0,
                declarer_cards: 0,
                trick_cards: 0,
                value: 0,
                flag: TranspositionFlag::Exact,
                bestcard: 0,
            });
        }
        Self { data }
    }

    pub fn write(
        &mut self,
        position: &Position,
        mapped_hash: usize,
        alpha: u8,
        beta: u8,
        value: (u32, u8),
    ) {
        let flag = match value.1 {
            x if x <= alpha => TranspositionFlag::Upper,
            x if x >= beta => TranspositionFlag::Lower,
            _ => TranspositionFlag::Exact,
        };

        // SAFETY: u8 fits in i16
        // Logic: Store Remaining Value (Total - Accumulated)
        let stored_val = if value.1 > position.declarer_points {
            (value.1 - position.declarer_points) as i16
        } else {
            0
        };

        let entry = TranspositionEntry {
            occupied: true,
            player: position.player,
            left_cards: position.left_cards,
            right_cards: position.right_cards,
            declarer_cards: position.declarer_cards,
            trick_cards: position.trick_cards,
            value: stored_val,
            bestcard: value.0,
            flag,
        };

        let old = &self.data[mapped_hash];
        let replace = if !old.occupied {
            true
        } else {
            entry.flag == TranspositionFlag::Exact || old.flag != TranspositionFlag::Exact
        };

        if replace {
            self.data[mapped_hash] = entry;
        }
    }

    pub fn read(&self, position: &Position, cnt: &mut Counters) -> Option<&TranspositionEntry> {
        let candidate = &self.data[position.get_hash()];

        if !candidate.occupied {
            None
        } else if candidate.matches(position) {
            cnt.inc_reads();
            Some(candidate)
        } else {
            cnt.inc_collisions();
            None
        }
    }

    // New methods for Optimum Search (Score based i16)
    pub fn write_optimum(
        &mut self,
        position: &Position,
        mapped_hash: usize,
        alpha: i16,
        beta: i16,
        value: (u32, i16),
    ) {
        // optimum mode uses i16 scores directly
        let flag = match value.1 {
            x if x <= alpha => TranspositionFlag::Upper,
            x if x >= beta => TranspositionFlag::Lower,
            _ => TranspositionFlag::Exact,
        };

        // For Optimum Search, score is position-dependent (depth included in score),
        // but 'depth' is fixed for a given card configuration in Skat.
        // So we can store absolute score.
        let entry = TranspositionEntry {
            occupied: true,
            player: position.player,
            left_cards: position.left_cards,
            right_cards: position.right_cards,
            declarer_cards: position.declarer_cards,
            trick_cards: position.trick_cards,
            value: value.1,
            bestcard: value.0,
            flag,
        };

        let old = &self.data[mapped_hash];
        let replace = if !old.occupied {
            true
        } else {
            entry.flag == TranspositionFlag::Exact || old.flag != TranspositionFlag::Exact
        };

        if replace {
            self.data[mapped_hash] = entry;
        }
    }

    pub fn read_optimum(
        &self,
        position: &Position,
        cnt: &mut Counters,
    ) -> Option<&TranspositionEntry> {
        // Re-use standard read matching
        self.read(position, cnt)
    }

    pub fn get_occupied_slots(&self) -> usize {
        self.data.iter().filter(|e| e.occupied).count()
    }

    pub fn add_entries(&mut self, other: TranspositionTable) {
        for i in 0..TT_SIZE {
            if !self.data[i].occupied && other.data[i].occupied {
                self.data[i] = other.data[i];
            }
        }
    }
}
