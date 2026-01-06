//! # All Games Extension
//!
//! Calculates results for all game types (Grand, Null, Suits) in parallel.

use crate::skat::context::{GameContext, ProblemTransformation};
use crate::skat::defs::{Game, Player};
use crate::skat::engine::SkatEngine;
use crate::extensions::skat_solving::{solve_with_skat, AccelerationMode, SolveWithSkatRet};
use crate::extensions::solver::{solve_and_add_skat, SolveRet};
use rayon::prelude::*;

// -----------------------------------------------------------------------------
// TYPES
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameKey {
    Eichel,
    Gruen,
    Herz,
    Schell,
    Grand,
    Null,
}

#[derive(Debug)]
pub enum CalculationError {
    NoBestSkatFound(GameKey),
    SolverResultError(String),
}

impl std::fmt::Display for CalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalculationError::NoBestSkatFound(key) => {
                write!(f, "No best skat found for: {:?}", key)
            }
            CalculationError::SolverResultError(msg) => write!(f, "Solver error: {}", msg),
        }
    }
}

pub struct AllGames {
    pub eichel_farbe: u8,
    pub gruen_farbe: u8,
    pub herz_farbe: u8,
    pub schell_farbe: u8,
    pub eichel_hand: u8,
    pub gruen_hand: u8,
    pub herz_hand: u8,
    pub schell_hand: u8,

    pub grand: u8,
    pub grand_hand: u8,

    pub null: u8,
    pub null_hand: u8,
}

impl Default for AllGames {
    fn default() -> AllGames {
        AllGames {
            eichel_farbe: 123,
            gruen_farbe: 123,
            herz_farbe: 123,
            schell_farbe: 123,
            eichel_hand: 123,
            gruen_hand: 123,
            herz_hand: 123,
            schell_hand: 123,
            grand: 123,
            grand_hand: 123,
            null: 123,
            null_hand: 123,
        }
    }
}

// -----------------------------------------------------------------------------
// FUNCTIONS
// -----------------------------------------------------------------------------

pub fn calc_all_games(
    left_cards: u32,
    right_cards: u32,
    my_cards: u32,
    start_player: Player,
) -> Result<AllGames, CalculationError> {
    let acc_mode = AccelerationMode::AlphaBetaAccelerating;

    // Create base problem
    let base_problem_farbe_eichel =
        GameContext::create(my_cards, left_cards, right_cards, Game::Farbe, start_player);

    // Create transformed problems
    let problem_farbe_gruen = GameContext::create_transformation(
        base_problem_farbe_eichel.clone(),
        ProblemTransformation::SpadesSwitch,
    );
    let problem_farbe_herz = GameContext::create_transformation(
        base_problem_farbe_eichel.clone(),
        ProblemTransformation::HeartsSwitch,
    );
    let problem_farbe_schell = GameContext::create_transformation(
        base_problem_farbe_eichel.clone(),
        ProblemTransformation::DiamondsSwitch,
    );

    let problem_grand =
        GameContext::create(my_cards, left_cards, right_cards, Game::Grand, start_player);
    let problem_null =
        GameContext::create(my_cards, left_cards, right_cards, Game::Null, start_player);

    let tasks = vec![
        (
            GameKey::Eichel,
            base_problem_farbe_eichel,
            Game::Farbe,
            acc_mode,
        ),
        (GameKey::Gruen, problem_farbe_gruen, Game::Farbe, acc_mode),
        (GameKey::Herz, problem_farbe_herz, Game::Farbe, acc_mode),
        (GameKey::Schell, problem_farbe_schell, Game::Farbe, acc_mode),
        (GameKey::Grand, problem_grand, Game::Grand, acc_mode),
        (GameKey::Null, problem_null, Game::Null, acc_mode),
    ];

    let calculate_single_pair = |(key, context, game_type, acc_mode): (
        GameKey,
        GameContext,
        Game,
        AccelerationMode,
    )|
     -> Result<
        (GameKey, SolveWithSkatRet, SolveRet),
        CalculationError,
    > {
        // Calculate with skat
        let skat_result = solve_with_skat(
            context.left_cards(),
            context.right_cards(),
            context.declarer_cards(),
            game_type,
            context.start_player(),
            acc_mode,
        );

        // Calculate hand
        let mut engine = SkatEngine::new(context, None);
        let hand_result = solve_and_add_skat(&mut engine);

        Ok((key, skat_result, hand_result))
    };

    let results: Vec<Result<(GameKey, SolveWithSkatRet, SolveRet), CalculationError>> =
        tasks.into_par_iter().map(calculate_single_pair).collect();

    let mut final_games = AllGames::default();

    for result in results {
        let (key, skat_ret, hand_ret) = result?;

        let skat_value = skat_ret
            .best_skat
            .ok_or(CalculationError::NoBestSkatFound(key))?
            .value;

        let hand_value = hand_ret.best_value;

        match key {
            GameKey::Eichel => {
                final_games.eichel_farbe = skat_value;
                final_games.eichel_hand = hand_value;
            }
            GameKey::Gruen => {
                final_games.gruen_farbe = skat_value;
                final_games.gruen_hand = hand_value;
            }
            GameKey::Herz => {
                final_games.herz_farbe = skat_value;
                final_games.herz_hand = hand_value;
            }
            GameKey::Schell => {
                final_games.schell_farbe = skat_value;
                final_games.schell_hand = hand_value;
            }
            GameKey::Grand => {
                final_games.grand = skat_value;
                final_games.grand_hand = hand_value;
            }
            GameKey::Null => {
                final_games.null = skat_value;
                final_games.null_hand = hand_value;
            }
        }
    }

    Ok(final_games)
}
