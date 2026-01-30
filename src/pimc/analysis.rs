use crate::consts::bitboard::ALLCARDS;
use crate::pimc::pimc_problem_builder::PimcProblemBuilder;
use crate::pimc::pimc_search::PimcSearch;
use crate::skat::defs::Player;
use crate::skat::signature::HandSignature;
use crate::traits::StringConverter;
use rayon::prelude::*;

pub fn analyze_hand(my_hand: u32, samples: u32) -> (HandSignature, f32) {
    let sig = HandSignature::from_hand(my_hand);

    // Build PIMC problem
    // Grand Hand, Player is Declarer, 61 points to win.

    let remaining = ALLCARDS ^ my_hand;

    let builder = PimcProblemBuilder::new_grand()
        .my_player(Player::Declarer)
        .turn(Player::Declarer)
        .my_cards_val(my_hand)
        .remaining_cards(&remaining.__str())
        .threshold(61);

    let problem = builder.build();
    let search = PimcSearch::new(problem, samples, None);

    // estimate_win returns (win_prob, wins_count)
    let (prob, _) = search.estimate_win(false);

    (sig, prob)
}

pub fn analyze_hand_with_pickup(
    my_hand: u32,
    skat: u32,
    samples: u32,
    post_discard: bool,
) -> (HandSignature, f32, u32) {
    let sig_initial = HandSignature::from_hand(my_hand);
    let cards_12 = my_hand | skat;

    // Extract bits to vector for easier iteration
    let mut bits = Vec::new();
    for i in 0..32 {
        if (cards_12 & (1 << i)) != 0 {
            bits.push(1 << i);
        }
    }

    let mut discards = Vec::new();
    for i in 0..bits.len() {
        for j in (i + 1)..bits.len() {
            let discard = bits[i] | bits[j];
            let keep = cards_12 ^ discard;
            discards.push((keep, discard));
        }
    }

    let selection_samples = if samples > 20 { 20 } else { samples };

    // Find best discard using parallel iterator
    let best_option = discards
        .par_iter()
        .map(|(keep, discard)| {
            let remaining = ALLCARDS ^ cards_12;
            let builder = PimcProblemBuilder::new_grand()
                .my_player(Player::Declarer)
                .turn(Player::Declarer)
                .my_cards_val(*keep)
                .skat_cards(&discard.__str())
                .remaining_cards(&remaining.__str())
                .threshold(61);

            let problem = builder.build();
            let search = PimcSearch::new(problem, selection_samples, None);
            let (prob, _) = search.estimate_win(false);
            (prob, *keep, *discard)
        })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();

    let (best_prob, best_keep, best_discard) = best_option;

    let result_sig = if post_discard {
        HandSignature::from_hand_and_skat(best_keep, best_discard)
    } else {
        sig_initial
    };

    // Run full analysis on best discard (if selection samples was smaller)
    if samples > selection_samples {
        let remaining = ALLCARDS ^ cards_12;
        let builder = PimcProblemBuilder::new_grand()
            .my_player(Player::Declarer)
            .turn(Player::Declarer)
            .my_cards_val(best_keep)
            .skat_cards(&best_discard.__str())
            .remaining_cards(&remaining.__str())
            .threshold(61);

        let problem = builder.build();
        let search = PimcSearch::new(problem, samples, None);
        let (prob, _) = search.estimate_win(false);
        return (result_sig, prob, best_discard);
    }

    (result_sig, best_prob, best_discard)
}

use crate::skat::context::{GameContext, ProblemTransformation};

pub fn analyze_suit(my_hand: u32, suit: u8, samples: u32) -> (HandSignature, f32) {
    let sig = HandSignature::from_hand_and_skat_suit(my_hand, 0, Some(suit));

    let trans = match suit {
        0 => None,
        1 => Some(ProblemTransformation::SpadesSwitch),
        2 => Some(ProblemTransformation::HeartsSwitch),
        3 => Some(ProblemTransformation::DiamondsSwitch),
        _ => None,
    };

    let my_cards_val = if let Some(t) = trans {
        GameContext::get_switched_cards(my_hand, t)
    } else {
        my_hand
    };

    let remaining = ALLCARDS ^ my_cards_val;

    let builder = PimcProblemBuilder::new_farbspiel()
        .my_player(Player::Declarer)
        .turn(Player::Declarer)
        .my_cards_val(my_cards_val)
        .remaining_cards(&remaining.__str())
        .threshold(61);

    let problem = builder.build();
    let search = PimcSearch::new(problem, samples, None);
    let (prob, _) = search.estimate_win(false);

    (sig, prob)
}

pub fn analyze_suit_with_pickup(
    my_hand: u32,
    skat: u32,
    suit: u8,
    samples: u32,
    _post_discard: bool, // Argument ignored, we always return both now
) -> (HandSignature, HandSignature, f32, u32, u32) {
    let sig_initial = HandSignature::from_hand_and_skat_suit(my_hand, 0, Some(suit));
    let cards_12 = my_hand | skat;

    // Transpose EVERYTHING to Clubs world for optimization
    let trans = match suit {
        0 => None,
        1 => Some(ProblemTransformation::SpadesSwitch),
        2 => Some(ProblemTransformation::HeartsSwitch),
        3 => Some(ProblemTransformation::DiamondsSwitch),
        _ => None,
    };

    let cards_12_opt = if let Some(t) = trans {
        GameContext::get_switched_cards(cards_12, t)
    } else {
        cards_12
    };

    let mut bits = Vec::new();
    for i in 0..32 {
        if (cards_12_opt & (1 << i)) != 0 {
            bits.push(1 << i);
        }
    }

    let mut discards = Vec::new();
    for i in 0..bits.len() {
        for j in (i + 1)..bits.len() {
            let discard = bits[i] | bits[j];
            let keep = cards_12_opt ^ discard;
            discards.push((keep, discard));
        }
    }

    let selection_samples = if samples > 20 { 20 } else { samples };

    let best_option = discards
        .par_iter()
        .map(|(keep, discard)| {
            let remaining = ALLCARDS ^ cards_12_opt;
            let builder = PimcProblemBuilder::new_farbspiel()
                .my_player(Player::Declarer)
                .turn(Player::Declarer)
                .my_cards_val(*keep)
                .skat_cards(&discard.__str())
                .remaining_cards(&remaining.__str())
                .threshold(61);

            let problem = builder.build();
            let search = PimcSearch::new(problem, selection_samples, None);
            let (prob, _) = search.estimate_win(false);
            (prob, *keep, *discard)
        })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();

    let (best_prob, best_keep_opt, best_discard_opt) = best_option;

    let (best_keep, best_discard) = if let Some(t) = trans {
        (
            GameContext::get_switched_cards(best_keep_opt, t),
            GameContext::get_switched_cards(best_discard_opt, t),
        )
    } else {
        (best_keep_opt, best_discard_opt)
    };

    let sig_post = HandSignature::from_hand_and_skat_suit(best_keep, best_discard, Some(suit));

    // Run full analysis on best discard if needed
    if samples > selection_samples {
        let remaining = ALLCARDS ^ cards_12_opt;
        let builder = PimcProblemBuilder::new_farbspiel()
            .my_player(Player::Declarer)
            .turn(Player::Declarer)
            .my_cards_val(best_keep_opt)
            .skat_cards(&best_discard_opt.__str())
            .remaining_cards(&remaining.__str())
            .threshold(61);

        let problem = builder.build();
        let search = PimcSearch::new(problem, samples, None);
        let (prob, _) = search.estimate_win(false);
        return (sig_initial, sig_post, prob, best_keep, best_discard);
    }

    (sig_initial, sig_post, best_prob, best_keep, best_discard)
}

pub fn analyze_general_pre_discard<F>(count: u32, samples: u32, on_result: F)
where
    F: Fn((u32, u32, u32, HandSignature, [f32; 5], f32, u8, u128)) + Sync + Send,
{
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use std::time::Instant;

    (0..count).into_iter().for_each(|_| {
        let start_time = Instant::now();
        let mut rng = thread_rng();
        let mut deck: Vec<u32> = (0..32).map(|i| 1 << i).collect();
        deck.shuffle(&mut rng);

        let mut my_hand = 0;
        for i in 0..10 {
            my_hand |= deck[i];
        }
        let mut skat = 0;
        for i in 10..12 {
            skat |= deck[i];
        }

        // Improve samples for best-game search slightly?
        // "Best Game" involves checking 5 variants (Grand + 4 Suits) * Discards.
        // Using analyze_hand_with_pickup logic for each.

        // 1. Analyze Grand
        // analyze_hand_with_pickup now returns (sig, prob, discard)
        let (_, prob_grand, discard_grand) =
            analyze_hand_with_pickup(my_hand, skat, samples, false);

        // 2. Analyze Suits
        let mut suit_probs = [0.0; 4];
        let mut suit_discards = [0u32; 4];
        for suit in 0..4 {
            // analyze_suit_with_pickup returns (sig_init, sig_post, prob, keep, discard)
            let (_, _, prob, _, discard) =
                analyze_suit_with_pickup(my_hand, skat, suit, samples, false);
            suit_probs[suit as usize] = prob;
            suit_discards[suit as usize] = discard;
        }

        let all_probs = [
            prob_grand,
            suit_probs[0], // Clubs
            suit_probs[1], // Spades
            suit_probs[2], // Hearts
            suit_probs[3], // Diamonds
        ];

        // 3. Analyze Null
        let (_, _, prob_null, _, discard_null) =
            analyze_null_with_pickup(my_hand, skat, samples, false);

        // Determine Best Variant (Prob > Value > Preference)
        use crate::skat::rules::calculate_game_value;

        struct Candidate {
            variant: u8,
            prob: f32,
            discard: u32,
        }

        let mut candidates = Vec::with_capacity(6);
        // 0: Grand
        candidates.push(Candidate {
            variant: 0,
            prob: prob_grand,
            discard: discard_grand,
        });
        // 1..4: Suits (Clubs=1, Spades=2, Hearts=3, Diamonds=4)
        for i in 0..4 {
            candidates.push(Candidate {
                variant: (i + 1) as u8,
                prob: suit_probs[i],
                discard: suit_discards[i],
            });
        }
        // 5: Null
        candidates.push(Candidate {
            variant: 5,
            prob: prob_null,
            discard: discard_null,
        });

        candidates.sort_by(|a, b| {
            // 1. Probability (desc)
            b.prob
                .partial_cmp(&a.prob)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    // 2. Game Value (desc)
                    let val_a = calculate_game_value(my_hand | skat, a.variant);
                    let val_b = calculate_game_value(my_hand | skat, b.variant);
                    val_b.cmp(&val_a)
                })
                .then_with(|| {
                    // 3. Preference (Grand < Suits < Null by variant index? No.)
                    // User prefers Grand (0).
                    // Variant 0 < 1..4 < 5.
                    // Lower variant index = higher preference?
                    // Grand(0) > others.
                    // Null(5) vs Suit(1..4)? User said "Null before simple Diamonds".
                    // If Values Equal: Null (23) vs Diamond (9)? No, values differ.
                    // If Values Equal? E.g. Diamond With 2 (27) vs Null (23)? Null loses on Value.
                    // If Values identical? (Unlikely).
                    // Just fallback to Variant ASC (Grand first).
                    a.variant.cmp(&b.variant)
                })
        });

        let best = &candidates[0];
        let optimal_discard = best.discard;
        let best_variant = best.variant;

        // Signature of the PRE-DISCARD hand
        let sig = HandSignature::from_hand_and_skat_suit(my_hand, 0, None);

        let duration = start_time.elapsed().as_micros();
        on_result((
            my_hand,
            skat,
            optimal_discard,
            sig,
            all_probs,
            prob_null,
            best_variant,
            duration,
        ));
    })
}

pub fn analyze_null_with_pickup(
    my_hand: u32,
    skat: u32,
    samples: u32,
    post_discard: bool,
) -> (HandSignature, HandSignature, f32, u32, u32) {
    use crate::consts::bitboard::ALLCARDS;
    use crate::pimc::pimc_problem_builder::PimcProblemBuilder;
    use crate::pimc::pimc_search::PimcSearch;
    use crate::skat::defs::Player;
    use crate::traits::StringConverter;

    let sig_initial = HandSignature::from_hand(my_hand);
    let cards_12 = my_hand | skat;

    // Extract bits to vector for easier iteration
    let mut bits = Vec::new();
    for i in 0..32 {
        if (cards_12 & (1 << i)) != 0 {
            bits.push(1 << i);
        }
    }

    let mut discards = Vec::new();
    for i in 0..bits.len() {
        for j in (i + 1)..bits.len() {
            let discard = bits[i] | bits[j];
            let keep = cards_12 ^ discard;
            discards.push((keep, discard));
        }
    }

    let selection_samples = if samples > 20 { 20 } else { samples };

    // Find best discard using parallel iterator
    let best_option = discards
        .par_iter()
        .map(|(keep, discard)| {
            let remaining = ALLCARDS ^ cards_12;
            let builder = PimcProblemBuilder::new_null()
                .my_player(Player::Declarer)
                .turn(Player::Declarer)
                .my_cards_val(*keep)
                .skat_cards(&discard.__str())
                .remaining_cards(&remaining.__str());
            // Threshold is set by new_null()

            let problem = builder.build();
            let search = PimcSearch::new(problem, selection_samples, None);
            let (prob, _) = search.estimate_win(false);
            (prob, *keep, *discard)
        })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();

    let (best_prob, best_keep, best_discard) = best_option;

    let sig_post = if post_discard {
        HandSignature::from_hand_and_skat(best_keep, best_discard)
    } else {
        HandSignature::from_hand(best_keep)
    };

    // Run full analysis on best discard (if selection samples was smaller)
    let final_prob = if samples > selection_samples {
        let remaining = ALLCARDS ^ cards_12;
        let builder = PimcProblemBuilder::new_null()
            .my_player(Player::Declarer)
            .turn(Player::Declarer)
            .my_cards_val(best_keep)
            .skat_cards(&best_discard.__str())
            .remaining_cards(&remaining.__str());

        let problem = builder.build();
        let search = PimcSearch::new(problem, samples, None);
        let (prob, _) = search.estimate_win(false);
        prob
    } else {
        best_prob
    };

    (sig_initial, sig_post, final_prob, best_keep, best_discard)
}

pub fn analyze_general_hand<F>(count: u32, samples: u32, on_result: F)
where
    F: Fn((u32, u32, u32, HandSignature, [f32; 5], f32, u8, u128)) + Sync + Send,
{
    use crate::skat::context::GameContext;
    use crate::skat::defs::Game; // Fix import
    use crate::skat::defs::{
        Player, CLUBS, DIAMONDS, HEARTS, SPADES, TRUMP_GRAND, TRUMP_NULL, TRUMP_SUIT,
    }; // Ensure Player available
    use crate::skat::rules::calculate_game_value; // We will use this but boost it
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use std::time::Instant;

    // Parallel iterator for the main count loop
    (0..count).into_par_iter().for_each(|_| {
        let start_time = Instant::now();
        let mut rng = thread_rng();
        let mut deck: Vec<u32> = (0..32).map(|i| 1 << i).collect();
        deck.shuffle(&mut rng);

        let mut my_hand = 0;
        for i in 0..10 {
            my_hand |= deck[i];
        }

        let mut remaining_vec: Vec<u32> = deck[10..32].to_vec();

        let mut wins = [0u32; 6];

        for _ in 0..samples {
            remaining_vec.shuffle(&mut rng);
            let skat_vec = &remaining_vec[0..2];
            let left_vec = &remaining_vec[2..12];
            let right_vec = &remaining_vec[12..22];

            let mut skat = 0;
            for &c in skat_vec {
                skat |= c;
            }
            let mut left = 0;
            for &c in left_vec {
                left |= c;
            }
            let mut right = 0;
            for &c in right_vec {
                right |= c;
            }

            let game_configs = [
                (0, Game::Grand, None),
                (1, Game::Suit, None), // Clubs
                (2, Game::Suit, Some(ProblemTransformation::SpadesSwitch)),
                (3, Game::Suit, Some(ProblemTransformation::HeartsSwitch)),
                (4, Game::Suit, Some(ProblemTransformation::DiamondsSwitch)),
                (5, Game::Null, None),
            ];

            for (idx, game_type, transform) in game_configs.iter() {
                let my_c = if let Some(t) = transform {
                    GameContext::get_switched_cards(my_hand, *t)
                } else {
                    my_hand
                };
                let left_c = if let Some(t) = transform {
                    GameContext::get_switched_cards(left, *t)
                } else {
                    left
                };
                let right_c = if let Some(t) = transform {
                    GameContext::get_switched_cards(right, *t)
                } else {
                    right
                };

                let context =
                    GameContext::create(my_c, left_c, right_c, *game_type, Player::Declarer);
                // context.set_hand(true); REMOVED

                use crate::extensions::solver::solve_and_add_skat;
                use crate::skat::engine::SkatEngine;

                let mut engine = SkatEngine::new(context, None);
                let result = solve_and_add_skat(&mut engine);

                if *game_type == Game::Null {
                    if result.best_value == 0 {
                        wins[*idx] += 1;
                    }
                } else {
                    if result.best_value >= 61 {
                        wins[*idx] += 1;
                    }
                }
            }
        }

        let mut probs = [0.0; 6];
        for i in 0..6 {
            probs[i] = wins[i] as f32 / samples as f32; // index 5 is Null
        }

        struct Candidate {
            variant: u8,
            prob: f32,
        }

        let mut candidates = Vec::new();
        for i in 0..6 {
            candidates.push(Candidate {
                variant: i as u8,
                prob: probs[i],
            });
        }

        candidates.sort_by(|a, b| {
            b.prob
                .partial_cmp(&a.prob)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    // PRIORITIZE GRAND (0) if tied on probability
                    a.variant.cmp(&b.variant)
                })
                .then_with(|| {
                    let hand_val = if a.variant == 5 {
                        35
                    } else {
                        let base = match a.variant {
                            0 => 24,
                            1 => 12,
                            2 => 11,
                            3 => 10,
                            4 => 9,
                            _ => 0,
                        };
                        calculate_game_value(my_hand, a.variant) + base
                    };

                    let hand_val_b = if b.variant == 5 {
                        35
                    } else {
                        let base = match b.variant {
                            0 => 24,
                            1 => 12,
                            2 => 11,
                            3 => 10,
                            4 => 9,
                            _ => 0,
                        };
                        calculate_game_value(my_hand, b.variant) + base
                    };

                    hand_val_b.cmp(&hand_val)
                })
        });

        let best = &candidates[0];
        let best_variant = best.variant;

        let sig = HandSignature::from_hand_and_skat_suit(my_hand, 0, None);
        let duration = start_time.elapsed().as_micros();

        let result_probs = [probs[0], probs[1], probs[2], probs[3], probs[4]];
        let prob_null = probs[5];

        on_result((
            my_hand,
            0,
            0,
            sig,
            result_probs,
            prob_null,
            best_variant,
            duration,
        ));
    });
}
