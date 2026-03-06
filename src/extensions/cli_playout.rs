use crate::consts::bitboard::{
    CLUBS, DIAMONDS, EIGHTS, HEARTS, JACKS, KINGS, NINES, NULL_CLUBS, NULL_DIAMONDS, NULL_HEARTS,
    NULL_SPADES, QUEENS, SEVENS, SPADES, TENS,
};
use crate::extensions::skat_solving::{solve_best_game_all_variants, AccelerationMode};
use crate::extensions::solver::{solve_all_cards_from_position, solve_optimum_from_position, OptimumMode};
use crate::pimc::facts::Facts;
use crate::pimc::pimc_problem_builder::PimcProblemBuilder;
use crate::pimc::pimc_search::PimcSearch;
use crate::skat::context::GameContext;
use crate::skat::defs::{Game, Player};
use crate::skat::engine::SkatEngine;
use crate::skat::rules::get_suit_for_card;
use crate::traits::{Bitboard, Points, StringConverter};
use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FallbackStrategy {
    Average,
    Minimum,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointStrategy {
    Average,
    Minimum,
    Hybrid {
        delta: f32,
        fallback: FallbackStrategy,
    },
    /// Sort by avg_points, but within `threshold` points of the best move prefer
    /// trump (declarer) or non-trump (opponent) as a realistic move heuristic.
    AverageWithHeuristic {
        threshold: f32,
    },
}

impl PointStrategy {
    pub fn from_args(mode: &str, delta: f32, fallback: &str, heuristic: Option<f32>) -> Self {
        if let Some(threshold) = heuristic {
            return PointStrategy::AverageWithHeuristic { threshold };
        }
        let fb = match fallback.to_lowercase().as_str() {
            "minimum" | "min" => FallbackStrategy::Minimum,
            _ => FallbackStrategy::Average,
        };
        match mode.to_lowercase().as_str() {
            "minimum" | "min" => PointStrategy::Minimum,
            "hybrid" => PointStrategy::Hybrid {
                delta,
                fallback: fb,
            },
            _ => PointStrategy::Average,
        }
    }
}

pub fn run_playout(
    initial_context: GameContext,
    game_type: Game,
    start_player: Player,
    samples: u32,
    sampling_mode: crate::pimc::pimc_problem::SamplingMode,
) {
    println!("=== Playout Configuration ===");
    println!("Game Type: {:?}", game_type);
    println!("Start Player: {:?}", start_player);
    println!("PIMC Samples: {}", samples);
    println!("=============================\n");

    // 2. Log Distribution
    log_distribution(&initial_context, game_type, start_player);

    // 3. Run Perfect Play (Reference)
    println!("\n=== Perfect Play Simulation ===");
    let perfect_score = run_perfect_play(&initial_context, game_type);

    // 4. Run PIMC Play (Comparison)
    println!("\n=== PIMC Play Simulation ===");
    run_pimc_play(
        initial_context,
        game_type,
        samples,
        perfect_score,
        sampling_mode,
    );
}

pub fn generate_random_deal(
    game_type_str: String,
    start_player_str: String,
    mode: crate::pimc::pimc_problem::SamplingMode,
) -> (GameContext, Game, Player) {
    let game_type = match game_type_str.to_lowercase().as_str() {
        "grand" => Game::Grand,
        "null" => Game::Null,
        // All suit variants use Game::Suit internally (engine uses Clubs as canonical trump)
        "clubs" | "suit" | "spades" | "hearts" | "diamonds" => Game::Suit,
        _ => panic!("Invalid game type: {}", game_type_str),
    };

    let start_player = match start_player_str.to_lowercase().as_str() {
        "declarer" => Player::Declarer,
        "left" => Player::Left,
        "right" => Player::Right,
        _ => panic!("Invalid start player: {}", start_player_str),
    };

    let is_acceptable = |ctx: &GameContext| -> bool {
        match mode {
            crate::pimc::pimc_problem::SamplingMode::Random => true,
            crate::pimc::pimc_problem::SamplingMode::LikelyNull => {
                crate::pimc::pimc_problem::is_likely_null_hand(ctx.left_cards())
                    && crate::pimc::pimc_problem::is_likely_null_hand(ctx.right_cards())
            }
            crate::pimc::pimc_problem::SamplingMode::SmartGrand => {
                crate::pimc::pimc_problem::is_playable_grand_hand(ctx.declarer_cards())
            }
            crate::pimc::pimc_problem::SamplingMode::SmartSuit => {
                crate::pimc::pimc_problem::is_playable_suit_hand(ctx.declarer_cards())
            }
        }
    };

    let mut rng = rand::thread_rng();

    // Try up to 200 times to deal an acceptable hand
    for _ in 0..200 {
        let mut deck: Vec<u8> = (0..32).collect();
        deck.shuffle(&mut rng);

        let mut declarer_cards = 0u32;
        let mut left_cards = 0u32;
        let mut right_cards = 0u32;

        for i in 0..10 {
            declarer_cards |= 1 << deck[i];
        }
        for i in 10..20 {
            left_cards |= 1 << deck[i];
        }
        for i in 20..30 {
            right_cards |= 1 << deck[i];
        }

        let context = GameContext::create(
            declarer_cards,
            left_cards,
            right_cards,
            game_type,
            start_player,
        );

        if is_acceptable(&context) {
            return (context, game_type, start_player);
        }
    }

    // Fallback: Just return a random deal if condition not met after 200 tries
    let mut deck: Vec<u8> = (0..32).collect();
    deck.shuffle(&mut rng);
    let mut declarer_cards = 0u32;
    let mut left_cards = 0u32;
    let mut right_cards = 0u32;
    for i in 0..10 {
        declarer_cards |= 1 << deck[i];
    }
    for i in 10..20 {
        left_cards |= 1 << deck[i];
    }
    for i in 20..30 {
        right_cards |= 1 << deck[i];
    }
    let context = GameContext::create(
        declarer_cards,
        left_cards,
        right_cards,
        game_type,
        start_player,
    );
    (context, game_type, start_player)
}

/// Tries to generate one "interesting" deal:
///   1. Shuffle a full 32-card deck (declarer gets 12, left/right each 10).
///   2. Run perfect-information best-game for Grand and all 4 Suit variants
///      (Null excluded). Skip Null entirely.
///   3. Take the variant with the highest value.  Reject if value < min_value.
///   4. Apply the optimal 2-card discard → declarer now holds 10 cards.
///   5. Apply any suit transformation so the GameContext uses Clubs order.
///   6. Return (context_10cards, game_type, start_player, label, discard_str).
///
/// `type_filter`: if Some(Game::Grand) only Grand deals are accepted;
/// if Some(Game::Suit) only Suit deals are accepted; None = accept both.
///
/// Returns `None` if the random deal did not qualify.
/// Returns `(ctx, game_type, start_player, label, discard_str, original_12_cards)`.
/// `original_12_cards` is the declarer's 12-card pre-discard hand in the
/// (possibly suit-transformed) card space, matching the card names used in `ctx`.
pub fn generate_smart_deal_with_min_typed(
    min_value: u8,
    type_filter: Option<Game>,
) -> Option<(GameContext, Game, Player, String, String, u32)> {
    let start_player = Player::Declarer; // Declarer always leads first trick

    let mut rng = rand::thread_rng();
    let mut deck: Vec<u8> = (0..32).collect();
    deck.shuffle(&mut rng);

    // 12 cards for declarer (includes 2 Skat cards), 10 each for Left / Right.
    let mut decl_12 = 0u32;
    let mut left_cards = 0u32;
    let mut right_cards = 0u32;
    for i in 0..12 {
        decl_12 |= 1 << deck[i];
    }
    for i in 12..22 {
        left_cards |= 1 << deck[i];
    }
    for i in 22..32 {
        right_cards |= 1 << deck[i];
    }

    // Run best-game for Grand + all 4 Suit variants (skip Null).
    let all_variants = solve_best_game_all_variants(
        decl_12,
        left_cards,
        right_cards,
        start_player,
        AccelerationMode::AlphaBetaAccelerating,
    );

    // Filter out Null; optionally restrict to Grand or Suit only; pick best value.
    let best = all_variants
        .into_iter()
        .filter(|r| r.game_type != Game::Null)
        .filter(|r| match type_filter {
            Some(Game::Grand) => r.game_type == Game::Grand,
            Some(Game::Suit) => r.game_type == Game::Suit,
            _ => true,
        })
        .max_by_key(|r| r.value)?;

    let skat_discard = best.skat_1 | best.skat_2;
    let discard_str = format!("{} {}", best.skat_1.__str(), best.skat_2.__str());
    let skat_points = skat_discard.points();

    // best.value is inflated: evaluate_skat_combination adds skat_points on top of
    // solve_double_dummy's result, which already includes skat via create_initial_position().
    // So best.value = true_score + skat_points. Correct for this before filtering.
    let true_score = best.value.saturating_sub(skat_points);
    if true_score < min_value {
        return None;
    }

    // The declarer's final 10-card hand after discard.
    let decl_10 = decl_12 & !skat_discard;

    // Apply suit transformation to Left/Right if needed (Spades/Hearts/Diamonds).
    let (d_final, l_final, r_final) = match best.transformation {
        None => (decl_10, left_cards, right_cards),
        Some(trans) => (
            GameContext::get_switched_cards(decl_10, trans),
            GameContext::get_switched_cards(left_cards, trans),
            GameContext::get_switched_cards(right_cards, trans),
        ),
    };

    let ctx = GameContext::create(d_final, l_final, r_final, best.game_type, start_player);

    // Original 12-card hand in the same (possibly transformed) card space as ctx.
    let skat_final = match best.transformation {
        None => best.skat_1 | best.skat_2,
        Some(trans) => GameContext::get_switched_cards(best.skat_1 | best.skat_2, trans),
    };
    let decl_12_final = d_final | skat_final;

    Some((ctx, best.game_type, start_player, best.label, discard_str, decl_12_final))
}

/// Backward-compatible wrapper: uses the old 50-pt threshold, no type filter.
pub fn generate_smart_deal_with_min(min_value: u8) -> Option<(GameContext, Game, Player, String, String)> {
    generate_smart_deal_with_min_typed(min_value, None)
        .map(|(ctx, g, p, l, d, _)| (ctx, g, p, l, d))
}

/// Backward-compatible wrapper: uses the old 50-pt threshold.
pub fn generate_smart_deal() -> Option<(GameContext, Game, Player, String, String)> {
    generate_smart_deal_with_min(50)
}

use crate::skat::formatter::format_hand_for_game;

fn log_distribution(ctx: &GameContext, game_type: Game, start_player: Player) {
    let game_name = match game_type {
        Game::Grand => "Grand",
        Game::Null => "Null",
        Game::Suit => "Clubs", // Fallback for Suit
    };

    println!("Cards Distribution (Sorted for {:?}):", game_type);

    println!(
        "Declarer: {}",
        format_hand_for_game(ctx.declarer_cards(), game_name)
    );
    println!(
        "Left    : {}",
        format_hand_for_game(ctx.left_cards(), game_name)
    );
    println!(
        "Right   : {}",
        format_hand_for_game(ctx.right_cards(), game_name)
    );
    println!(
        "Skat    : {}",
        format_hand_for_game(ctx.get_skat(), game_name)
    );
    println!("Start Player: {:?}", start_player);
}

fn player_abbr(p: Player) -> &'static str {
    match p {
        Player::Declarer => "D",
        Player::Left => "L",
        Player::Right => "R",
    }
}

fn run_perfect_play(ctx: &GameContext, _game_type: Game) -> i16 {
    let mut engine = SkatEngine::new(ctx.clone(), None);
    let mut position = engine.create_initial_position();
    let mut trick_num = 1;

    let mut trick_buf: Vec<(Player, u32)> = Vec::with_capacity(3);

    println!("Starting Perfect Play...");
    while position.get_legal_moves() != 0 {
        let (best_card, _, _) =
            solve_optimum_from_position(&mut engine, &position, OptimumMode::BestValue).unwrap();

        trick_buf.push((position.player, best_card));
        position = position.make_move(best_card, ctx);

        if trick_buf.len() == 3 {
            let cards_str: String = trick_buf
                .iter()
                .map(|(p, c)| format!("{}:{}", player_abbr(*p), c.__str()))
                .collect::<Vec<_>>()
                .join("  ");
            println!("  Trick {:2}  {}", trick_num, cards_str);
            trick_num += 1;
            trick_buf.clear();

            // Null: game ends the moment the declarer takes any trick.
            if ctx.game_type() == Game::Null && position.declarer_points > 0 {
                break;
            }
        }
    }

    let final_val = if ctx.game_type() == Game::Null {
        if position.declarer_points > 0 {
            1
        } else {
            0
        }
    } else {
        position.declarer_points as i16
    };
    println!("Perfect Play Finished. Result: {} pts", final_val);
    final_val
}

// Facts Tracker Struct
struct FactsTracker {
    declarer: Facts,
    left: Facts,
    right: Facts,
}

impl FactsTracker {
    fn new() -> Self {
        Self {
            declarer: Facts::zero_fact(),
            left: Facts::zero_fact(),
            right: Facts::zero_fact(),
        }
    }

    fn update_voids(&mut self, played_card: u32, player: Player, trick_suit: u32, game_type: Game) {
        if trick_suit == 0 {
            return;
        } // Lead, no voids

        let card_suit = get_suit_for_card(played_card, game_type);
        if card_suit != trick_suit {
            // Player is VOID in trick_suit
            let facts = match player {
                Player::Declarer => &mut self.declarer,
                Player::Left => &mut self.left,
                Player::Right => &mut self.right,
            };

            // Map trick_suit to Facts fields
            let trump = game_type.get_trump();
            if trick_suit == trump {
                facts.no_trump = true;
            } else {
                if (trick_suit & CLUBS) != 0 {
                    facts.no_clubs = true;
                } else if (trick_suit & SPADES) != 0 {
                    facts.no_spades = true;
                } else if (trick_suit & HEARTS) != 0 {
                    facts.no_hearts = true;
                } else if (trick_suit & DIAMONDS) != 0 {
                    facts.no_diamonds = true;
                }
            }
        }
    }
}

fn run_pimc_play(
    initial_ctx: GameContext,
    _game_type: Game,
    samples: u32,
    _perfect_benchmark_val: i16,
    sampling_mode: crate::pimc::pimc_problem::SamplingMode,
) {
    let mut engine = SkatEngine::new(initial_ctx.clone(), None);
    let mut position = engine.create_initial_position();
    let mut trick_num = 0usize;
    let mut cards_in_trick = 0u8;

    let mut declarer_loss = 0i16;
    let mut opponent_loss = 0i16;
    let mut facts_tracker = FactsTracker::new();
    let mut current_trick: Vec<(Player, u32)> = Vec::new();

    println!("Starting PIMC Play...");
    while position.get_legal_moves() != 0 {
        // ── Trick separator ─────────────────────────────────────────────────────
        if cards_in_trick == 0 {
            trick_num += 1;
            current_trick.clear();
            println!(
                "-- Trick {:2} ({} leads) ----------------------------------------------------------------",
                trick_num, player_abbr(position.player)
            );
        }

        let cur_player = position.player;

        // 1. Get Perfect Move & Value (Benchmark)
        let (perfect_card, _, perfect_val_u8) =
            solve_optimum_from_position(&mut engine, &position, OptimumMode::BestValue).unwrap();
        let perfect_val = perfect_val_u8 as i16;

        // 2. Build PIMC problem
        let pred = match cur_player {
            Player::Declarer => Player::Right,
            Player::Left => Player::Declarer,
            Player::Right => Player::Left,
        };
        let succ = match cur_player {
            Player::Declarer => Player::Left,
            Player::Left => Player::Right,
            Player::Right => Player::Declarer,
        };
        let prev_c = current_trick
            .iter()
            .find(|(p, _)| *p == pred)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let next_c = current_trick
            .iter()
            .find(|(p, _)| *p == succ)
            .map(|(_, c)| *c)
            .unwrap_or(0);

        let all_game_cards_pimc = engine.context.declarer_cards()
            | engine.context.left_cards()
            | engine.context.right_cards();

        // position.declarer_points already includes skat (added by create_initial_position
        // at game start). The PIMC sample's create_initial_position will add skat again,
        // so we subtract it here to avoid double-counting.
        let pimc_decl_pts = position
            .declarer_points
            .saturating_sub(engine.context.get_skat().points());

        let builder = PimcProblemBuilder::new(engine.context.game_type())
            .my_player(cur_player)
            .my_cards_val(position.player_cards)
            .all_cards_val(all_game_cards_pimc & !position.played_cards)
            .turn(cur_player)
            .threshold(engine.context.points_to_win())
            .declarer_start_points(pimc_decl_pts)
            .trick_previous_player(position.trick_suit, prev_c)
            .trick_next_player(next_c)
            .facts(Player::Declarer, facts_tracker.declarer)
            .facts(Player::Left, facts_tracker.left)
            .facts(Player::Right, facts_tracker.right)
            .sampling_mode(sampling_mode)
            .declarer_played_cards(initial_ctx.declarer_cards() & !position.declarer_cards)
            .all_played_cards(position.played_cards);

        let mut search = PimcSearch::new(builder.build(), samples, None);
        let probs = search.estimate_probability_of_all_cards(false);

        // Pick best card
        let pimc_card = if probs.is_empty() {
            let (arr, _) = position.get_legal_moves().__decompose();
            arr[0]
        } else {
            probs[0].0
        };

        // 3. Compare with Benchmark
        let actual_val = if pimc_card == perfect_card {
            perfect_val
        } else {
            let child_pos = position.make_move(pimc_card, &engine.context);
            if child_pos.get_legal_moves() == 0 {
                if engine.context.game_type() == Game::Null {
                    if child_pos.declarer_points > 0 { 1i16 } else { 0i16 }
                } else {
                    child_pos.declarer_points as i16
                }
            } else {
                let (_, _, val) =
                    solve_optimum_from_position(&mut engine, &child_pos, OptimumMode::BestValue)
                        .unwrap_or((0, 0, 0));
                val as i16
            }
        };

        let raw_loss = if engine.context.game_type() == Game::Null {
            if cur_player == Player::Declarer { actual_val - perfect_val } else { perfect_val - actual_val }
        } else {
            if cur_player == Player::Declarer { perfect_val - actual_val } else { actual_val - perfect_val }
        };
        let loss = raw_loss.max(0);
        let is_decl = cur_player == Player::Declarer;
        if loss > 0 {
            if is_decl { declarer_loss += loss; } else { opponent_loss += loss; }
        }

        // 4. Update State
        facts_tracker.update_voids(
            pimc_card,
            cur_player,
            position.trick_suit,
            engine.context.game_type(),
        );
        current_trick.push((cur_player, pimc_card));
        position = position.make_move(pimc_card, &engine.context);
        cards_in_trick = (cards_in_trick + 1) % 3;

        // 5. Print card line
        let diff_mark = if pimc_card != perfect_card && loss > 0 { "(!)" } else { "   " };
        let who = if is_decl { "D" } else { "O" };
        let probs_str: String = probs
            .iter()
            .map(|(c, p)| format!("{}={:.0}%", c.__str(), p * 100.0))
            .collect::<Vec<_>>()
            .join("  ");

        println!(
            "  {} {}  PIMC:{} {}  perf:{} opt={:3}  loss={}({})  probs:[{}]",
            player_abbr(cur_player),
            if pimc_card != perfect_card { "*" } else { " " },
            diff_mark,
            pimc_card.__str(),
            perfect_card.__str(),
            perfect_val,
            loss,
            who,
            probs_str,
        );

        if cards_in_trick == 0 {
            println!(
                "  +- score after trick {:2}: {} pts",
                trick_num, position.declarer_points
            );
        }
    }

    println!(
        "Game Finished. Total Point Loss: {} (D:{} O:{}) | Final score: {} pts",
        declarer_loss + opponent_loss,
        declarer_loss,
        opponent_loss,
        position.declarer_points
    );
}

// ---------------------------------------------------------------------------
// Average-Points PIMC Playout
// ---------------------------------------------------------------------------

/// Returns the highest-set bit of `x` as an isolated bit mask.
/// Panics if `x == 0`.
#[inline]
fn highest_bit(x: u32) -> u32 {
    debug_assert!(x != 0);
    1u32 << (31 - x.leading_zeros())
}

/// Returns `true` if `candidate_card` (single bit) would win the trick when
/// played as the **last** (3rd) card.
///
/// `trick_cards_so_far` contains exactly 2 bits (the two already-played cards).
/// `trick_suit` is the lead suit established by the first card of the trick.
fn would_win_trick_regular(
    candidate_card: u32,
    trick_cards_so_far: u32,
    trick_suit: u32,
    game_type: Game,
) -> bool {
    if trick_cards_so_far == 0 {
        return false;
    }
    let trump = game_type.get_trump();
    let is_trump_played = (trick_cards_so_far & trump) != 0;
    let candidate_is_trump = (candidate_card & trump) != 0;

    if is_trump_played {
        // Effective suit is trump: candidate must also be trump and beat the
        // current best trump.
        if !candidate_is_trump {
            return false;
        }
        let trump_in_trick = trick_cards_so_far & trump;
        return candidate_card > highest_bit(trump_in_trick);
    }

    // No trump played yet.
    if candidate_is_trump {
        // Playing trump on a non-trump trick always wins (legal play only).
        return true;
    }

    // Neither trump played nor candidate is trump: compare within lead suit.
    let candidate_in_lead = (candidate_card & trick_suit) != 0;
    if !candidate_in_lead {
        return false; // discarding off-suit – can never win
    }

    let lead_in_trick = trick_cards_so_far & trick_suit;
    if lead_in_trick == 0 {
        // Neither played card followed the lead suit – candidate is the only one.
        return true;
    }

    candidate_card > highest_bit(lead_in_trick)
}

/// Entry point for the points-maximising PIMC playout.
/// First runs `run_perfect_play` as a benchmark, then `run_pimc_points_play`.
pub fn run_points_playout(
    initial_context: GameContext,
    game_type: Game,
    start_player: Player,
    samples: u32,
    sampling_mode: crate::pimc::pimc_problem::SamplingMode,
    point_strategy: PointStrategy,
) {
    println!("=== Points Playout Configuration ===");
    println!("Game Type: {:?}", game_type);
    println!("Start Player: {:?}", start_player);
    println!("PIMC Samples: {}", samples);
    println!("Strategy: {:?}", point_strategy);
    println!("=====================================\n");

    log_distribution(&initial_context, game_type, start_player);

    println!("\n=== Perfect Play Simulation ===");
    let perfect_score = run_perfect_play(&initial_context, game_type);

    println!("\n=== PIMC Points Play Simulation ===");
    run_pimc_points_play(
        initial_context,
        game_type,
        samples,
        perfect_score,
        sampling_mode,
        point_strategy,
    );
}

/// Like `run_pimc_play` but selects moves by **average expected declarer points**
/// instead of binary win probability.
fn run_pimc_points_play(
    initial_ctx: GameContext,
    _game_type: Game,
    samples: u32,
    _perfect_benchmark_val: i16,
    sampling_mode: crate::pimc::pimc_problem::SamplingMode,
    point_strategy: PointStrategy,
) {
    let mut engine = SkatEngine::new(initial_ctx.clone(), None);
    let mut position = engine.create_initial_position();
    let mut trick_num = 0usize;
    let mut cards_in_trick = 0u8;

    let mut current_trick = Vec::<(Player, u32)>::new();
    let mut declarer_loss = 0i16;
    let mut opponent_loss = 0i16;
    let mut facts_tracker = FactsTracker::new();

    println!("Starting PIMC Points Play...");
    while position.get_legal_moves() != 0 {
        // ── Trick separator ─────────────────────────────────────────────────────
        if cards_in_trick == 0 {
            trick_num += 1;
            current_trick.clear();
            println!(
                "-- Trick {:2} ({} leads) ----------------------------------------------------------------",
                trick_num, player_abbr(position.player)
            );
        }

        // ── Perfect benchmark ───────────────────────────────────────────────────
        let cur_player = position.player;
        let (perfect_card, _, pv_u8) =
            solve_optimum_from_position(&mut engine, &position, OptimumMode::BestValue).unwrap();
        let perfect_val = pv_u8 as i16;
        // Per-card perfect values (TT already warm from the call above → fast).
        let all_perf = solve_all_cards_from_position(&mut engine, &position, 0, 120);
        let perfect_map: std::collections::HashMap<u32, u8> =
            all_perf.results.iter().map(|(c, _, v)| (*c, *v)).collect();

        // ── PIMC problem ────────────────────────────────────────────────────────
        let unplayed = position.get_all_unplayed_cards();
        let pred = match cur_player {
            Player::Declarer => Player::Right,
            Player::Left => Player::Declarer,
            Player::Right => Player::Left,
        };
        let succ = match cur_player {
            Player::Declarer => Player::Left,
            Player::Left => Player::Right,
            Player::Right => Player::Declarer,
        };
        let prev_c = current_trick
            .iter()
            .find(|(p, _)| *p == pred)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let next_c = current_trick
            .iter()
            .find(|(p, _)| *p == succ)
            .map(|(_, c)| *c)
            .unwrap_or(0);

        // Use only the cards actually in the game, not ALLCARDS (excludes the 2 Skat cards)
        let all_game_cards = engine.context.declarer_cards()
            | engine.context.left_cards()
            | engine.context.right_cards();

        // position.declarer_points already includes skat (added by create_initial_position
        // at game start). The PIMC sample's create_initial_position will add skat again,
        // so we subtract it here to avoid double-counting.
        let pimc_decl_pts = position
            .declarer_points
            .saturating_sub(engine.context.get_skat().points());

        let builder = PimcProblemBuilder::new(engine.context.game_type())
            .my_player(cur_player)
            .my_cards_val(position.player_cards)
            .all_cards_val(all_game_cards & !position.played_cards)
            .turn(cur_player)
            .threshold(engine.context.points_to_win())
            .declarer_start_points(pimc_decl_pts)
            .trick_previous_player(position.trick_suit, prev_c)
            .trick_next_player(next_c)
            .facts(Player::Declarer, facts_tracker.declarer)
            .facts(Player::Left, facts_tracker.left)
            .facts(Player::Right, facts_tracker.right)
            .sampling_mode(sampling_mode)
            .declarer_played_cards(initial_ctx.declarer_cards() & !position.declarer_cards)
            .all_played_cards(position.played_cards);

        let mut scores =
            PimcSearch::new(builder.build(), samples, None).estimate_move_metrics(false);

        // Tracks whether the trump heuristic overrode the pure avg-points best card.
        // Only set to true for AverageWithHeuristic when the chosen card changes.
        let mut trump_heuristic_overrode = false;

        match point_strategy {
            PointStrategy::Average => {
                scores.sort_by(|a, b| {
                    b.1.avg_points
                        .partial_cmp(&a.1.avg_points)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PointStrategy::Minimum => {
                scores.sort_by(|a, b| {
                    b.1.min_points
                        .partial_cmp(&a.1.min_points)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PointStrategy::Hybrid { delta, fallback } => {
                // Sort primarily by win_prob. But if the difference is < delta, fallback to the secondary metric.
                scores.sort_by(|a, b| {
                    let win_cmp =
                        b.1.win_prob
                            .partial_cmp(&a.1.win_prob)
                            .unwrap_or(std::cmp::Ordering::Equal);
                    if win_cmp == std::cmp::Ordering::Equal
                        || (b.1.win_prob - a.1.win_prob).abs() <= delta
                    {
                        match fallback {
                            FallbackStrategy::Average => {
                                b.1.avg_points
                                    .partial_cmp(&a.1.avg_points)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            }
                            FallbackStrategy::Minimum => {
                                b.1.min_points
                                    .partial_cmp(&a.1.min_points)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            }
                        }
                    } else {
                        win_cmp
                    }
                });
            }
            PointStrategy::AverageWithHeuristic { threshold } => {
                // ── Phase 1: sort by avg_points to establish the baseline winner ──
                scores.sort_by(|a, b| {
                    b.1.avg_points
                        .partial_cmp(&a.1.avg_points)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let pure_avg_best = scores.first().map(|(c, _)| *c);

                if !scores.is_empty() {
                    let best_pts = scores[0].1.avg_points;
                    let game_type = engine.context.game_type();
                    let trump_mask = game_type.get_trump();
                    let is_declarer = cur_player == Player::Declarer;

                    // Trump preference rules:
                    //   Declarer always prefers trump (Grand + Suit).
                    //   Opponents prefer non-trump ONLY in Suit games (not Grand –
                    //   Grand's only trump is the four Jacks, avoiding them makes
                    //   less tactical sense for defenders).
                    let apply_trump_pref =
                        is_declarer || game_type == Game::Suit;

                    // ── Phase 2: card-value tiebreak for 3rd player ──────────
                    // When we're last to play (cards_in_trick == 2) and all
                    // threshold-eligible candidates share the same trick outcome
                    // (all win or all lose), break ties by card point value:
                    //   winning trick → play highest value  (collect more points fast)
                    //   losing trick  → play lowest value   (don't waste good cards)
                    let value_sort: Option<bool> = if cards_in_trick == 2 {
                        let played: u32 = current_trick
                            .iter()
                            .map(|(_, c)| *c)
                            .fold(0u32, |a, c| a | c);
                        let ts = position.trick_suit;
                        let in_range_outcomes: Vec<bool> = scores
                            .iter()
                            .filter(|(_, m)| m.avg_points >= best_pts - threshold)
                            .map(|(c, _)| {
                                would_win_trick_regular(*c, played, ts, game_type)
                            })
                            .collect();
                        if in_range_outcomes.iter().all(|&w| w) {
                            Some(true) // all win → pick highest card value
                        } else if in_range_outcomes.iter().all(|&w| !w) {
                            Some(false) // all lose → pick lowest card value
                        } else {
                            None // mixed – no value tiebreak
                        }
                    } else {
                        None
                    };

                    // ── Phase 3: unified sort with four-level priority ────────
                    scores.sort_by(|a, b| {
                        use std::cmp::Ordering::*;
                        let a_in = a.1.avg_points >= best_pts - threshold;
                        let b_in = b.1.avg_points >= best_pts - threshold;

                        // P1: in-range beats out-of-range
                        if a_in != b_in {
                            return if b_in { Greater } else { Less };
                        }

                        if a_in {
                            // P2: trump preference
                            if apply_trump_pref {
                                let a_trump = (a.0 & trump_mask) != 0;
                                let b_trump = (b.0 & trump_mask) != 0;
                                if a_trump != b_trump {
                                    let a_pref =
                                        if is_declarer { a_trump } else { !a_trump };
                                    let b_pref =
                                        if is_declarer { b_trump } else { !b_trump };
                                    if a_pref != b_pref {
                                        return if b_pref { Greater } else { Less };
                                    }
                                }
                            }

                            // P3: card-value tiebreak (3rd player only)
                            if let Some(prefer_high) = value_sort {
                                let va = a.0.card_points();
                                let vb = b.0.card_points();
                                let vord = if prefer_high {
                                    vb.cmp(&va) // DESC – highest value first
                                } else {
                                    va.cmp(&vb) // ASC – lowest value first
                                };
                                if vord != Equal {
                                    return vord;
                                }
                            }
                        }

                        // P4: avg_points descending (baseline)
                        b.1.avg_points
                            .partial_cmp(&a.1.avg_points)
                            .unwrap_or(Equal)
                    });
                }

                // Detect override: heuristic chose a different card than pure avg.
                trump_heuristic_overrode =
                    scores.first().map(|(c, _)| *c) != pure_avg_best;
            }
        }

        let (pimc_card, all_scores_str): (u32, Vec<(u32, String)>) = if scores.is_empty() {
            let (arr, _) = position.get_legal_moves().__decompose();
            (arr[0], vec![])
        } else {
            let str_scores = scores
                .iter()
                .map(|(c, m)| match point_strategy {
                    PointStrategy::Average => (*c, format!("{:.0}", m.avg_points)),
                    PointStrategy::Minimum => (*c, format!("{:.0}", m.min_points)),
                    PointStrategy::Hybrid { fallback, .. } => {
                        let fval = match fallback {
                            FallbackStrategy::Average => m.avg_points,
                            FallbackStrategy::Minimum => m.min_points,
                        };
                        (*c, format!("{:.0}%/{:.0}", m.win_prob * 100.0, fval))
                    }
                    PointStrategy::AverageWithHeuristic { .. } => {
                        let decl_pv = perfect_map.get(c).copied().unwrap_or(0);
                        let pv = if cur_player == Player::Declarer {
                            decl_pv as i16
                        } else {
                            120 - decl_pv as i16
                        };
                        (*c, format!("{:.0}(\u{00B1}{:.1})|{}", m.avg_points, m.std_dev, pv))
                    }
                })
                .collect();
            (scores[0].0, str_scores)
        };

        // ── Loss ────────────────────────────────────────────────────────────────
        let actual_val = if pimc_card == perfect_card {
            perfect_val
        } else {
            let child = position.make_move(pimc_card, &engine.context);
            if child.get_legal_moves() == 0 {
                if engine.context.game_type() == Game::Null {
                    if child.declarer_points > 0 {
                        1i16
                    } else {
                        0i16
                    }
                } else {
                    child.declarer_points as i16
                }
            } else {
                let (_, _, v) =
                    solve_optimum_from_position(&mut engine, &child, OptimumMode::BestValue)
                        .unwrap_or((0, 0, 0));
                v as i16
            }
        };
        let raw_loss = if engine.context.game_type() == Game::Null {
            if cur_player == Player::Declarer {
                actual_val - perfect_val
            } else {
                perfect_val - actual_val
            }
        } else {
            if cur_player == Player::Declarer {
                perfect_val - actual_val
            } else {
                actual_val - perfect_val
            }
        };
        let loss = raw_loss.max(0);
        let is_decl = cur_player == Player::Declarer;
        if loss > 0 {
            if is_decl {
                declarer_loss += loss;
            } else {
                opponent_loss += loss;
            }
        }

        // ── Update state ────────────────────────────────────────────────────────
        facts_tracker.update_voids(
            pimc_card,
            cur_player,
            position.trick_suit,
            engine.context.game_type(),
        );
        current_trick.push((cur_player, pimc_card));
        position = position.make_move(pimc_card, &engine.context);
        cards_in_trick = (cards_in_trick + 1) % 3;

        // ── Print card line ─────────────────────────────────────────────────────
        // "(!)" only when a different card was chosen AND it caused a loss
        let diff_mark = if pimc_card != perfect_card && loss > 0 {
            "(!)"
        } else {
            "   "
        };
        // "^" marks when the trump heuristic overrode the pure avg-points best card.
        let perf_mark = if pimc_card != perfect_card { "*" } else { " " };
        let trump_mark = if trump_heuristic_overrode { "^" } else { " " };
        let who = if is_decl { "D" } else { "O" };
        let scores_str: String = all_scores_str
            .iter()
            .map(|(c, v)| format!("{}={}", c.__str(), v))
            .collect::<Vec<_>>()
            .join("  ");

        println!(
            "  {} {}{}  PIMC:{} {}  perf:{} opt={:3}  loss={}({})  avgs:[{}]",
            player_abbr(cur_player),
            perf_mark,
            trump_mark,
            diff_mark,
            pimc_card.__str(),
            perfect_card.__str(),
            perfect_val,
            loss,
            who,
            scores_str,
        );

        if cards_in_trick == 0 {
            println!(
                "  +- score after trick {:2}: {} pts",
                trick_num, position.declarer_points
            );
        }
    }

    println!(
        "Game Finished. Total Point Loss: {} (D:{} O:{}) | Final score: {} pts",
        declarer_loss + opponent_loss,
        declarer_loss,
        opponent_loss,
        position.declarer_points
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Null Playout
// ─────────────────────────────────────────────────────────────────────────────

/// Entry point for the Null-specific PIMC playout.
use crate::pimc::pimc_problem::SamplingMode;

/// Runs `run_perfect_play` as a benchmark, then `run_pimc_null_play`.
pub fn run_null_playout(initial_context: GameContext, samples: u32, mode: SamplingMode) {
    let game_type = Game::Null;
    let start_player = initial_context.start_player();

    println!("=== Null Playout Configuration ===");
    println!("Game Type: Null");
    println!("Start Player: {:?}", start_player);
    println!("PIMC Samples: {}", samples);
    println!("===================================\n");

    log_distribution(&initial_context, game_type, start_player);

    println!("\n=== Perfect Play Simulation ===");
    let perfect_score = run_perfect_play(&initial_context, game_type);

    println!("\n=== PIMC Null Play Simulation ===");
    run_pimc_null_play(initial_context, samples, perfect_score, mode);
}

/// Null-order rank: 7=1, 8=2, 9=3, J=4, Q=5, K=6, T=7, A=8.
/// Lower rank = weaker card = loses easier (good for declarer to discard).
fn null_rank(card: u32) -> u8 {
    if (card & SEVENS) != 0 {
        1
    } else if (card & EIGHTS) != 0 {
        2
    } else if (card & NINES) != 0 {
        3
    } else if (card & JACKS) != 0 {
        4
    } else if (card & QUEENS) != 0 {
        5
    } else if (card & KINGS) != 0 {
        6
    } else if (card & TENS) != 0 {
        7
    } else {
        8
    } // Ace
}

/// Returns `true` if `card` would WIN the current trick under Null rules.
///
/// Uses NULL_CLUBS/NULL_SPADES/NULL_HEARTS/NULL_DIAMONDS which include Jacks
/// (the regular CLUBS/SPADES/HEARTS/DIAMONDS constants exclude Jacks).
fn would_win_trick_null(
    card: u32,
    trick_lead_suit: u32, // 0 if this player IS the lead
    trick_best_card: u32, // current trick winner so far (0 if no card yet)
) -> bool {
    if trick_lead_suit == 0 || trick_best_card == 0 {
        return false; // this player leads — not "beating" anyone yet
    }

    // Determine lead suit from trick_best_card using null-aware suit masks.
    let lead_suit_mask = if (trick_best_card & NULL_CLUBS) != 0 {
        NULL_CLUBS
    } else if (trick_best_card & NULL_SPADES) != 0 {
        NULL_SPADES
    } else if (trick_best_card & NULL_HEARTS) != 0 {
        NULL_HEARTS
    } else if (trick_best_card & NULL_DIAMONDS) != 0 {
        NULL_DIAMONDS
    } else {
        return false; // unknown suit — shouldn't happen
    };

    // Card must follow the lead suit to be able to win.
    if (card & lead_suit_mask) == 0 {
        return false;
    }

    // Follows suit — wins if its Null rank is strictly higher.
    null_rank(card) > null_rank(trick_best_card)
}

/// Null card selection — constraint-first, then PIMC, then positional tiebreaker.
///
/// Step 0 – Hard constraints (applied regardless of probability):
///   Declarer not leading:
///     Must play a non-winning (safe) card if any exist.
///     → candidate pool = safe cards; fallback: all cards (forced win).
///   Opponent, teammate already winning (opponent_has_trick):
///     Must NOT steal the trick from a teammate.
///     → candidate pool = non-winning cards; fallback: all cards (forced win).
///   Declarer leading / Opponent in all other cases:
///     → candidate pool = all legal cards.
///
/// Step 1 – PIMC probability filter:
///   Among the candidate pool, keep only those with maximum win-probability.
///   (Irrelevant cards below the max are excluded.)
///
/// Step 2 – Positional tiebreaker (among max-prob candidates):
///   Declarer leading            → LOWEST card.
///   Declarer not leading        → HIGHEST card (dump dangerous high values).
///   Opponent leading            → LOWEST card (kleine Karte anführen).
///   Opponent, opponent_has_trick→ HIGHEST card (dump high values).
///   Opponent, declarer winning  → HIGHEST non-winning card (let declarer keep stich).
fn select_null_card(
    probs: &[(u32, f32)],
    legal_mask: u32,
    is_declarer: bool,
    trick_lead_suit: u32,
    trick_best_card: u32,
    opponent_has_trick: bool,
) -> u32 {
    let (arr, n) = legal_mask.__decompose();
    let legal: Vec<u32> = (0..n).map(|i| arr[i]).collect();
    let is_lead = trick_lead_suit == 0;

    let min_card = |v: &Vec<u32>| {
        v.iter()
            .cloned()
            .min_by_key(|&c| null_rank(c))
            .unwrap_or(legal[0])
    };
    let max_card = |v: &Vec<u32>| {
        v.iter()
            .cloned()
            .max_by_key(|&c| null_rank(c))
            .unwrap_or(legal[0])
    };
    let non_winning_of = |pool: &Vec<u32>| -> Vec<u32> {
        pool.iter()
            .cloned()
            .filter(|&c| !would_win_trick_null(c, trick_lead_suit, trick_best_card))
            .collect()
    };

    // ── Step 0: build hard-constrained candidate pool ────────────────────────
    let candidates: Vec<u32> = if is_declarer && !is_lead {
        // Declarer must not win the trick (play safe).
        let safe = non_winning_of(&legal);
        if safe.is_empty() {
            legal.clone()
        } else {
            safe
        }
    } else if !is_declarer && !is_lead && opponent_has_trick {
        // Teammate already winning: use ALL cards — dump the highest, even if it
        // oversteps the teammate (getting rid of dangerous high cards is the goal).
        legal.clone()
    } else if !is_declarer && !is_lead && !opponent_has_trick {
        // Declarer currently winning — opponent must let him keep it.
        let non_win = non_winning_of(&legal);
        if non_win.is_empty() {
            legal.clone()
        } else {
            non_win
        }
    } else {
        legal.clone() // leading: all cards are valid
    };

    // ── Step 1: keep only max-probability candidates ─────────────────────────
    let max_prob_candidates: Vec<u32> = if probs.is_empty() {
        candidates.clone()
    } else {
        let max_p = probs
            .iter()
            .filter(|(c, _)| candidates.contains(c))
            .map(|(_, p)| *p)
            .fold(f32::NEG_INFINITY, f32::max);
        let top: Vec<u32> = probs
            .iter()
            // Threshold 0.015: with 100 samples, 1 sample = 1% = 0.01 difference;
            // treat cards within ~1 sample of each other as equal probability.
            .filter(|(c, p)| candidates.contains(c) && (*p - max_p).abs() < 0.015)
            .map(|(c, _)| *c)
            .collect();
        if top.is_empty() {
            candidates.clone()
        } else {
            top
        }
    };

    // ── Step 2: positional tiebreaker among max-prob candidates ──────────────
    if is_declarer {
        if is_lead {
            min_card(&max_prob_candidates) // lead: stay small
        } else {
            min_card(&max_prob_candidates) // safe pool filtered; play lowest safe
        }
    } else if is_lead {
        min_card(&max_prob_candidates) // opponent leads: kleine Karte
    } else if opponent_has_trick {
        max_card(&max_prob_candidates) // teammate has trick: dump highest
    } else {
        // Declarer currently winning: non-winning pool already filtered; dump highest.
        max_card(&max_prob_candidates)
    }
}

fn run_pimc_null_play(
    initial_ctx: GameContext,
    samples: u32,
    _perfect_score: i16,
    mode: SamplingMode,
) {
    let mut engine = SkatEngine::new(initial_ctx.clone(), None);
    let mut position = engine.create_initial_position();
    let mut trick_num = 0usize;
    let mut cards_in_trick = 0u8;

    let mut current_trick = Vec::<(Player, u32)>::new();
    let mut facts_tracker = FactsTracker::new();

    // The card currently "winning" the trick and its lead suit.
    let mut trick_best_card = 0u32;

    let mut pimc_declarer_took = false; // did PIMC-declarer ever take a trick?

    println!("Starting PIMC Null Play...");
    while position.get_legal_moves() != 0 {
        // ── Trick separator ───────────────────────────────────────────────────
        if cards_in_trick == 0 {
            trick_num += 1;
            current_trick.clear();
            trick_best_card = 0;
            println!(
                "-- Trick {:2} ({} leads) --",
                trick_num,
                player_abbr(position.player)
            );
        }

        let cur_player = position.player;

        // ── Perfect benchmark ─────────────────────────────────────────────────
        let (perfect_card, _, _) =
            solve_optimum_from_position(&mut engine, &position, OptimumMode::BestValue).unwrap();

        // ── PIMC problem setup ────────────────────────────────────────────────
        let unplayed = position.get_all_unplayed_cards();
        let pred = match cur_player {
            Player::Declarer => Player::Right,
            Player::Left => Player::Declarer,
            Player::Right => Player::Left,
        };
        let succ = match cur_player {
            Player::Declarer => Player::Left,
            Player::Left => Player::Right,
            Player::Right => Player::Declarer,
        };
        let prev_c = current_trick
            .iter()
            .find(|(p, _)| *p == pred)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let next_c = current_trick
            .iter()
            .find(|(p, _)| *p == succ)
            .map(|(_, c)| *c)
            .unwrap_or(0);

        // Use only the cards actually in the game, not ALLCARDS (excludes the 2 Skat cards)
        let all_game_cards_null = engine.context.declarer_cards()
            | engine.context.left_cards()
            | engine.context.right_cards();

        let builder = PimcProblemBuilder::new(engine.context.game_type())
            .my_player(cur_player)
            .my_cards_val(position.player_cards)
            .all_cards_val(all_game_cards_null & !position.played_cards)
            .turn(cur_player)
            .threshold(engine.context.points_to_win())
            .declarer_start_points(position.declarer_points)
            .trick_previous_player(position.trick_suit, prev_c)
            .trick_next_player(next_c)
            .facts(Player::Declarer, facts_tracker.declarer)
            .facts(Player::Left, facts_tracker.left)
            .facts(Player::Right, facts_tracker.right)
            .sampling_mode(mode);

        let problem = builder.build();
        let search = PimcSearch::new(problem.clone(), samples, None);

        // --- DEBUG SKAT ROTATION ---
        let ctx = problem.generate_concrete_problem();
        let total_cards =
            ctx.declarer_cards() | ctx.left_cards() | ctx.right_cards() | ctx.trick_cards();
        if total_cards.count_ones() != 30 {
            eprintln!(
                "WARNING: Total cards in PIMC is {}! Skat size = {}",
                total_cards.count_ones(),
                32 - total_cards.count_ones()
            );
        }
        // ---------------------------

        let probs = search.estimate_probability_of_all_cards(false);

        // ── 3-tier card selection ─────────────────────────────────────────────
        let legal_mask = position.get_legal_moves();
        let is_declarer = cur_player == Player::Declarer;

        // Is the current trick already won by an opponent?
        // True when trick_best_card != 0 AND the player who played it is not the Declarer.
        let opponent_has_trick = trick_best_card != 0
            && current_trick
                .iter()
                .any(|(p, c)| *c == trick_best_card && *p != Player::Declarer);

        let pimc_card = select_null_card(
            &probs,
            legal_mask,
            is_declarer,
            position.trick_suit,
            trick_best_card,
            opponent_has_trick,
        );

        // ── Update trick-winner tracking ──────────────────────────────────────
        if would_win_trick_null(pimc_card, position.trick_suit, trick_best_card)
            || position.trick_suit == 0
        {
            trick_best_card = pimc_card;
        }

        // ── Probability string ────────────────────────────────────────────────
        let probs_str: String = probs
            .iter()
            .map(|(c, p)| format!("{}={:.0}%", c.__str(), p * 100.0))
            .collect::<Vec<_>>()
            .join("  ");

        // ── Check if PIMC card differs from perfect card ──────────────────────
        let diff_mark = if pimc_card != perfect_card { "*" } else { " " };

        println!(
            "  {} {}  PIMC:{} perf:{}  probs:[{}]",
            player_abbr(cur_player),
            diff_mark,
            pimc_card.__str(),
            perfect_card.__str(),
            probs_str,
        );

        // ── Update state ──────────────────────────────────────────────────────
        facts_tracker.update_voids(
            pimc_card,
            cur_player,
            position.trick_suit,
            engine.context.game_type(),
        );
        current_trick.push((cur_player, pimc_card));
        position = position.make_move(pimc_card, &engine.context);
        cards_in_trick = (cards_in_trick + 1) % 3;

        if cards_in_trick == 0 {
            // Trick just finished — did declarer take it?
            if position.declarer_points > 0 {
                pimc_declarer_took = true;
            }
            println!(
                "  +- after trick {:2}: declarer has {} pts{}",
                trick_num,
                position.declarer_points,
                if pimc_declarer_took {
                    "  ← STICH TAKEN!"
                } else {
                    ""
                }
            );
            // Null: stop immediately once the declarer takes a trick.
            if pimc_declarer_took {
                break;
            }
        }
    }

    // perfect_score > 0 means declarer took at least one trick (Null lost for declarer).
    let perfect_declarer_took = _perfect_score > 0;

    let pimc_result = if pimc_declarer_took { "LOST" } else { "WON" };
    let perfect_result = if perfect_declarer_took { "LOST" } else { "WON" };
    let loss = if pimc_declarer_took && !perfect_declarer_took {
        1
    } else {
        0
    };

    println!(
        "\nNull Game Finished. PIMC={} | Perfect={} | Loss={}",
        pimc_result, perfect_result, loss
    );
    // Emit the standard Total-Point-Loss line so the Python parser works.
    println!("Total Point Loss: {} (D:{} O:{})", loss, loss, 0);
}
