use crate::{
    traits::{Augen, Bitboard},
    types::{
        counter::Counters,
        game::Game,
        player::Player,
        problem::Problem,
        solver::{
            retargs::{SolveWithSkatRet, SolveWithSkatRetLine},
            Solver,
        },
        state::State,
    },
};

use super::acceleration_mode::AccelerationMode;

impl Solver {
    pub fn solve_with_skat(
        left_cards: u32,
        right_cards: u32,
        declarer_cards: u32,
        game: Game,
        start_player: Player,
        accelerating_mode: AccelerationMode,
    ) -> SolveWithSkatRet {
        let mut ret: SolveWithSkatRet = SolveWithSkatRet {
            best_skat: None,
            all_skats: Vec::new(),
            counters: Counters::new(),
        };

        let p = Problem::create(declarer_cards, left_cards, right_cards, game, start_player);

        let mut solver = Solver::new(p, None);

        let initial_state = State::create_initial_state_from_problem(&solver.problem);
        let skatcards_bitmask = !initial_state.get_all_unplayed_cards();
        let twelve_cards_bitmask = skatcards_bitmask | initial_state.declarer_cards;
        let twelve_cards = twelve_cards_bitmask.__decompose_twelve();

        let mut alpha = 0;
        if game == Game::Null {
            alpha = 1;
        }

        let skat_combinations = Solver::generate_skat_combinations(&twelve_cards);

        for (skat_card_1, skat_card_2) in skat_combinations {
            let skat_bitmask = skat_card_1 | skat_card_2;
            let skat_value = skat_bitmask.__get_value();

            let player_hand_bitmask = twelve_cards_bitmask ^ skat_bitmask;

            solver.problem.set_declarer_cards(player_hand_bitmask);

            let game_value = solver.evaluate_skat_combination(
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

            Solver::update_best_skat(
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
        &mut self,
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
                        return 0; // Early return, Wert spielt hier keine Rolle
                    }
                    lower = 60 - skat_value;
                    upper = lower + 1;
                }
                AccelerationMode::NotAccelerating => {
                    // Hier erfolgt keine Änderung
                }
            }
        } else {
            lower = 0;
            upper = 1;
        }

        let result = match game {
            Game::Null => self.solve_double_dummy(0, 1, 1),
            _ => self.solve_double_dummy(lower, upper, 1),
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
}
