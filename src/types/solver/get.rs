use crate::{types::{state::State, counter::Counters}, traits::Bitboard};

use super::{Solver, retargs::{SolveRet, SolveAllLineRetArgs, SolveAllCardsRet}};

impl Solver {
    
    pub fn get_all_cards(&mut self, state: State, alpha: u8, beta: u8) -> SolveAllCardsRet {
        
        let mut cnt = Counters::new();
        let legal_moves = state.get_legal_moves().__decompose();
    
        let card_list: Vec<SolveAllLineRetArgs> = legal_moves.0.iter()
            .take(legal_moves.1)
            .map(|&card| {
                let state_adv = state.create_child_state(card, &self.problem, alpha, beta);
                let res = self.problem.search(&state_adv, &mut self.tt, &mut cnt);
                SolveAllLineRetArgs {
                    investigated_card: card,
                    best_follow_up_card: res.0,
                    value: res.1,
                }
            })
            .collect();
    
        SolveAllCardsRet {
            card_list,
            counters: cnt,
        }
    }    

    pub fn get(&mut self, state: State) -> SolveRet {        
        let mut cnt = Counters::new();
        let result = self.problem.search(&state, &mut self.tt, &mut cnt);
        SolveRet { best_card: result.0, best_value: result.1, counters: cnt }
    }

}