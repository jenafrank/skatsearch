use crate::consts::bitboard::{
    ACES, CLUBS, DIAMONDS, EIGHTS, HEARTS, JACKS, KINGS, NINES, QUEENS, SPADES, TENS, TRUMP_SUIT,
};
use crate::extensions::solver::{solve_optimum_from_position, OptimumMode};
use crate::pimc::facts::Facts;
use crate::pimc::pimc_problem_builder::PimcProblemBuilder;
use crate::pimc::pimc_search::PimcSearch;
use crate::skat::context::GameContext;
use crate::skat::defs::{Game, Player};
use crate::skat::engine::SkatEngine;
use crate::skat::rules::get_suit_for_card;
use crate::traits::{Bitboard, StringConverter};
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

fn log_distribution(ctx: &GameContext, game_type: Game, start_player: Player) {
    println!("Cards Distribution (Sorted for {:?}):", game_type);

    let sort = |c| sort_cards(c, game_type);

    let d_cards = sort(ctx.declarer_cards());
    let l_cards = sort(ctx.left_cards());
    let r_cards = sort(ctx.right_cards());
    let s_cards = sort(ctx.get_skat());

    println!("Declarer: {}", cards_to_string(&d_cards));
    println!("Left    : {}", cards_to_string(&l_cards));
    println!("Right   : {}", cards_to_string(&r_cards));
    println!("Skat    : {}", cards_to_string(&s_cards));
    println!("Start Player: {:?}", start_player);
}

fn cards_to_string(cards: &[u32]) -> String {
    cards
        .iter()
        .map(|c| c.__str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sort_cards(cards: u32, game_type: Game) -> Vec<u32> {
    let (arr, n) = cards.__decompose();
    let mut list = Vec::new();
    for i in 0..n {
        list.push(arr[i]);
    }

    list.sort_by(|&a, &b| {
        let val_a = get_sorting_value(a, game_type);
        let val_b = get_sorting_value(b, game_type);
        val_b.cmp(&val_a) // Descending order
    });
    list
}

fn get_sorting_value(card: u32, game_type: Game) -> i32 {
    let is_jack = (card & JACKS) != 0;

    match game_type {
        Game::Null => {
            let suit_val = if (card & CLUBS) != 0 {
                400
            } else if (card & SPADES) != 0 {
                300
            } else if (card & HEARTS) != 0 {
                200
            } else {
                100
            };

            let rank_val = if (card & ACES) != 0 {
                8
            } else if (card & KINGS) != 0 {
                7
            } else if (card & QUEENS) != 0 {
                6
            } else if (card & JACKS) != 0 {
                5
            } else if (card & TENS) != 0 {
                4
            } else if (card & NINES) != 0 {
                3
            } else if (card & EIGHTS) != 0 {
                2
            } else {
                1
            };

            suit_val + rank_val
        }
        Game::Grand => {
            if is_jack {
                if (card & CLUBS) != 0 {
                    1000
                } else if (card & SPADES) != 0 {
                    900
                } else if (card & HEARTS) != 0 {
                    800
                } else {
                    700
                }
            } else {
                let suit_val = if (card & CLUBS) != 0 {
                    400
                } else if (card & SPADES) != 0 {
                    300
                } else if (card & HEARTS) != 0 {
                    200
                } else {
                    100
                };

                let rank_val = if (card & ACES) != 0 {
                    7
                } else if (card & TENS) != 0 {
                    6
                } else if (card & KINGS) != 0 {
                    5
                } else if (card & QUEENS) != 0 {
                    4
                } else if (card & NINES) != 0 {
                    3
                } else if (card & EIGHTS) != 0 {
                    2
                } else {
                    1
                };
                suit_val + rank_val
            }
        }
        Game::Suit => {
            let trump_suit = TRUMP_SUIT;
            let is_trump = is_jack || (card & trump_suit) != 0;

            if is_trump {
                if is_jack {
                    if (card & CLUBS) != 0 {
                        2000
                    } else if (card & SPADES) != 0 {
                        1900
                    } else if (card & HEARTS) != 0 {
                        1800
                    } else {
                        1700
                    }
                } else {
                    // Trump Suit cards
                    let rank_val = if (card & ACES) != 0 {
                        7
                    } else if (card & TENS) != 0 {
                        6
                    } else if (card & KINGS) != 0 {
                        5
                    } else if (card & QUEENS) != 0 {
                        4
                    } else if (card & NINES) != 0 {
                        3
                    } else if (card & EIGHTS) != 0 {
                        2
                    } else {
                        1
                    };
                    1000 + rank_val
                }
            } else {
                // Non-Trump
                let suit_val = if (card & CLUBS) != 0 {
                    400
                } else if (card & SPADES) != 0 {
                    300
                } else if (card & HEARTS) != 0 {
                    200
                } else {
                    100
                };

                let rank_val = if (card & ACES) != 0 {
                    7
                } else if (card & TENS) != 0 {
                    6
                } else if (card & KINGS) != 0 {
                    5
                } else if (card & QUEENS) != 0 {
                    4
                } else if (card & NINES) != 0 {
                    3
                } else if (card & EIGHTS) != 0 {
                    2
                } else {
                    1
                };
                suit_val + rank_val
            }
        }
    }
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

    let mut current_trick: Vec<(Player, u32)> = Vec::new();
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
