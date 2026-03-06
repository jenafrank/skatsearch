#![allow(clippy::unusual_byte_groupings)]

extern crate core;

pub mod consts;
pub mod extensions;
pub mod lib_wasm;
pub mod pimc;
pub mod skat;
pub mod traits;

#[cfg(test)]
mod verification_tests {
    use crate::extensions::solver::{solve_optimum_from_position, OptimumMode};
    use crate::skat::context::GameContext;
    use crate::skat::defs::*;
    use crate::skat::engine::SkatEngine;
    use crate::traits::Bitboard;

    #[test]
    fn verify_constants() {
        println!("TRUMP_SUIT: {:032b}", TRUMP_SUIT);
        println!("SPADES:     {:032b}", SPADES);

        // Check overlap
        assert_eq!(TRUMP_SUIT & SPADES, 0, "Trump and Spades overlap!");

        // Check ST bit (19)
        let st = TENOFSPADES;
        assert!((st & SPADES) > 0, "ST is not in SPADES!");
        assert_eq!(st & TRUMP_SUIT, 0, "ST is in TRUMP_SUIT!");
    }

    #[test]
    fn verify_solver_logic() {
        // Reproduce Game 2 Trick 1 Situation
        let decl_cards = JACKOFSPADES
            | JACKOFHEARTS
            | TENOFSPADES
            | EIGHTOFSPADES
            | SEVENOFSPADES
            | QUEENOFHEARTS
            | NINEOFHEARTS
            | EIGHTOFHEARTS
            | NINEOFDIAMONDS
            | SEVENOFDIAMONDS;
        let left_cards = JACKOFCLUBS
            | JACKOFDIAMONDS
            | ACEOFCLUBS
            | KINGOFCLUBS
            | ACEOFSPADES
            | QUEENOFSPADES
            | SEVENOFHEARTS
            | TENOFDIAMONDS
            | QUEENOFDIAMONDS
            | EIGHTOFDIAMONDS;
        let right_cards = TENOFCLUBS
            | QUEENOFCLUBS
            | NINEOFCLUBS
            | EIGHTOFCLUBS
            | SEVENOFCLUBS
            | NINEOFSPADES
            | ACEOFHEARTS
            | TENOFHEARTS
            | KINGOFHEARTS
            | KINGOFDIAMONDS;

        let ctx = GameContext::create(
            decl_cards,
            left_cards,
            right_cards,
            Game::Suit,
            Player::Declarer,
        );
        let mut engine = SkatEngine::new(ctx, None);
        let mut pos = engine.create_initial_position();

        // D plays ST
        pos = pos.make_move(TENOFSPADES, &ctx);

        // Check legal moves for L
        let legal = pos.get_legal_moves();
        assert!((legal & ACEOFSPADES) > 0);
        assert!((legal & QUEENOFSPADES) > 0);
        assert_eq!(legal & (!SPADES), 0); // MUST only have Spades

        let (best, _, _) =
            solve_optimum_from_position(&mut engine, &pos, OptimumMode::BestValue).unwrap();
        assert!(
            (legal & best) > 0,
            "Solver suggested illegal move {:032b} which is not in {:032b}",
            best,
            legal
        );
    }
}
