use crate::types::state_transposition_table::StateTranspositionTable;
use super::*;

impl ProblemTranspositionTable {
    pub fn from_problem(problem: Problem) -> ProblemTranspositionTable {
        ProblemTranspositionTable {
            problem,
            transposition_table: Default::default(),
            counters: Default::default()
        }
    }

    pub fn initial_state(&mut self) -> StateTranspositionTable {
        StateTranspositionTable::initial_state_from_problem(self)
    }

    pub fn from_problem_with_initial_state(problem: Problem) -> (ProblemTranspositionTable, StateTranspositionTable) {

        let specific_problem = ProblemTranspositionTable::from_problem(problem);
        let initial_state = StateTranspositionTable::initial_state_from_problem(&specific_problem);

        (
            specific_problem,
            initial_state
        )

    }
}
