mod constructors;
mod methods;
mod methods_create_child_state;

use crate::types::state::State;

pub struct StateTranspositionTable {
    pub state: State,
    pub alpha: u8,
    pub beta: u8,
    pub mapped_hash: usize,
    pub is_root_state: bool
}


