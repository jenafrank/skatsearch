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
    F: Fn((u32, u32, u32, HandSignature, [f32; 5], f32, u128)) + Sync + Send,
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

        // Find best discard
        let (best_suit_idx, max_suit_prob) = suit_probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let optimal_discard = if prob_null >= prob_grand && prob_null >= *max_suit_prob {
            discard_null
        } else if prob_grand >= *max_suit_prob {
            discard_grand
        } else {
            suit_discards[best_suit_idx]
        };

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
