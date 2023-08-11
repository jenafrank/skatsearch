use crate::types::state_transposition_table::StateTranspositionTable;

impl StateTranspositionTable {
    pub fn is_not_root_state(&self) -> bool {
        self.is_root_state == false
    }
}