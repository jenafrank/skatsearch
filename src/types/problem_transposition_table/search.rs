use std::cmp;
use std::time::Instant;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::problem_transposition_table::{CountersTranspositionTable, ProblemTranspositionTable};
use crate::types::state_transposition_table::*;
use crate::types::tt_flag::TtFlag;
use crate::types::tt_table::TtTable;
use crate::core_functions::get_sorted_by_value::get_sorted_by_value;
use crate::traits::{Augen, Bitboard};
use crate::types::problem_transposition_table::playout_row::PlayoutRow;

impl ProblemTranspositionTable {

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
    pub fn get_all_skat_values(&mut self, is_alpha_beta: bool, is_winning: bool) -> (u32, u32, [((u32, u32), u8); 66])
    {

        let mut ret : (u32, u32, [((u32, u32), u8); 66]) = (0,0,[((0,0),0); 66]);

        let state = StateTranspositionTable::initial_state_from_problem(self);
        let remaining_cards = !state.state.get_all_unplayed_cards();
        let twelve_bit = remaining_cards | state.state.declarer_cards;
        let twelve = twelve_bit.__decompose_twelve();

        let mut k = 0;
        let mut alpha_with_skat = 0;
        for i in 0..11 {
            for j in i+1..12 {
                let skat = twelve[i] | twelve[j];
                let declarer_cards = twelve_bit ^ skat;

                let partial_problem = Problem::create(
                    declarer_cards,
                self.problem.left_cards_all,
                self.problem.right_cards_all,
                self.problem.game_type,
                self.problem.start_player);

                self.problem = partial_problem;
                let mut partial_state = StateTranspositionTable::initial_state_from_problem(self);

                if is_alpha_beta {
                    if alpha_with_skat > skat.__get_value() {
                        partial_state.alpha = alpha_with_skat - skat.__get_value();
                    }
                } else if is_winning {

                    if alpha_with_skat >= 61 {
                        return ret;
                    }

                    partial_state.alpha = 60 - skat.__get_value();
                    partial_state.beta = partial_state.alpha + 1;
                }

                let result = self.search(&partial_state);
                let value_with_skat = result.1 + skat.__get_value();

                ret.2[k].0.0 = twelve[i];
                ret.2[k].0.1 = twelve[j];
                ret.2[k].1 = value_with_skat;

                if value_with_skat > alpha_with_skat {
                    ret.0 = twelve[i];
                    ret.1 = twelve[j];

                    alpha_with_skat = value_with_skat;
                }

                k+= 1;
            }
        }

        ret
    }

    /// Checks if winning without determining the correct value (alpha = 60, beta = 61)
    pub fn search_if_declarer_is_winning(&mut self, state_tt: &mut StateTranspositionTable) -> bool {
        state_tt.alpha = 60;
        state_tt.beta = 61;

        let result = self.search(&state_tt);

        result.1 >= 61
    }

    /// Investigates all legal moves for a given state and returns an option array
    /// with 0) card under investigation 1) follow-up card from tree search (tree root) and
    /// 2) value of search
    pub fn get_allvalues(&mut self, state_tt: &StateTranspositionTable)
    -> [Option<(u32,u32,u8)>; 10]
    {
        let mut ret: [Option<(u32, u32, u8)>; 10] = [None; 10];
        let legal_moves = state_tt.state.get_legal_moves().__decompose();

        for i in 0..legal_moves.1 {
            let card = legal_moves.0[i];
            let state_adv =
                state_tt.create_child_state(card,&self.problem,0,120);
            let res = self.search(&state_adv);
            ret[i] = Some((card, res.0, res.1));
        }

        ret
    }

    /// Generates playout.
    pub fn get_playout(problem: Problem) -> [Option<PlayoutRow>; 30] {

        let mut ret: [Option<PlayoutRow>; 30] = [None; 30];
        let mut i: usize= 0;
        let n: usize = problem.nr_of_cards as usize;

        let mut problem_tt = ProblemTranspositionTable::from_problem(problem);
        let mut state_tt = StateTranspositionTable::initial_state_from_problem(&problem_tt);

        while i < n {

            let mut row : PlayoutRow = Default::default();

            row.declarer_cards = state_tt.state.declarer_cards;
            row.left_cards = state_tt.state.left_cards;
            row.right_cards = state_tt.state.right_cards;

            problem_tt.counters.cnt_iters = 0;
            problem_tt.counters.cnt_breaks = 0;

            let now = Instant::now();
            let res = problem_tt.search(&state_tt);
            let time = now.elapsed().as_millis();

            let played_card = res.0;

            row.player = state_tt.state.player;
            row.card = played_card;
            row.augen_declarer = state_tt.state.augen_declarer;
            row.augen_team = state_tt.state.augen_team;
            row.cnt_iters = problem_tt.counters.cnt_iters;
            row.cnt_breaks = problem_tt.counters.cnt_breaks;
            row.time = time;

            state_tt = state_tt.create_child_state(
                played_card,
                &(problem_tt.problem),
                state_tt.alpha,
                state_tt.beta);

            ret[i] = Some(row);
            i += 1;
        }

        ret
    }

    /// Generates playout with all values for each card..
    pub fn get_allvalues_playout(problem: Problem) -> [(u32, Player, u8, [Option<(u32, u32, u8)>; 10]); 30] {

        let mut ret: [(u32, Player, u8, [Option<(u32, u32, u8)>; 10]); 30] = [(0, Player::Declarer, 0, [None; 10]) ;30];
        let mut i: usize= 0;
        let n: usize = problem.nr_of_cards as usize;

        let mut problem_tt = ProblemTranspositionTable::from_problem(problem);
        let mut state_tt = StateTranspositionTable::initial_state_from_problem(&problem_tt);

        while i < n {

            problem_tt.counters.cnt_iters = 0;
            problem_tt.counters.cnt_breaks = 0;

            let res = problem_tt.search(&state_tt);
            let resall = problem_tt.get_allvalues(&state_tt);

            let played_card = res.0;
            ret[i].1 = state_tt.state.player;

            state_tt = state_tt.create_child_state(
                played_card,
                &(problem_tt.problem),
                state_tt.alpha,
                state_tt.beta);

            ret[i].0 = played_card;
            ret[i].2 = state_tt.state.augen_declarer;

            for (j, el) in resall.iter().flatten().enumerate() {
                ret[i].3[j] = Some((el.0, el.1, el.2));
            }

            i += 1;
        }

        ret
    }

    pub fn search_win_loss(problem: Problem) -> (u8, u32, u32) {
        let mut problem = ProblemTranspositionTable::from_problem(problem);
        let mut state = StateTranspositionTable::initial_state_from_problem(&problem);

        let skat_value = problem.problem.get_skat().__get_value();

        match problem.problem.game_type {
            Game::Farbe => {state.alpha = 60 - skat_value; state.beta = 61 - skat_value;}
            Game::Grand => {state.alpha = 60 - skat_value; state.beta = 61 - skat_value;}
            Game::Null => {state.alpha = 0; state.beta = 1;}
        }

        let val = problem.search(&state).1;

        (val, problem.counters.cnt_iters, problem.counters.cnt_collisions)
    }

    pub fn search_with_problem_using_double_dummy_solver(problem: Problem) -> (u8, u32, u32) {
        let mut problem = ProblemTranspositionTable::from_problem(problem);
        let mut val = 0;
        let mdf = 5u8;

        for i in 0..119 {
            let mut state = StateTranspositionTable::initial_state_from_problem(&problem);
            state.alpha = mdf*i;
            state.beta = mdf*(i+1);
            val = problem.search(&state).1;

            if val < state.beta {
                break;
            }
        }

        (val, problem.counters.cnt_iters, problem.counters.cnt_collisions)
    }

    pub fn search_with_problem(problem: Problem) -> u8 {
        let mut problem = ProblemTranspositionTable::from_problem(problem);
        let state = StateTranspositionTable::initial_state_from_problem(&problem);
        let res = problem.search(&state);
        let val = res.1;
        println!(" Iters: {}, Slots: {}, Writes: {}, Reads: {}, ExactReads: {}, Collisions: {}, Breaks: {}",
                 problem.counters.cnt_iters,
                 problem.transposition_table.get_occupied_slots(),
                 problem.counters.cnt_writes,
                 problem.counters.cnt_reads,
                 problem.counters.cnt_exactreads,
                 problem.counters.cnt_collisions,
                 problem.counters.cnt_breaks);

        val
    }

    pub fn search(&mut self, state_trans_table: &StateTranspositionTable) -> (u32, u8, Option<bool>) {

        self.counters.cnt_iters += 1;

        // BASIC: Termination of recursive search
        if let Some(x) = apply_termination_criteria(&self.problem, &state_trans_table) {
            return (0, x, None);
        }

        let state = state_trans_table.state;
        let mut alpha = state_trans_table.alpha;
        let mut beta = state_trans_table.beta;
        let mut optimized_value: (u32, u8, Option<bool>) = (0, get_value_to_optimize(state.player,self.problem.game_type), None);

        // TRANS:
        if let Some(x) = transposition_table_lookup(
            &self.transposition_table,
            &state_trans_table,
            &mut self.counters,
            &mut alpha,
            &mut beta
        ) {
            return x;
        }

        // TRANS: Freeze alpha and beta for tt entry later on
        let alphaorig = alpha;
        let betaorig = beta;

        // BASIC: Reduce moves, sort moves, find connections
        let moves_word = state.get_reduced(&self.problem);
        let (moves, n) = get_sorted_by_value(moves_word);

        // BASIC: Branching loop
        for mov in &moves[0..n] {

            // BASIC: Generate child state
            let child_state = state_trans_table.create_child_state(
                *mov,
                &self.problem,
                alpha,
                beta);

            // BASIC: Search child state
            let child_state_value = self.search(&child_state);

            // Optimize value
            optimized_value = optimize(child_state_value, optimized_value, state.player, *mov,self.problem.game_type);

            // Alpha-beta cutoffs
            if shrink_alpha_beta_window(state.player, &mut alpha, &mut beta, child_state_value.1, self.problem.game_type) {
                self.counters.cnt_breaks += 1;
                break;
            }
        }

        transposition_table_write(
            self,
            &state_trans_table,
            alphaorig,
            betaorig,
            optimized_value
        );

        optimized_value
    }
}

fn optimize(child_state_value: (u32, u8, Option<bool>),
            optimized_value: (u32, u8, Option<bool>),
            player: Player, mov: u32, game: Game) -> (u32, u8, Option<bool>) {

    match game {
        Game::Null => {
            match player {
                Player::Declarer => if child_state_value.1 < optimized_value.1
                {
                    let mut ret = child_state_value;
                    ret.0 = mov;
                    ret
                }
                else
                {
                    optimized_value
                },

                _ => if child_state_value.1 > optimized_value.1
                {
                    let mut ret = child_state_value;
                    ret.0 = mov;
                    ret
                }
                else
                {
                    optimized_value
                }
            }

        }
        _ => {
            match player {
                Player::Declarer => if child_state_value.1 > optimized_value.1
                {
                    let mut ret = child_state_value;
                    ret.0 = mov;
                    ret
                }
                else
                {
                    optimized_value
                },

                _ => if child_state_value.1 < optimized_value.1
                {
                    let mut ret = child_state_value;
                    ret.0 = mov;
                    ret
                }
                else
                {
                    optimized_value
                }
            }
        }
    }


}

fn get_value_to_optimize(player: Player, game: Game) -> u8 {
    match player  {
        Player::Declarer => {
            match game {
                Game::Farbe => 0,
                Game::Grand => 0,
                Game::Null => 1
            }
        },
        _ => {
            match game {
                Game::Farbe => 120,
                Game::Grand => 120,
                Game::Null => 0
            }
        }
    }
}

#[inline(always)]
fn apply_termination_criteria(problem: &Problem, state_trans_table: &StateTranspositionTable) -> Option<u8> {

    let state = state_trans_table.state;

    /* 1. Termination criteria: Return if no cards anymore available */
    if state.player_cards == 0 {
        return Some(state.augen_declarer);
    }

    /* 2. Termination criteria: Check ab window */
    match problem.game_type {
        Game::Null => {
            if state.augen_declarer > 0 {
                return Some(1);
            }
        }
        _ => {
            if problem.augen_total - state.augen_team <= state_trans_table.alpha {
                return Some(state_trans_table.alpha);
            }

            if state.augen_declarer >= state_trans_table.beta {
                return Some(state_trans_table.beta);
            }
        }
    }

    return None;
}

#[inline(always)]
fn transposition_table_lookup(
    tt: &TtTable,
    state_tt: &StateTranspositionTable,
    counters: &mut CountersTranspositionTable,
    alpha: &mut u8,
    beta: &mut u8
) -> Option<(u32, u8, Option<bool>)>
{

    if TtTable::is_tt_compatible_state(state_tt) {
        if let Some(tt_entry) = tt.read(state_tt, counters) {
            let value = tt_entry.value + state_tt.state.augen_declarer;
            let trickwon = tt_entry.trickwon;
            let bestcard = tt_entry.bestcard;
            match tt_entry.flag {
                TtFlag::EXACT => {
                    counters.cnt_exactreads += 1;
                    return Some((bestcard,value,trickwon));
                },
                TtFlag::LOWER => {
                    *alpha = cmp::max(*alpha, value);
                },
                TtFlag::UPPER => {
                    *beta = cmp::min(*beta, value);
                }
            }
            if *alpha >= *beta {
                return Some((bestcard,value,trickwon));
            }
        }
    }

    None
}

#[inline(always)]
fn transposition_table_write(
    problem_tt: &mut ProblemTranspositionTable,
    state_tt: &StateTranspositionTable,
    alphaorig: u8,
    betaorig: u8,
    value: (u32, u8, Option<bool>)
) {
    if TtTable::is_tt_compatible_state(state_tt) {
        problem_tt.counters.cnt_writes += 1;
        problem_tt.transposition_table.write(
            &state_tt.state,
            state_tt.mapped_hash,
            alphaorig,
            betaorig,
            value
        );
    }
}

#[inline(always)]
fn shrink_alpha_beta_window(player: Player, alpha: &mut u8, beta: &mut u8, child_state_value: u8, game: Game) -> bool {

    match game {
        Game::Null => {
            match player {
                Player::Declarer => {
                    *beta = cmp::min(*beta, child_state_value);
                    if *beta <= *alpha {
                        return true;
                    }
                },
                _ => {
                    *alpha = cmp::max(*alpha, child_state_value);
                    if *alpha >= *beta {
                        return true;
                    }
                }
            }
        }
        _ => {
            match player {
                Player::Declarer => {
                    *alpha = cmp::max(*alpha, child_state_value);
                    if *alpha >= *beta {
                        return true;
                    }
                },
                _ => {
                    *beta = cmp::min(*beta, child_state_value);
                    if *beta <= *alpha {
                        return true;
                    }
                }
            }
        }
    }

    false
}
