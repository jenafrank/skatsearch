//! # Search
//!
//! Implementation of the Alpha-Beta pruning algorithm.

use crate::skat::context::GameContext;
use crate::skat::counters::Counters; // Moving this later
use crate::skat::defs::{Game, Player};
use crate::skat::position::Position;
use crate::skat::rules::get_sorted_by_value;
use crate::skat::tt::{TranspositionFlag, TranspositionTable};

use std::cmp;

enum GameStrategy {
    Standard,
    Null,
}

impl GameStrategy {
    fn evaluate(&self, a: u8, b: u8, player: Player) -> bool {
        match self {
            GameStrategy::Standard => {
                (player == Player::Declarer && a > b) || (player != Player::Declarer && a < b)
            }
            GameStrategy::Null => {
                (player == Player::Declarer && a < b) || (player != Player::Declarer && a > b)
            }
        }
    }

    fn initial_value(&self, player: Player) -> u8 {
        match self {
            GameStrategy::Standard => {
                if player == Player::Declarer {
                    0
                } else {
                    120
                }
            }
            GameStrategy::Null => {
                if player == Player::Declarer {
                    1
                } else {
                    0
                }
            }
        }
    }
}

pub fn search(
    game_context: &GameContext,
    position: &Position,
    tt: &mut TranspositionTable,
    cnt: &mut Counters,
    mut alpha: u8,
    mut beta: u8,
) -> (u32, u8) {
    cnt.inc_iters();

    let strategy = match game_context.game_type {
        Game::Null => GameStrategy::Null,
        _ => GameStrategy::Standard,
    };

    // TERMINATION
    if let Some(x) = apply_termination_criteria(game_context, position, alpha, beta) {
        return (0, x);
    }

    let mut optimized_value: (u32, u8) = (0, strategy.initial_value(position.player));
    let mut tt_best_card = 0;

    // TRANSPOSITION TABLE LOOKUP
    if let Some(x) =
        transposition_table_lookup(tt, position, &mut alpha, &mut beta, cnt, &mut tt_best_card)
    {
        return x;
    }

    // Freeze alpha and beta for tt entry later on
    let alphaorig = alpha;
    let betaorig = beta;

    // MOVE GENERATION & SORTING
    // Using Position::get_reduced_moves (was get_reduced)
    let moves_word = position.get_reduced_moves(game_context);
    let (mut moves, n) = get_sorted_by_value(moves_word);

    // Put best card from TT to front (PV-Move)
    if tt_best_card > 0 {
        for i in 0..n {
            if moves[i] == tt_best_card {
                moves.swap(0, i);
                break;
            }
        }
    }

    // Set dummy card if no optimization of start value possible
    optimized_value.0 = moves[0];

    // BRANCHING LOOP
    for mov in &moves[0..n] {
        let child_position = position.make_move(*mov, game_context);

        let child_value = search(game_context, &child_position, tt, cnt, alpha, beta);

        // Optimize value
        if strategy.evaluate(child_value.1, optimized_value.1, position.player) {
            optimized_value.0 = *mov;
            optimized_value.1 = child_value.1;
        }

        // Alpha-beta cutoffs
        if shrink_alpha_beta_window(
            position.player,
            &mut alpha,
            &mut beta,
            child_value.1,
            game_context.game_type,
        ) {
            cnt.inc_breaks();
            break;
        }
    }

    transposition_table_write(tt, position, alphaorig, betaorig, optimized_value, cnt);

    optimized_value
}

#[inline(always)]
fn apply_termination_criteria(
    context: &GameContext,
    position: &Position,
    alpha: u8,
    beta: u8,
) -> Option<u8> {
    if position.player_cards == 0 {
        return Some(position.declarer_points);
    }

    match context.game_type {
        Game::Null => {
            if position.declarer_points > 0 {
                return Some(1);
            }
            None
        }
        _ => {
            if context.total_points() - position.team_points <= alpha {
                return Some(alpha);
            }

            if position.declarer_points >= beta {
                return Some(beta);
            }
            None
        }
    }
}

#[inline(always)]
fn transposition_table_lookup(
    tt: &TranspositionTable,
    position: &Position,
    alpha: &mut u8,
    beta: &mut u8,
    cnt: &mut Counters,
    tt_best_card: &mut u32,
) -> Option<(u32, u8)> {
    // Check compatibility (was read from TtTable::is_tt_compatible_state or similar)
    // Inline check: not root state and no trick cards played?
    if !is_tt_compatible(position) {
        return None;
    }

    if let Some(tt_entry) = tt.read(position, cnt) {
        *tt_best_card = tt_entry.bestcard;
        // Stored value is REMAINING value. Adding accumulated augen.
        let value = tt_entry.value + position.declarer_points;
        let bestcard = tt_entry.bestcard;

        match tt_entry.flag {
            TranspositionFlag::Exact => {
                cnt.inc_exactreads();
                return Some((bestcard, value));
            }
            TranspositionFlag::Lower => {
                *alpha = cmp::max(*alpha, value);
            }
            TranspositionFlag::Upper => {
                *beta = cmp::min(*beta, value);
            }
        }
        if *alpha >= *beta {
            return Some((bestcard, value));
        }
    }

    None
}

#[inline(always)]
fn transposition_table_write(
    tt: &mut TranspositionTable,
    position: &Position,
    alphaorig: u8,
    betaorig: u8,
    value: (u32, u8),
    cnt: &mut Counters,
) {
    if is_tt_compatible(position) {
        cnt.inc_writes();
        tt.write(position, position.get_hash(), alphaorig, betaorig, value);
    }
}

fn is_tt_compatible(position: &Position) -> bool {
    // TtTable::is_tt_compatible_state(state) checked: state.is_not_root_state() && state.trick_cards_count == 0
    !position.is_root_position && position.trick_cards_count == 0
}

#[inline(always)]
fn shrink_alpha_beta_window(
    player: Player,
    alpha: &mut u8,
    beta: &mut u8,
    child_state_value: u8,
    game: Game,
) -> bool {
    match game {
        Game::Null => match player {
            Player::Declarer => {
                *beta = cmp::min(*beta, child_state_value);
                if *beta <= *alpha {
                    return true;
                }
            }
            _ => {
                *alpha = cmp::max(*alpha, child_state_value);
                if *alpha >= *beta {
                    return true;
                }
            }
        },

        _ => match player {
            Player::Declarer => {
                *alpha = cmp::max(*alpha, child_state_value);
                if *alpha >= *beta {
                    return true;
                }
            }
            _ => {
                *beta = cmp::min(*beta, child_state_value);
                if *beta <= *alpha {
                    return true;
                }
            }
        },
    }

    false
}
