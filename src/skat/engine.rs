//! # Skat Engine
//!
//! The main entry point for the Skat solver.
//! Holds the game context and the transposition table.

use crate::skat::context::GameContext;
use crate::skat::counters::Counters;
use crate::skat::position::Position;
use crate::skat::search::search;
use crate::skat::tt::TranspositionTable;

pub struct SkatEngine {
    pub context: GameContext,
    pub tt: TranspositionTable,
}

impl SkatEngine {
    pub fn new(context: GameContext, tt: Option<TranspositionTable>) -> Self {
        Self {
            context,
            tt: tt.unwrap_or_else(TranspositionTable::new),
        }
    }

    pub fn search(
        &mut self,
        position: &Position,
        cnt: &mut Counters,
        alpha: u8,
        beta: u8,
    ) -> (u32, u8) {
        search(&self.context, position, &mut self.tt, cnt, alpha, beta)
    }

    pub fn create_initial_position(&self) -> Position {
        self.context.create_initial_position()
    }
}
