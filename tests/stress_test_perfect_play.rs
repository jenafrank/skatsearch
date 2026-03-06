use rand::prelude::*;
use skat_aug23::extensions::solver::{solve_optimum_from_position, OptimumMode};
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::defs::{Game, Player};
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::skat::rules::get_legal_moves;
use skat_aug23::traits::StringConverter;

#[test]
fn stress_test_perfect_play() {
    let mut rng = thread_rng();

    for game_idx in 0..1000 {
        let mut deck: Vec<u8> = (0..32).collect();
        deck.shuffle(&mut rng);

        let mut d_cards = 0u32;
        let mut l_cards = 0u32;
        let mut r_cards = 0u32;

        for i in 0..10 {
            d_cards |= 1 << deck[i];
        }
        for i in 10..20 {
            l_cards |= 1 << deck[i];
        }
        for i in 20..30 {
            r_cards |= 1 << deck[i];
        }

        let game_types = [Game::Suit, Game::Grand, Game::Null];
        let game_type = game_types[rng.gen_range(0..3)];

        let ctx = GameContext::create(d_cards, l_cards, r_cards, game_type, Player::Declarer);
        let mut engine = SkatEngine::new(ctx.clone(), None);
        let mut position = engine.create_initial_position();

        while position.get_legal_moves() != 0 {
            let (perfect_card, _, _) =
                solve_optimum_from_position(&mut engine, &position, OptimumMode::BestValue)
                    .unwrap();

            let legal_moves = get_legal_moves(position.trick_suit, position.player_cards);
            if (legal_moves & perfect_card) == 0 {
                panic!("ILLEGAL MOVE DETERMINED! Game {}, Trick {}, Player {:?}\nCard: {}\nLegal: {:032b}\nTrick Suit: {:032b}\nHand: {:032b}\nGame Type: {:?}", 
                    game_idx, (30 - position.player_cards.count_ones()) / 3 + 1, position.player,
                    perfect_card.__str(), legal_moves, position.trick_suit, position.player_cards, game_type);
            }

            position = position.make_move(perfect_card, &ctx);

            if game_type == Game::Null && position.declarer_points > 0 {
                break;
            }
        }
    }
}
