//! # Skat Solving Extensions
//!
//! Functions to find the best skat and game type.

use crate::skat::context::GameContext;
use crate::skat::counters::Counters;
use crate::skat::defs::{Game, Player};
use crate::skat::engine::SkatEngine;
// use crate::skat::position::Position;
use crate::extensions::solver::solve_double_dummy;
use crate::traits::{Bitboard, Points};

// -----------------------------------------------------------------------------
// TYPES
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccelerationMode {
    AlphaBetaAccelerating,
    WinningOnly,
    NotAccelerating,
}

#[derive(Clone, Copy, Debug)]
pub struct SolveWithSkatRetLine {
    pub skat_card_1: u32,
    pub skat_card_2: u32,
    pub value: u8,
}

pub struct SolveWithSkatRet {
    pub best_skat: Option<SolveWithSkatRetLine>,
    pub all_skats: Vec<SolveWithSkatRetLine>,
    pub counters: Counters,
}

// -----------------------------------------------------------------------------
// FUNCTIONS
// -----------------------------------------------------------------------------

pub fn solve_with_skat(
    left_cards: u32,
    right_cards: u32,
    declarer_cards: u32,
    game: Game,
    start_player: Player,
    accelerating_mode: AccelerationMode,
) -> SolveWithSkatRet {
    let mut ret = SolveWithSkatRet {
        best_skat: None,
        all_skats: Vec::new(),
        counters: Counters::new(),
    };

    let p = GameContext::create(declarer_cards, left_cards, right_cards, game, start_player);

    let mut engine = SkatEngine::new(p, None);

    let initial_position = engine.create_initial_position();
    // Using core rules directly or methods on Position? Position has get_all_unplayed_cards() wrapper?
    // Position methods were consolidated.
    // get_all_unplayed_cards() was in Position methods.
    let skatcards_bitmask = !crate::skat::rules::get_all_unplayed_cards(
        initial_position.declarer_cards,
        initial_position.left_cards,
        initial_position.right_cards,
    );
    // Wait, Position struct doesn't expose get_all_unplayed_cards public method?
    // I should check `position.rs` generated.

    // Assuming Position stores declarer_cards etc.
    // `initial_position.declarer_cards` is public.

    let twelve_cards_bitmask = skatcards_bitmask | initial_position.declarer_cards;
    let twelve_cards = twelve_cards_bitmask.__decompose_twelve();

    let mut alpha = 0;
    if game == Game::Null {
        alpha = 1;
    }

    let skat_combinations = generate_skat_combinations(&twelve_cards);

    for (skat_card_1, skat_card_2) in skat_combinations {
        let skat_bitmask = skat_card_1 | skat_card_2;
        let skat_value = skat_bitmask.points();

        let player_hand_bitmask = twelve_cards_bitmask ^ skat_bitmask;

        engine.context.set_declarer_cards(player_hand_bitmask);

        let game_value = evaluate_skat_combination(
            &mut engine,
            skat_value,
            accelerating_mode,
            alpha,
            game,
            &mut ret.counters,
        );

        ret.all_skats.push(SolveWithSkatRetLine {
            skat_card_1,
            skat_card_2,
            value: game_value,
        });

        update_best_skat(
            &mut ret,
            skat_card_1,
            skat_card_2,
            game_value,
            game,
            &mut alpha,
        );

        if (accelerating_mode == AccelerationMode::AlphaBetaAccelerating
            || accelerating_mode == AccelerationMode::WinningOnly)
            && game == Game::Null
            && game_value == 0
        {
            break;
        }
    }

    ret
}

fn evaluate_skat_combination(
    engine: &mut SkatEngine,
    skat_value: u8,
    mode: AccelerationMode,
    alpha: u8,
    game: Game,
    cnt: &mut Counters,
) -> u8 {
    // Default window
    let mut lower = 0;
    let mut upper = 120;

    if game != Game::Null {
        match mode {
            AccelerationMode::AlphaBetaAccelerating => {
                if alpha > skat_value {
                    lower = alpha - skat_value;
                }
            }
            AccelerationMode::WinningOnly => {
                if alpha >= 61 {
                    return 0; // Early return
                }
                lower = 60 - skat_value;
                upper = lower + 1;
            }
            AccelerationMode::NotAccelerating => {}
        }
    } else {
        lower = 0;
        upper = 1;
    }

    let result = match game {
        Game::Null => solve_double_dummy(engine, 0, 1, 1),
        _ => solve_double_dummy(engine, lower, upper, 1),
    };

    cnt.add(result.counters);

    match game {
        Game::Null => result.best_value,
        _ => result.best_value + skat_value,
    }
}

fn update_best_skat(
    ret: &mut SolveWithSkatRet,
    skat_card_1: u32,
    skat_card_2: u32,
    game_value: u8,
    game: Game,
    alpha: &mut u8,
) {
    match game {
        Game::Null => {
            if game_value < *alpha || ret.best_skat.is_none() {
                ret.best_skat = Some(SolveWithSkatRetLine {
                    skat_card_1,
                    skat_card_2,
                    value: game_value,
                });
                *alpha = game_value;
            }
        }
        _ => {
            if game_value > *alpha || ret.best_skat.is_none() {
                ret.best_skat = Some(SolveWithSkatRetLine {
                    skat_card_1,
                    skat_card_2,
                    value: game_value,
                });
                *alpha = game_value;
            }
        }
    }
}

pub fn generate_skat_combinations(cards: &[u32]) -> Vec<(u32, u32)> {
    let mut combinations = Vec::new();
    for i in 0..11 {
        for j in i + 1..12 {
            combinations.push((cards[i], cards[j]));
        }
    }
    combinations
}
