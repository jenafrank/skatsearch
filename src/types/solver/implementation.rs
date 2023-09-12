use super::Solver;
use crate::consts::bitboard::ALLCARDS;
use crate::traits::Augen;
use crate::traits::Bitboard;
use crate::types::game::Game;
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
        is_alpha_beta_accelerating: bool,
        is_winning_only: bool,
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

                if is_alpha_beta_accelerating {
                    if alpha_with_skat > skat.__get_value() {
                        current_initial_state.alpha = alpha_with_skat - skat.__get_value();
                    }
                } else if is_winning_only {
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
    pub fn solve_all_cards(&mut self) -> [Option<(u32, u32, u8)>; 10] {
        let initial_state = State::create_initial_state_from_problem(&self.problem);        
        self.get_all_cards(initial_state)
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

    pub fn solve(&mut self) -> (u32, u8) {
        let state = State::create_initial_state_from_problem(&self.problem);
        let result = self.get(state);

        println!(" Iters: {}, Slots: {}, Writes: {}, Reads: {}, ExactReads: {}, Collisions: {}, Breaks: {}",
        self.problem.counters.iters,
        self.problem.transposition_table.get_occupied_slots(),
        self.problem.counters.writes,
        self.problem.counters.reads,
        self.problem.counters.exactreads,
        self.problem.counters.collisions,
        self.problem.counters.breaks);

        result
    }

}
