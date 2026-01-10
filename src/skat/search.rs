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
// -----------------------------------------------------------------------------
// OPTIMUM SEARCH (Fast Win / Slow Loss)
// -----------------------------------------------------------------------------

pub fn search_optimum(
    game_context: &GameContext,
    position: &Position,
    cnt: &mut Counters,
    mut alpha: i16,
    mut beta: i16,
    depth: i16,
) -> (u32, i16) {
    cnt.inc_iters();

    // TERMINATION
    // Check if game is over (all cards played)
    if position.player_cards == 0 {
        return (0, evaluate_terminal_node(game_context, position, depth));
    }

    // Null Game Early Termination: If Declarer takes a trick (points > 0), they lose immediately.
    if game_context.game_type == Game::Null && position.declarer_points > 0 {
        return (0, evaluate_terminal_node(game_context, position, depth));
    }

    // Determine current player's objective
    // Suits/Grand: Declarer wants MAX, Defenders want MIN.
    // Null: Declarer wants MIN (0), Defenders want MAX (1).
    let is_declarer = position.player == Player::Declarer;

    // For Null game, we map values: Declarer Loss = 0, Declarer Win = 1?
    // In standard search: Null Declarer Win = 1 (Max), Loss = 0 (Min)?
    // Wait, let's check `initial_value`:
    // Null: Declarer init=1 (wants MIN?), Defenders init=0 (want MAX?).
    // GameStrategy::Null verify:
    // (player == Player::Declarer && a < b) => Declarer minimizes?
    // "Declarer init=1". Yes. If child is 0 (Loss for Opponent/Win for Decl?), 0 < 1.
    // So usually Null: Declarer wants 0 (No tricks taken? No simply 'Win').
    // Let's standardise on "Score relative to Declarer".
    // Score > 0 => Declarer Win. Score < 0 => Declarer Loss.

    // Move Generation
    let moves_word = position.get_reduced_moves(game_context);
    let (moves, n) = get_sorted_by_value(moves_word);

    let mut best_move = moves[0];
    let mut best_score;

    // Let's define Score Viewpoint always from DECLARER perspective.
    // Suit/Grand: Declarer wants MAX. Opponents want MIN.
    // Null: Declarer wants WIN.
    //   If we define Win = Positive, Loss = Negative.
    //   Then Declarer always wants MAX. Opponents always want MIN.
    //   This simplifies everything.
    best_score = if is_declarer {
        i16::MIN + 1
    } else {
        i16::MAX - 1
    };

    for mov in &moves[0..n] {
        let child_position = position.make_move(*mov, game_context);
        let (_, child_score) =
            search_optimum(game_context, &child_position, cnt, alpha, beta, depth + 1);

        if is_declarer {
            if child_score > best_score {
                best_score = child_score;
                best_move = *mov;
            }
            alpha = cmp::max(alpha, best_score);
            if alpha >= beta {
                break; // Beta Cutoff
            }
        } else {
            if child_score < best_score {
                best_score = child_score;
                best_move = *mov;
            }
            beta = cmp::min(beta, best_score);
            if beta <= alpha {
                break; // Alpha Cutoff
            }
        }
    }

    (best_move, best_score)
}

fn evaluate_terminal_node(context: &GameContext, position: &Position, depth: i16) -> i16 {
    let declarer_points = position.declarer_points as i16;
    let max_depth = 40; // Max plies (3*10=30, plus safety)

    match context.game_type {
        Game::Null => {
            // Declarer Win condition: No tricks taken? Or points?
            // In Null, `declarer_points` > 0 means LOST.
            let lost = declarer_points > 0;
            if !lost {
                // WIN (0 points)
                // Fast Win: Small Depth is better?
                // No, usually in Null, "Win" is surviving until end.
                // So "Fast Win" is irrelevant? Or is "Fast Win" avoiding tricks?
                // Actually, if I am forced to take a trick, I lose.
                // If I play cards such that I DON'T take a trick, game continues.
                // So "Win" is reaching depth 30 without points.
                // Score = 1000. Depth doesn't matter for win (always end of game).
                // What about "Slow Loss"?
                // If I must take a trick, I want to delay it.
                // Loss at depth 10 > Loss at depth 5.
                // Loss Score = -1000 + depth.
                1000
            } else {
                // LOST (Taken a trick).
                // Score < 0.
                // Later loss (higher depth) is better.
                // -1000 + depth.
                -1000 + depth
            }
        }
        _ => {
            // Suit / Grand
            let win = declarer_points > 60; // 61+ to win
            if win {
                // WIN
                // Fast Win: Small depth is better.
                // Score = 1000 + (MaxDepth - depth).
                // e.g. Depth 10: 1000 + (40-10) = 1030.
                // e.g. Depth 30: 1000 + (40-30) = 1010.
                // 1030 > 1010. Fast win preferred.
                10000 + (max_depth - depth) * 10
            } else {
                // LOSS
                // Slow Loss: Large depth is better.
                // Score = -10000 + depth * 10.
                // e.g. Depth 10: -10000 + 100 = -9900.
                // e.g. Depth 30: -10000 + 300 = -9700.
                // -9700 > -9900. Slow loss preferred.
                -10000 + depth * 10
            }
        }
    }
}
