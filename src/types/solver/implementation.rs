use std::time::Instant;

use super::playout_row::PlayoutRow;
use super::Solver;
use crate::consts::bitboard::ALLCARDS;
use crate::traits::Augen;
use crate::traits::Bitboard;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::state::State;

impl Solver {
    /// Analyses a twelve-card hand during the task of putting away two cards (Skat) before
    /// game starts. It analyses all 66 cases and calculating the best play for each of them
    /// using the same transposition table for speed-up reasons.
    /// # Variants of arguments
    /// * _false_ _false_: Returns exact values for all 66 games.
    /// * _true_ _false_: Returns best skat by means of alpha-window narrowing. Thus, the
    /// 66-array does also contain wrong values.
    /// * _false_ _true_: Returns some skat for which the game will be won.
    /// The routine always takes into account the value of the skat which is neglected by default
    /// in the basic search routines.
    pub fn solve_with_skat(
        &mut self,
        is_alpha_beta: bool,
        is_winning: bool,
    ) -> (u32, u32, [((u32, u32), u8); 66]) {
        let mut ret: (u32, u32, [((u32, u32), u8); 66]) = (0, 0, [((0, 0), 0); 66]);

        let state = State::create_initial_state_from_problem(&self.problem);
        let remaining_cards = !state.get_all_unplayed_cards();
        let twelve_bit = remaining_cards | state.declarer_cards;
        let twelve = twelve_bit.__decompose_twelve();

        let mut k = 0;
        let mut alpha_with_skat = 0;
        for i in 0..11 {
            for j in i + 1..12 {
                let skat = twelve[i] | twelve[j];
                let declarer_cards = twelve_bit ^ skat;

                let current_all_cards = ALLCARDS ^ skat;

                self.problem.declarer_cards_all = declarer_cards;
                self.problem.augen_total = current_all_cards.__get_value();
                self.problem.nr_of_cards = current_all_cards.__get_number_of_bits();

                let mut current_initial_state =
                    State::create_initial_state_from_problem(&self.problem);

                if is_alpha_beta {
                    if alpha_with_skat > skat.__get_value() {
                        current_initial_state.alpha = alpha_with_skat - skat.__get_value();
                    }
                } else if is_winning {
                    if alpha_with_skat >= 61 {
                        return ret;
                    }

                    current_initial_state.alpha = 60 - skat.__get_value();
                    current_initial_state.beta = current_initial_state.alpha + 1;
                }

                let result = self.problem.search(&current_initial_state);
                let value_with_skat = result.1 + skat.__get_value();

                ret.2[k].0 .0 = twelve[i];
                ret.2[k].0 .1 = twelve[j];
                ret.2[k].1 = value_with_skat;

                if value_with_skat > alpha_with_skat {
                    ret.0 = twelve[i];
                    ret.1 = twelve[j];

                    alpha_with_skat = value_with_skat;
                }

                k += 1;
            }
        }

        ret
    }

    /// Investigates all legal moves for a given state and returns an option array
    /// with 0) card under investigation 1) follow-up card from tree search (tree root) and
    /// 2) value of search
    pub fn solve_all_cards(&mut self, state_tt: &State) -> [Option<(u32, u32, u8)>; 10] {
        let mut ret: [Option<(u32, u32, u8)>; 10] = [None; 10];
        let legal_moves = state_tt.get_legal_moves().__decompose();

        for i in 0..legal_moves.1 {
            let card = legal_moves.0[i];
            let state_adv = state_tt.create_child_state(card, &self.problem, 0, 120);
            let res = self.problem.search(&state_adv);
            ret[i] = Some((card, res.0, res.1));
        }

        ret
    }

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
    pub fn playout_all_cards(
        &mut self,
    ) -> [(u32, Player, u8, [Option<(u32, u32, u8)>; 10]); 30] {

        let mut ret: [(u32, Player, u8, [Option<(u32, u32, u8)>; 10]); 30] =
            [(0, Player::Declarer, 0, [None; 10]); 30];
        let mut i: usize = 0;
        let n: usize = self.problem.nr_of_cards as usize;
        
        let mut state = State::create_initial_state_from_problem(&self.problem);

        while i < n {
            self.problem.counters.iters = 0;
            self.problem.counters.breaks = 0;

            let res = self.problem.search(&state);
            let resall = self.solve_all_cards(&state);

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

    pub fn solve_win(&mut self) -> (u8, u32, u32) {
        let mut state = State::create_initial_state_from_problem(&self.problem);

        let skat_value = self.problem.get_skat().__get_value();

        match self.problem.game_type {
            Game::Farbe => {
                state.alpha = 60 - skat_value;
                state.beta = 61 - skat_value;
            }
            Game::Grand => {
                state.alpha = 60 - skat_value;
                state.beta = 61 - skat_value;
            }
            Game::Null => {
                state.alpha = 0;
                state.beta = 1;
            }
        }

        let val = self.problem.search(&state).1;

        (
            val,
            self.problem.counters.iters,
            self.problem.counters.collisions,
        )
    }

    pub fn solve_double_dummy(&mut self) -> (u8, u32, u32) {
        let mut val = 0;
        let mdf = 5u8;

        for i in 0..119 {
            let mut state = State::create_initial_state_from_problem(&self.problem);
            state.alpha = mdf * i;
            state.beta = mdf * (i + 1);
            val = self.problem.search(&state).1;

            if val < state.beta {
                break;
            }
        }

        (
            val,
            self.problem.counters.iters,
            self.problem.counters.collisions,
        )
    }

    pub fn solve(&mut self) -> u8 {
        let state = State::create_initial_state_from_problem(&self.problem);
        let res = self.problem.search(&state);
        let val = res.1;
        println!(" Iters: {}, Slots: {}, Writes: {}, Reads: {}, ExactReads: {}, Collisions: {}, Breaks: {}",
        self.problem.counters.iters,
        self.problem.transposition_table.get_occupied_slots(),
        self.problem.counters.writes,
        self.problem.counters.reads,
        self.problem.counters.exactreads,
        self.problem.counters.collisions,
        self.problem.counters.breaks);

        val
    }
}
