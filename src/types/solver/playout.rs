use std::time::Instant;
use crate::types::{state::State, player::Player};
use super::{Solver, playout_row::PlayoutRow};

impl Solver {

       /// Generates playout.
       pub fn playout(&mut self) -> [Option<PlayoutRow>; 30] {
        let mut ret: [Option<PlayoutRow>; 30] = [None; 30];
        let mut i: usize = 0;
        let n: usize = self.problem.nr_of_cards as usize;

        let mut initial_state = State::create_initial_state_from_problem(&self.problem);

        while i < n {
            let mut row: PlayoutRow = Default::default();

            row.declarer_cards = initial_state.declarer_cards;
            row.left_cards = initial_state.left_cards;
            row.right_cards = initial_state.right_cards;

            self.problem.counters.iters = 0;
            self.problem.counters.breaks = 0;

            let now = Instant::now();
            let res = self.problem.search(&initial_state);
            let time = now.elapsed().as_millis();

            let played_card = res.0;

            row.player = initial_state.player;
            row.card = played_card;
            row.augen_declarer = initial_state.augen_declarer;
            row.augen_team = initial_state.augen_team;
            row.cnt_iters = self.problem.counters.iters;
            row.cnt_breaks = self.problem.counters.breaks;
            row.time = time;

            initial_state = initial_state.create_child_state(
                played_card,
                &self.problem,
                initial_state.alpha,
                initial_state.beta,
            );

            ret[i] = Some(row);
            i += 1;
        }

        ret
    }

    /// Generates playout with all values for each card..
    pub fn playout_all_cards(&mut self) -> [(u32, Player, u8, [Option<(u32, u32, u8)>; 10]); 30] {

        let mut ret: [(u32, Player, u8, [Option<(u32, u32, u8)>; 10]); 30] =
            [(0, Player::Declarer, 0, [None; 10]); 30];
        let mut i: usize = 0;
        let n: usize = self.problem.nr_of_cards as usize;
        
        let mut state = State::create_initial_state_from_problem(&self.problem);

        while i < n {
            self.problem.counters.iters = 0;
            self.problem.counters.breaks = 0;

            let res = self.get(state);
            let resall = self.get_all_cards(state);

            let played_card = res.0;
            ret[i].1 = state.player;

            state = state.create_child_state(
                played_card,
                &self.problem,
                state.alpha,
                state.beta,
            );

            ret[i].0 = played_card;
            ret[i].2 = state.augen_declarer;

            for (j, el) in resall.iter().flatten().enumerate() {
                ret[i].3[j] = Some((el.0, el.1, el.2));
            }

            i += 1;
        }

        ret
    }

}