//! # Solver Extensions
//!
//! High-level solving functions built on top of the SkatEngine.

use crate::skat::counters::Counters;
use crate::skat::defs::Game;
use crate::skat::engine::SkatEngine;
use crate::skat::position::Position;
use crate::traits::Points;

// Return types (formerly retargs)
pub struct SolveWinRet {
    pub best_card: u32,
    pub declarer_wins: bool,
    pub counters: Counters,
}

pub struct SolveRet {
    pub best_card: u32,
    pub best_value: u8,
    pub counters: Counters,
}

pub struct SolveAllCardsRet {
    pub results: Vec<(u32, u32, u8)>, // card, follow_up, value
}

// -----------------------------------------------------------------------------
// SOLVER FUNCTIONS
// -----------------------------------------------------------------------------

pub fn solve_win(engine: &mut SkatEngine) -> SolveWinRet {
    let mut cnt = Counters::new();

    let mut alpha = engine.context.points_to_win() - 1;
    let mut beta = engine.context.points_to_win();

    if engine.context.game_type() == Game::Null {
        alpha = 0;
        beta = 1;
    }

    // create_initial_position uses the context
    let position = engine.create_initial_position();

    // Check if we need to set explicit alpha/beta for search or use default?
    // The original code passed alpha/beta to search.
    let (best_card, value) = engine.search(&position, &mut cnt, alpha, beta);

    let mut declarer_wins = value > alpha;

    if engine.context.game_type() == Game::Null {
        declarer_wins = !declarer_wins;
    }

    SolveWinRet {
        best_card,
        declarer_wins,
        counters: cnt,
    }
}

pub fn solve_double_dummy(engine: &mut SkatEngine, alpha: u8, beta: u8, width: u8) -> SolveRet {
    let mut cnt = Counters::new();
    let mut result = (0u32, 0u8);

    let mut current_alpha = alpha;
    while current_alpha < beta {
        let current_beta = std::cmp::min(current_alpha + width, beta);

        let position = engine.create_initial_position();
        result = engine.search(&position, &mut cnt, current_alpha, current_beta);

        if result.1 < current_beta {
            break;
        }

        current_alpha = current_beta;
    }

    SolveRet {
        best_card: result.0,
        best_value: result.1,
        counters: cnt,
    }
}

pub fn solve(engine: &mut SkatEngine) -> SolveRet {
    solve_double_dummy(engine, 0, 120, 1)
}

pub fn solve_all_cards(engine: &mut SkatEngine, alpha: u8, beta: u8) -> SolveAllCardsRet {
    let position = engine.create_initial_position();
    solve_all_cards_from_position(engine, &position, alpha, beta)
}

pub fn solve_all_cards_from_position(
    engine: &mut SkatEngine,
    position: &Position,
    alpha: u8,
    beta: u8,
) -> SolveAllCardsRet {
    let mut results = Vec::new();

    let moves_word = position.get_legal_moves();
    // Assuming we want to sort them or just iterate
    let (moves, n) = crate::skat::rules::get_sorted_by_value(moves_word);

    let mut cnt = Counters::new();

    for mov in &moves[0..n] {
        let child_pos = position.make_move(*mov, &engine.context);
        let (best_response, value) = engine.search(&child_pos, &mut cnt, alpha, beta);

        results.push((*mov, best_response, value));
    }

    SolveAllCardsRet { results }
}

pub fn solve_and_add_skat(engine: &mut SkatEngine) -> SolveRet {
    // Solve core problem
    let mut ret = solve_double_dummy(engine, 0, 120, 1);

    let double_dummy_result = ret.best_value;
    let skat_value = engine.context.get_skat().points();

    match engine.context.game_type() {
        Game::Null => {
            if double_dummy_result == 0 {
                ret.best_value = 0;
            } else {
                ret.best_value = 1;
            }
        }
        _ => {
            ret.best_value = double_dummy_result + skat_value;
        }
    }

    ret
}

// -----------------------------------------------------------------------------
// OPTIMUM SOLVER
// -----------------------------------------------------------------------------

use crate::skat::defs::Player;
use crate::skat::search::search_optimum;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimumMode {
    BestValue,
    AllWinning,
}

pub fn solve_optimum(engine: &mut SkatEngine, mode: OptimumMode) -> Result<u32, &'static str> {
    let position = engine.create_initial_position();
    solve_optimum_from_position(engine, &position, mode)
}

pub fn solve_optimum_from_position(
    engine: &mut SkatEngine,
    position: &Position,
    mode: OptimumMode,
) -> Result<u32, &'static str> {
    // Phase 1: Get all outcomes (Exact values)
    let phase1_results = solve_all_cards_from_position(engine, position, 0, 120);

    if phase1_results.results.is_empty() {
        return Err("No legal moves found");
    }

    // Filter Logic
    let candidates: Vec<u32> = match mode {
        OptimumMode::BestValue => {
            // Find best value based on player perspective?
            // solve_all_cards returns 'Value' (Declarer Points).
            // If Declarer: Max Value is best.
            // If Defender: Min Value is best.
            let is_declarer = position.player == Player::Declarer;
            let is_null = engine.context.game_type() == Game::Null;

            if is_declarer {
                if is_null {
                    // Start of game null decl minimizes?
                    // solve_all_cards returns 0 (Win) or 1 (Loss).
                    // Decl wants 0 (Min).
                    let min_val = phase1_results
                        .results
                        .iter()
                        .map(|(_, _, v)| *v)
                        .min()
                        .unwrap();
                    phase1_results
                        .results
                        .iter()
                        .filter(|(_, _, v)| *v == min_val)
                        .map(|(m, _, _)| *m)
                        .collect()
                } else {
                    // Suit Decl wants Max
                    let max_val = phase1_results
                        .results
                        .iter()
                        .map(|(_, _, v)| *v)
                        .max()
                        .unwrap();
                    phase1_results
                        .results
                        .iter()
                        .filter(|(_, _, v)| *v == max_val)
                        .map(|(m, _, _)| *m)
                        .collect()
                }
            } else {
                // Defender
                if is_null {
                    // Defender wants Decl Loss (Value 1). Maximize.
                    let max_val = phase1_results
                        .results
                        .iter()
                        .map(|(_, _, v)| *v)
                        .max()
                        .unwrap();
                    phase1_results
                        .results
                        .iter()
                        .filter(|(_, _, v)| *v == max_val)
                        .map(|(m, _, _)| *m)
                        .collect()
                } else {
                    // Suit Defender wants Min Decl Points
                    let min_val = phase1_results
                        .results
                        .iter()
                        .map(|(_, _, v)| *v)
                        .min()
                        .unwrap();
                    phase1_results
                        .results
                        .iter()
                        .filter(|(_, _, v)| *v == min_val)
                        .map(|(m, _, _)| *m)
                        .collect()
                }
            }
        }
        OptimumMode::AllWinning => {
            let is_declarer = position.player == Player::Declarer;
            let is_null = engine.context.game_type() == Game::Null;

            if is_null {
                // Null Win Condition:
                // Decl Wins if Value == 0.
                // Defender Wins if Value == 1.
                let target_val = if is_declarer { 0 } else { 1 };

                let wins: Vec<u32> = phase1_results
                    .results
                    .iter()
                    .filter(|(_, _, v)| *v == target_val)
                    .map(|(m, _, _)| *m)
                    .collect();

                if wins.is_empty() {
                    // No wins found. Fallback to all (Optimization will pick best among losses)
                    phase1_results.results.iter().map(|(m, _, _)| *m).collect()
                } else {
                    wins
                }
            } else {
                // Suit Win Condition: Points > 60.
                // Decl Wins if > 60.
                // Defender Wins if <= 60.

                let wins: Vec<u32> = if is_declarer {
                    phase1_results
                        .results
                        .iter()
                        .filter(|(_, _, v)| *v > 60)
                        .map(|(m, _, _)| *m)
                        .collect()
                } else {
                    phase1_results
                        .results
                        .iter()
                        .filter(|(_, _, v)| *v <= 60)
                        .map(|(m, _, _)| *m)
                        .collect()
                };

                if wins.is_empty() {
                    phase1_results.results.iter().map(|(m, _, _)| *m).collect()
                } else {
                    wins
                }
            }
        }
    };

    // Phase 2: Optimum Search
    if candidates.is_empty() {
        return Err("No candidates after filtering");
    }

    let mut cnt = Counters::new(); // Local counters for phase 2
    let is_declarer = position.player == Player::Declarer;

    // Use i16::MIN/MAX for score tracking
    // search_optimum score is from Declarer perspective.
    // If I am Declarer: I want Max Score.
    // If I am Defender: I want Min Score (Declarer Loss/Low Score).

    let mut best_move = candidates[0];
    let mut best_score_so_far = if is_declarer { i16::MIN } else { i16::MAX };

    for mov in candidates {
        let child_pos = position.make_move(mov, &engine.context);

        let (_, score) = search_optimum(
            &engine.context,
            &child_pos,
            &mut cnt,
            i16::MIN + 1,
            i16::MAX - 1,
            1, // Depth 1
        );

        if is_declarer {
            if score > best_score_so_far {
                best_score_so_far = score;
                best_move = mov;
            }
        } else {
            if score < best_score_so_far {
                best_score_so_far = score;
                best_move = mov;
            }
        }
    }

    Ok(best_move)
}
