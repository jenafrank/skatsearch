mod search;
pub mod counters;
mod constructors;
mod playout_row;

use crate::types::problem::Problem;
use crate::types::problem_transposition_table::counters::CountersTranspositionTable;
use crate::types::tt_table::TtTable;

pub struct ProblemTranspositionTable {
    pub problem: Problem,
    pub transposition_table: TtTable,
    pub counters: CountersTranspositionTable
}

