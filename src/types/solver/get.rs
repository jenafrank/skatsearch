use crate::{types::{state::State, counter::Counters}, traits::Bitboard};

use super::{Solver, retargs::{SolveRet, SolveAllLineRetArgs, SolveAllCardsRet}};

impl Solver {
    
    pub fn get_all_cards(&mut self, state: State) -> SolveAllCardsRet {
        
        let mut ret: SolveAllCardsRet = SolveAllCardsRet { card_list: Vec::new(), counters: Counters::get() };

        let legal_moves = state.get_legal_moves().__decompose();

        for i in 0..legal_moves.1 {
            let card = legal_moves.0[i];
            let state_adv = state.create_child_state(card, &self.problem, 0, 120);
            let res = self.problem.search(&state_adv);
            ret.card_list.push(
                SolveAllLineRetArgs{
                    investigated_card: card,
                    best_follow_up_card: res.0,
                    value: res.1
                });                
        }

        ret.counters = Counters::get();

        ret
    }

    pub fn get(&mut self, state: State) -> SolveRet {        
        let result = self.problem.search(&state);
        SolveRet { best_card: result.0, best_value: result.1, counters: Counters::get() }
    }

}