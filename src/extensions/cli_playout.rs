use crate::consts::bitboard::{
    CLUBS, DIAMONDS, EIGHTS, HEARTS, JACKS, KINGS, NINES, NULL_CLUBS, NULL_DIAMONDS, NULL_HEARTS,
    NULL_SPADES, QUEENS, SEVENS, SPADES, TENS,
};
use crate::extensions::skat_solving::{solve_best_game_all_variants, AccelerationMode};
use crate::extensions::solver::{solve_optimum_from_position, OptimumMode};
use crate::pimc::facts::Facts;
use crate::pimc::pimc_problem_builder::PimcProblemBuilder;
use crate::pimc::pimc_search::PimcSearch;
use crate::skat::context::GameContext;
use crate::skat::defs::{Game, Player};
use crate::skat::engine::SkatEngine;
use crate::skat::rules::get_suit_for_card;
use crate::traits::{Bitboard, Points, StringConverter};
use rand::prelude::*;

pub fn run_playout(
    initial_context: GameContext,
    game_type: Game,
    start_player: Player,
    samples: u32,
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
    run_pimc_play(initial_context, game_type, samples, perfect_score);
}

pub fn generate_random_deal(
    game_type_str: String,
    start_player_str: String,
) -> (GameContext, Game, Player) {
    let game_type = match game_type_str.to_lowercase().as_str() {
        "grand" => Game::Grand,
        "null" => Game::Null,
        "clubs" | "suit" => Game::Suit, // Canonical Suit Game
        _ => panic!("Invalid game type: {}", game_type_str),
    };

    let start_player = match start_player_str.to_lowercase().as_str() {
        "declarer" => Player::Declarer,
        "left" => Player::Left,
        "right" => Player::Right,
        _ => panic!("Invalid start player: {}", start_player_str),
    };

    let mut rng = rand::thread_rng();
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
///   3. Take the variant with the highest value.  Reject if value < 50.
///   4. Apply the optimal 2-card discard → declarer now holds 10 cards.
///   5. Apply any suit transformation so the GameContext uses Clubs order.
///   6. Return (context_10cards, game_type, start_player, label, discard_str).
///
/// Returns `None` if the random deal did not qualify.
pub fn generate_smart_deal() -> Option<(GameContext, Game, Player, String, String)> {
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

    // Filter out Null, keep only Grand/Suit, pick best value.
    let best = all_variants
        .into_iter()
        .filter(|r| r.game_type != Game::Null)
        .max_by_key(|r| r.value)?;

    // Reject deals where perfect-play score is below 50 pts.
    if best.value < 50 {
        return None;
    }

    let skat_discard = best.skat_1 | best.skat_2;
    let discard_str = format!("{} {}", best.skat_1.__str(), best.skat_2.__str());
    let skat_points = skat_discard.points();

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

    let mut ctx = GameContext::create(d_final, l_final, r_final, best.game_type, start_player);
    ctx.set_declarer_start_points(skat_points);

    Some((ctx, best.game_type, start_player, best.label, discard_str))
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
) {
    let mut engine = SkatEngine::new(initial_ctx.clone(), None);
    let mut position = engine.create_initial_position();
    let mut trick_count = 1;

    let mut declarer_loss = 0;
    let mut opponent_loss = 0;
    let mut facts_tracker = FactsTracker::new();
    let mut current_trick: Vec<(Player, u32)> = Vec::new();

    println!("Starting PIMC Play...");
    while position.get_legal_moves() != 0 {
        if position.trick_cards_count == 0 {
            println!(
                "--- Trick {} (Player: {:?}) ---",
                trick_count, position.player
            );
            trick_count += 1;
            current_trick.clear();
        }

        // 1. Get Perfect Move & Value (Benchmark)
        let (perfect_card, _, perfect_val_u8) =
            solve_optimum_from_position(&mut engine, &position, OptimumMode::BestValue).unwrap();
        let perfect_val = perfect_val_u8 as i16;

        // 2. Get PIMC Move
        // We track unplayed cards from the position.
        // Position tracks 'declarer_cards', 'left_cards', 'right_cards'.
        // unplayed includes all keys.
        let unplayed = position.get_all_unplayed_cards();

        let prev_c = {
            let pred = match position.player {
                Player::Declarer => Player::Right,
                Player::Left => Player::Declarer,
                Player::Right => Player::Left,
            };
            current_trick
                .iter()
                .find(|(p, _)| *p == pred)
                .map(|(_, c)| *c)
                .unwrap_or(0)
        };

        let next_c = {
            let succ = match position.player {
                Player::Declarer => Player::Left,
                Player::Left => Player::Right,
                Player::Right => Player::Declarer,
            };
            current_trick
                .iter()
                .find(|(p, _)| *p == succ)
                .map(|(_, c)| *c)
                .unwrap_or(0)
        };

        let mut builder = PimcProblemBuilder::new(engine.context.game_type())
            .my_player(position.player)
            .my_cards_val(position.player_cards)
            .all_cards_val(unplayed | position.trick_cards)
            .turn(position.player)
            .threshold(engine.context.points_to_win())
            .declarer_start_points(position.declarer_points)
            .trick_previous_player(position.trick_suit, prev_c)
            .trick_next_player(next_c)
            .facts(Player::Declarer, facts_tracker.declarer)
            .facts(Player::Left, facts_tracker.left)
            .facts(Player::Right, facts_tracker.right);

        let problem = builder.build();

        let mut search = PimcSearch::new(problem, samples, None);
        let probs = search.estimate_probability_of_all_cards(false);

        // Pick best card
        let pimc_card = if probs.is_empty() {
            let moves_mask = position.get_legal_moves();
            let (arr, _) = moves_mask.__decompose();
            arr[0]
        } else {
            // Log top 3 probabilities
            print!("    PIMC Analysis:");
            for i in 0..std::cmp::min(3, probs.len()) {
                print!(" {} ({:.1})", probs[i].0.__str(), probs[i].1);
            }
            println!();
            probs[0].0
        };

        // 3. Compare with Benchmark
        let actual_val_of_pimc_move = if pimc_card == perfect_card {
            perfect_val
        } else {
            let child_pos = position.make_move(pimc_card, &engine.context);
            if child_pos.get_legal_moves() == 0 {
                if engine.context.game_type() == Game::Null {
                    if child_pos.declarer_points > 0 {
                        1i16
                    } else {
                        0i16
                    }
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

        let loss = if engine.context.game_type() == Game::Null {
            if position.player == Player::Declarer {
                actual_val_of_pimc_move - perfect_val
            } else {
                perfect_val - actual_val_of_pimc_move
            }
        } else {
            if position.player == Player::Declarer {
                perfect_val - actual_val_of_pimc_move
            } else {
                actual_val_of_pimc_move - perfect_val
            }
        };

        let loss = if loss < 0 { 0 } else { loss };
        if loss > 0 {
            if position.player == Player::Declarer {
                declarer_loss += loss;
            } else {
                opponent_loss += loss;
            }
        }

        println!("  PIMC plays: {} | Perfect was: {} | Loss: {} | (Perfect Val: {}, PIMC Action Val: {})", 
            pimc_card.__str(), perfect_card.__str(), loss, perfect_val, actual_val_of_pimc_move);

        // 4. Update State
        facts_tracker.update_voids(
            pimc_card,
            position.player,
            position.trick_suit,
            engine.context.game_type(),
        );
        current_trick.push((position.player, pimc_card));

        position = position.make_move(pimc_card, &engine.context);
    }

    println!(
        "Game Finished. Total Point Loss: {} (Declarer: {}, Opponents: {})",
        declarer_loss + opponent_loss,
        declarer_loss,
        opponent_loss
    );
}

// ---------------------------------------------------------------------------
// Average-Points PIMC Playout
// ---------------------------------------------------------------------------

/// Entry point for the points-maximising PIMC playout.
/// First runs `run_perfect_play` as a benchmark, then `run_pimc_points_play`.
pub fn run_points_playout(
    initial_context: GameContext,
    game_type: Game,
    start_player: Player,
    samples: u32,
) {
    println!("=== Points Playout Configuration ===");
    println!("Game Type: {:?}", game_type);
    println!("Start Player: {:?}", start_player);
    println!("PIMC Samples: {}", samples);
    println!("=====================================\n");

    log_distribution(&initial_context, game_type, start_player);

    println!("\n=== Perfect Play Simulation ===");
    let perfect_score = run_perfect_play(&initial_context, game_type);

    println!("\n=== PIMC Points Play Simulation ===");
    run_pimc_points_play(initial_context, game_type, samples, perfect_score);
}

/// Like `run_pimc_play` but selects moves by **average expected declarer points**
/// instead of binary win probability.
fn run_pimc_points_play(
    initial_ctx: GameContext,
    _game_type: Game,
    samples: u32,
    _perfect_benchmark_val: i16,
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

        let builder = PimcProblemBuilder::new(engine.context.game_type())
            .my_player(cur_player)
            .my_cards_val(position.player_cards)
            .all_cards_val(unplayed | position.trick_cards)
            .turn(cur_player)
            .threshold(engine.context.points_to_win())
            .declarer_start_points(position.declarer_points)
            .trick_previous_player(position.trick_suit, prev_c)
            .trick_next_player(next_c)
            .facts(Player::Declarer, facts_tracker.declarer)
            .facts(Player::Left, facts_tracker.left)
            .facts(Player::Right, facts_tracker.right);

        let scores =
            PimcSearch::new(builder.build(), samples, None).estimate_avg_points_of_all_cards(false);

        let (pimc_card, all_scores): (u32, Vec<(u32, f32)>) = if scores.is_empty() {
            let (arr, _) = position.get_legal_moves().__decompose();
            (arr[0], vec![])
        } else {
            (scores[0].0, scores.clone())
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
        let who = if is_decl { "D" } else { "O" };
        let scores_str: String = all_scores
            .iter()
            .map(|(c, v)| format!("{}={:.0}", c.__str(), v))
            .collect::<Vec<_>>()
            .join("  ");

        println!(
            "  {} {}  PIMC:{} {}  perf:{} opt={:3}  loss={}({})  avgs:[{}]",
            player_abbr(cur_player),
            if pimc_card != perfect_card { "*" } else { " " },
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

        let builder = PimcProblemBuilder::new(engine.context.game_type())
            .my_player(cur_player)
            .my_cards_val(position.player_cards)
            .all_cards_val(unplayed | position.trick_cards)
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
