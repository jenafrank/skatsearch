use crate::types::problem_transposition_table::ProblemTranspositionTable;
use crate::types::state::State;
use crate::types::state_transposition_table::StateTranspositionTable;

impl StateTranspositionTable {

    pub fn initial_state_from_problem(p0: &ProblemTranspositionTable) -> StateTranspositionTable {
        StateTranspositionTable {
            state: State::create_initial_state_from_problem(&p0.problem),
            alpha: 0,
            beta: 120,
            mapped_hash: 0,
            is_root_state: true
        }
    }

}
