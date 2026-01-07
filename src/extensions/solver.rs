//! # Solver Extensions
//!
//! High-level solving functions built on top of the SkatEngine.

use crate::skat::counters::Counters;
use crate::skat::defs::Game;
use crate::skat::engine::SkatEngine;
// use crate::skat::position::Position;
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
    // This requires logic to iterate all cards from root state.
    // Original implementation in get.rs called self.get_all_cards(state, alpha, beta).
    // Let's implement it here.

    let position = engine.create_initial_position();
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
