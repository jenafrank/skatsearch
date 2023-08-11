use crate::core_functions::get_mapped_hash::get_mapped_hash;
use crate::types::problem::Problem;
use crate::types::state_transposition_table::StateTranspositionTable;

impl StateTranspositionTable {

    pub fn create_child_state(&self,
                              played_card: u32,
                              problem: &Problem,
                              alpha_start: u8,
                              beta_start: u8) -> (StateTranspositionTable, Option<bool>) {

        let base_child_state = self.state.create_child_state(played_card, problem);

        let trick_won: Option<bool> =
            if self.state.augen_declarer < base_child_state.augen_declarer {
                Some(true)
            } else if self.state.augen_team < base_child_state.augen_team {
                Some(false)
            } else {
                None
            };

        ( StateTranspositionTable {
            state: base_child_state,
            alpha: alpha_start,
            beta: beta_start,
            mapped_hash: get_mapped_hash(
                base_child_state.player,
                base_child_state.get_all_unplayed_cards(),
                base_child_state.trick_cards),
            is_root_state: false
        }, trick_won )
    }
}
