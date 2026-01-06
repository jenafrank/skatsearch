use super::{playout_row::PlayoutLine, retargs::PlayoutAllCardsRetLine, Solver};
use crate::types::{counter::Counters, state::State};
use std::time::Instant;

impl Solver {
    /// Generates playout.
    pub fn playout(&mut self) -> Vec<PlayoutLine> {
        let mut ret: Vec<PlayoutLine> = Vec::new();
        let mut i: usize = 0;
        let n: usize = self.problem.number_of_cards() as usize;

        let mut initial_state = State::create_initial_state_from_problem(&self.problem);

        while i < n {
            let mut row: PlayoutLine = Default::default();

            row.declarer_cards = initial_state.declarer_cards;
            row.left_cards = initial_state.left_cards;
            row.right_cards = initial_state.right_cards;

            let mut cnt: Counters = Counters::new();

            let now = Instant::now();
            let mut alpha = 0;
            let mut beta = 120;
            let res = self
                .problem
                .search(&initial_state, &mut self.tt, &mut cnt, alpha, beta);
            let time = now.elapsed().as_millis();

            let played_card = res.0;

            row.player = initial_state.player;
            row.card = played_card;
            row.augen_declarer = initial_state.augen_declarer;
            row.augen_team = initial_state.augen_team;
            row.cnt_iters = cnt.iters;
            row.cnt_breaks = cnt.breaks;
            row.time = time;

            initial_state = initial_state.create_child_state(played_card, &self.problem);

            ret.push(row);
            i += 1;
        }

        ret
    }

    /// Generates playout with all values for each card..
    pub fn playout_all_cards(&mut self) -> Vec<PlayoutAllCardsRetLine> {
        let mut ret: Vec<PlayoutAllCardsRetLine> = Vec::new();
        let mut i: usize = 0;
        let n: usize = self.problem.number_of_cards() as usize;

        let mut state = State::create_initial_state_from_problem(&self.problem);

        while i < n {
            let mut row: PlayoutAllCardsRetLine = Default::default();

            let mut cnt: Counters = Counters::new();

            let res = self.get(state);
            let resall = self.get_all_cards(state, 0, 120);

            cnt.add(res.counters);
            cnt.add(resall.counters);

            let best_card = res.best_card;
            row.player = state.player;

            state = state.create_child_state(best_card, &self.problem);

            row.best_card = best_card;
            row.augen_declarer = state.augen_declarer;

            row.all_cards = resall;

            ret.push(row);

            i += 1;
        }

        ret
    }
}
