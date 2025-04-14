use std::cmp;
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::state::*;
use crate::types::tt_flag::TtFlag;
use crate::types::tt_table::TtTable;
use crate::core_functions::get_sorted_by_value::get_sorted_by_value;

enum GameStrategy {
    Standard,
    Null,
}

impl GameStrategy {    
    fn evaluate(&self, a: u8, b: u8, player: Player) -> bool {
        match self {
            GameStrategy::Standard => (player == Player::Declarer && a > b) || (player != Player::Declarer && a < b),
            GameStrategy::Null     => (player == Player::Declarer && a < b) || (player != Player::Declarer && a > b),
        }
    }

    fn initial_value(&self, player: Player) -> u8 {
        match self {
            GameStrategy::Standard => if player == Player::Declarer { 0 } else { 120 },
            GameStrategy::Null => if player == Player::Declarer { 1 } else { 0 },
        }
    }
}

impl Problem {

    pub fn search(&self, state: &State, tt: &mut TtTable, cnt: &mut Counters) -> (u32, u8) {

        cnt.inc_iters();

        let strategy = match self.game_type {
            Game::Null => GameStrategy::Null,
            _          => GameStrategy::Standard,
        };
 
        // BASIC: Termination of recursive search
        if let Some(x) = apply_termination_criteria(&self, &state) {
            return (0, x);
        }
        
        let mut alpha = state.alpha;
        let mut beta = state.beta;
        let mut optimized_value: (u32, u8) = (0, strategy.initial_value(state.player));

        // TRANS:
        if let Some(x) = transposition_table_lookup(
            &tt,
            &state,
            &mut alpha,
            &mut beta,
            cnt
        ) {
            return x;
        }

        // TRANS: Freeze alpha and beta for tt entry later on
        let alphaorig = alpha;
        let betaorig = beta;

        // BASIC: Reduce moves, sort moves, find connections
        let moves_word = state.get_reduced(&self);
        let (moves, n) = get_sorted_by_value(moves_word);

        // Set dummy card if no optimization of start value possible
        optimized_value.0 = moves[0];

        // BASIC: Branching loop
        for mov in &moves[0..n] {

            // BASIC: Generate child state
            let child_state = state.create_child_state(
                *mov,
                &self,
                alpha,
                beta);

            // BASIC: Search child state
            let child_state_value = self.search(&child_state, tt, cnt);

            // Optimize value
            if strategy.evaluate(child_state_value.1, optimized_value.1, state.player) {
                optimized_value.0 = *mov;
                optimized_value.1 = child_state_value.1;
            }

            // Alpha-beta cutoffs            
            if shrink_alpha_beta_window(state.player, &mut alpha, &mut beta, child_state_value.1, self.game_type) {
                cnt.inc_breaks();
                break;
            }            
        }

        transposition_table_write( 
            tt,           
            &state,
            alphaorig,
            betaorig,
            optimized_value,
            cnt
        );

        optimized_value
    }
}

#[inline(always)]
fn apply_termination_criteria(problem: &Problem, state: &State) -> Option<u8> {
    if state.player_cards == 0 {
        return Some(state.augen_declarer);
    }

    match problem.game_type {
        Game::Null => return apply_termination_criteria_null(state),
        _ => return apply_termination_criteria_standard(problem, state),
    };
}

#[inline(always)]
fn apply_termination_criteria_null(state: &State) -> Option<u8> {
    if state.augen_declarer > 0 {
        return Some(1);
    }
    None
}

#[inline(always)]
fn apply_termination_criteria_standard(problem: &Problem, state: &State) -> Option<u8> {
    if problem.augen_total() - state.augen_team <= state.alpha {
        return Some(state.alpha);
    }

    if state.augen_declarer >= state.beta {
        return Some(state.beta);
    }
    None
}

#[inline(always)]
fn transposition_table_lookup(
    tt: &TtTable,
    state: &State,
    alpha: &mut u8,
    beta: &mut u8,
    cnt: &mut Counters
) -> Option<(u32, u8)>
{

    if TtTable::is_tt_compatible_state(state) {
        if let Some(tt_entry) = tt.read(state, cnt) {
            let value = tt_entry.value + state.augen_declarer;
            let bestcard = tt_entry.bestcard;
            match tt_entry.flag {
                TtFlag::EXACT => {
                    cnt.inc_exactreads();
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
    tt: &mut TtTable,
    state: &State,
    alphaorig: u8,
    betaorig: u8,
    value: (u32, u8),
    cnt: &mut Counters
) {
    if TtTable::is_tt_compatible_state(state) {
        cnt.inc_writes();
        tt.write(
            &state,
            state.get_hash(),
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
