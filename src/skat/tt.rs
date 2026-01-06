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
    pub value: u8,

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

        let entry = TranspositionEntry {
            occupied: true,
            player: position.player,
            left_cards: position.left_cards,
            right_cards: position.right_cards,
            declarer_cards: position.declarer_cards,
            trick_cards: position.trick_cards,

            value: if value.1 > position.augen_declarer {
                value.1 - position.augen_declarer
            } else {
                0
            }, // Safety check for subtraction
            // Wait, original logic was: value.1 - state.augen_declarer.
            // value passed to write is (best_card, best_value_absolute).
            // State augen is accumulated so far.
            // Stored value is REMAINING value?
            // Let's re-verify the original logic.
            // Original: value: value.1 - state.augen_declarer
            // If value.1 is absolute total score, and augen_declarer is score so far, then YES, stored value is remaining score.
            // I'll keep it as is, but assuming u8 > 0.
            bestcard: value.0,
            flag,
        };

        let old = &self.data[mapped_hash];
        let replace = if !old.occupied {
            true
        } else {
            // New is exact -> Always replace
            // New is not exact -> Only replace if old is not exact
            entry.flag == TranspositionFlag::Exact || old.flag != TranspositionFlag::Exact
        };

        if replace {
            self.data[mapped_hash] = entry;
        }
    }

    pub fn read(&self, position: &Position, cnt: &mut Counters) -> Option<&TranspositionEntry> {
        // Position has get_hash()
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

    pub fn get_occupied_slots(&self) -> usize {
        self.data.iter().filter(|e| e.occupied).count()
    }

    // Original had add_entries for parallel merging?
    pub fn add_entries(&mut self, other: TranspositionTable) {
        for i in 0..TT_SIZE {
            if !self.data[i].occupied && other.data[i].occupied {
                self.data[i] = other.data[i];
            }
        }
    }
}
