use crate::types::problem::counters::Counters;
use crate::types::state::State;
use crate::types::tt_entry::TtEntry;
use crate::types::tt_flag::TtFlag;
use crate::types::tt_table::TtTable;
use crate::consts::general::TT_SIZE;
use crate::core_functions::get_mapped_hash::get_mapped_hash;

impl TtTable {
    pub fn write(&mut self, 
        state: &State, 
        mapped_hash: usize, 
        alpha: u8, 
        beta: u8, 
        value: (u32, u8)) {

        let flag: TtFlag =
            match value.1 {
                x if x <= alpha => TtFlag::UPPER,
                x if x >= beta => TtFlag::LOWER,
                _ => TtFlag::EXACT
            };

        let entry = TtEntry {
            occupied: true,
            player: state.player,
            cards: state.get_all_unplayed_cards(),
            value: value.1 - state.augen_declarer,
            bestcard: value.0,
            flag,
        };

        self.data[mapped_hash] = entry;
    }

    pub fn write_without_hash(&mut self, state: &State, alpha: u8, beta: u8, value: u8) {
        let idx = get_mapped_hash(state.player,state.get_all_unplayed_cards(), state.trick_cards);
        self.write(state, idx, alpha, beta, (0, value));
    }

    pub fn read(&self, state: &State, counters: &mut Counters) -> Option<&TtEntry> {

        let candidate = &self.data[state.mapped_hash];

        if !candidate.occupied {
            None // empty slot
        } else if candidate.matches(&state) {
            counters.reads += 1;
            Some(candidate) // matches key values
        } else {
            counters.collisions += 1;
            None // collision
        }
    }

    pub fn get_occupied_slots(&self) -> usize {
        let mut ret = 0usize;

        for i in 0..TT_SIZE {
            if self.data[i].occupied {
                ret += 1;
            }
        }

        ret
    }

    pub fn is_tt_compatible_state(state: &State) -> bool {
        state.is_not_root_state() && state.trick_cards_count == 0
    }
}