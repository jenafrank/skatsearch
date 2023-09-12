use crate::{types::state::State, traits::Bitboard};

use super::Solver;

impl Solver {
    
    pub fn get_all_cards(&mut self, state: State) -> [Option<(u32, u32, u8)>; 10] {
        let mut ret: [Option<(u32, u32, u8)>; 10] = [None; 10];

        let legal_moves = state.get_legal_moves().__decompose();

        for i in 0..legal_moves.1 {
            let card = legal_moves.0[i];
            let state_adv = state.create_child_state(card, &self.problem, 0, 120);
            let res = self.problem.search(&state_adv);
            ret[i] = Some((card, res.0, res.1));
        }

        ret
    }

    pub fn get(&mut self, state: State) -> (u32, u8) {        
        self.problem.search(&state)
    }

}