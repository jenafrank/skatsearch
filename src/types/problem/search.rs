use std::cmp;
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::state::*;
use crate::types::tt_flag::TtFlag;
use crate::types::tt_table::TtTable;
use crate::core_functions::get_sorted_by_value::get_sorted_by_value;

impl Problem {

    pub fn search(&mut self, state: &State) -> (u32, u8) {

        Counters::inc_iters();

        // BASIC: Termination of recursive search
        if let Some(x) = apply_termination_criteria(&self, &state) {
            return (0, x);
        }
        
        let mut alpha = state.alpha;
        let mut beta = state.beta;
        let mut optimized_value: (u32, u8) = (0, get_value_to_optimize(state.player,self.game_type));

        // TRANS:
        if let Some(x) = transposition_table_lookup(
            &self.transposition_table,
            &state,
            &mut alpha,
            &mut beta
        ) {
            return x;
        }

        // TRANS: Freeze alpha and beta for tt entry later on
        let alphaorig = alpha;
        let betaorig = beta;

        // BASIC: Reduce moves, sort moves, find connections
        let moves_word = state.get_reduced(&self);
        let (moves, n) = get_sorted_by_value(moves_word);

        // BASIC: Branching loop
        for mov in &moves[0..n] {

            // BASIC: Generate child state
            let child_state = state.create_child_state(
                *mov,
                &self,
                alpha,
                beta);

            // BASIC: Search child state
            let child_state_value = self.search(&child_state);

            // Optimize value
            optimized_value = optimize(child_state_value, optimized_value, state.player, *mov,self.game_type);

            // Alpha-beta cutoffs
            if shrink_alpha_beta_window(state.player, &mut alpha, &mut beta, child_state_value.1, self.game_type) {
                Counters::inc_breaks();
                break;
            }
        }

        transposition_table_write(
            self,
            &state,
            alphaorig,
            betaorig,
            optimized_value
        );

        optimized_value
    }
}

fn optimize(child_state_value: (u32, u8),
            optimized_value: (u32, u8),
            player: Player, mov: u32, game: Game) -> (u32, u8) {

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
fn apply_termination_criteria(problem: &Problem, state: &State) -> Option<u8> {

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
            if problem.augen_total - state.augen_team <= state.alpha {
                return Some(state.alpha);
            }

            if state.augen_declarer >= state.beta {
                return Some(state.beta);
            }
        }
    }

    return None;
}

#[inline(always)]
fn transposition_table_lookup(
    tt: &TtTable,
    state: &State,
    alpha: &mut u8,
    beta: &mut u8
) -> Option<(u32, u8)>
{

    if TtTable::is_tt_compatible_state(state) {
        if let Some(tt_entry) = tt.read(state) {
            let value = tt_entry.value + state.augen_declarer;
            let bestcard = tt_entry.bestcard;
            match tt_entry.flag {
                TtFlag::EXACT => {
                    Counters::inc_exactreads();
                    return Some((bestcard,value));
                },
                TtFlag::LOWER => {
                    *alpha = cmp::max(*alpha, value);
                },
                TtFlag::UPPER => {
                    *beta = cmp::min(*beta, value);
                }
            }
            if *alpha >= *beta {
                return Some((bestcard,value));
            }
        }
    }

    None
}

#[inline(always)]
fn transposition_table_write(
    problem: &mut Problem,
    state: &State,
    alphaorig: u8,
    betaorig: u8,
    value: (u32, u8)
) {
    if TtTable::is_tt_compatible_state(state) {
        Counters::inc_writes();
        problem.transposition_table.write(
            &state,
            state.mapped_hash,
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
