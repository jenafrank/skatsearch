use std::time::Instant;
use crate::types::{state::State, counter::Counters};
use super::{Solver, playout_row::PlayoutLine, retargs::PlayoutAllCardsRetLine};

impl Solver {

       /// Generates playout.
       pub fn playout(&mut self) -> Vec<PlayoutLine> {
        
        let mut ret: Vec<PlayoutLine> = Vec::new();
        let mut i: usize = 0;
        let n: usize = self.problem.nr_of_cards as usize;

        let mut initial_state = State::create_initial_state_from_problem(&self.problem);

        while i < n {
            let mut row: PlayoutLine = Default::default();

            row.declarer_cards = initial_state.declarer_cards;
            row.left_cards = initial_state.left_cards;
            row.right_cards = initial_state.right_cards;

            Counters::reset();

            let now = Instant::now();
            let res = self.problem.search(&initial_state);
            let time = now.elapsed().as_millis();

            let played_card = res.0;

            row.player = initial_state.player;
            row.card = played_card;
            row.augen_declarer = initial_state.augen_declarer;
            row.augen_team = initial_state.augen_team;
            row.cnt_iters = Counters::get().iters;
            row.cnt_breaks = Counters::get().breaks;
            row.time = time;

            initial_state = initial_state.create_child_state(
                played_card,
                &self.problem,
                initial_state.alpha,
                initial_state.beta,
            );

            ret.push(row);
            i += 1;
        }

        ret
    }

    /// Generates playout with all values for each card..
    pub fn playout_all_cards(&mut self) -> Vec<PlayoutAllCardsRetLine> {

        let mut ret: Vec<PlayoutAllCardsRetLine> = Vec::new();
        let mut i: usize = 0;
        let n: usize = self.problem.nr_of_cards as usize;
        
        let mut state = State::create_initial_state_from_problem(&self.problem);

        while i < n {

            let mut row: PlayoutAllCardsRetLine = Default::default();

            Counters::reset();

            let res = self.get(state);
            let resall = self.get_all_cards(state);

            let best_card = res.best_card;
            row.player = state.player;

            state = state.create_child_state(
                best_card,
                &self.problem,
                state.alpha,
                state.beta,
            );

            row.best_card = best_card;
            row.augen_declarer = state.augen_declarer;

            row.all_cards = resall;

            ret.push(row);

            i += 1;
        }

        ret
    }

}