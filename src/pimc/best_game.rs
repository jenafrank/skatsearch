use crate::extensions::skat_solving::{solve_with_skat, AccelerationMode};
use crate::skat::context::{GameContext, ProblemTransformation};
use crate::skat::defs::{Game, Player, ALLCARDS};
use crate::traits::{BitConverter, Bitboard, StringConverter};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{self, Write};

pub fn calculate_best_game(
    my_cards_str: &str,
    start_player: Player,
    samples: u32,
    log_file: Option<String>,
    verbose: bool,
) -> Vec<(String, f32)> {
    let my_cards = my_cards_str.__bit();

    // Validate we have 10 cards
    if my_cards.count_ones() != 10 {
        eprintln!("Error: Expected 10 cards, got {}", my_cards.count_ones());
        return vec![];
    }

    let all_cards_mask = ALLCARDS;
    let remaining_mask = all_cards_mask ^ my_cards;
    let (remaining_cards, count) = remaining_mask.__decompose();
    let mut remaining_vec: Vec<u32> = remaining_cards[0..count].to_vec();

    let mut wins_clubs = 0;
    let mut wins_spades = 0;
    let mut wins_hearts = 0;
    let mut wins_diamonds = 0;
    let mut wins_grand = 0;
    let mut wins_null = 0;

    let mut log_writer = if let Some(path) = &log_file {
        Some(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Could not open log file"),
        )
    } else {
        None
    };

    if let Some(w) = &mut log_writer {
        writeln!(w, "Best Game Analysis (Skat Pickup Simulation)").unwrap();
        writeln!(w, "My Cards: {}", my_cards_str).unwrap();
        writeln!(w, "Samples: {}", samples).unwrap();
        writeln!(w, "--------------------------------------------------").unwrap();
    }

    let mut rng = thread_rng();

    for i in 0..samples {
        remaining_vec.shuffle(&mut rng);

        // Distributions: 2 Skat, 10 Left, 10 Right
        let start_time = std::time::Instant::now();
        let skat_vec = &remaining_vec[0..2];
        let left_vec = &remaining_vec[2..12];
        let right_vec = &remaining_vec[12..22];

        let mut skat_mask = 0;
        for &c in skat_vec {
            skat_mask |= c;
        }

        let mut left_mask = 0;
        for &c in left_vec {
            left_mask |= c;
        }

        let mut right_mask = 0;
        for &c in right_vec {
            right_mask |= c;
        }

        let my_cards_with_skat = my_cards | skat_mask;

        if verbose {
            print!(".");
            io::stdout().flush().unwrap();
        }

        if let Some(w) = &mut log_writer {
            writeln!(w, "Sample {}:", i).unwrap();
            writeln!(w, "Skat: {}", skat_mask.__str()).unwrap();
            writeln!(w, "Left: {}", left_mask.__str()).unwrap();
            writeln!(w, "Right: {}", right_mask.__str()).unwrap();
            writeln!(w, "Result Scores (Win/Loss):").unwrap();
        }

        // Unroll or simpler loop
        let game_configs = [
            ("Clubs", Game::Suit, None),
            (
                "Spades",
                Game::Suit,
                Some(ProblemTransformation::SpadesSwitch),
            ),
            (
                "Hearts",
                Game::Suit,
                Some(ProblemTransformation::HeartsSwitch),
            ),
            (
                "Diamonds",
                Game::Suit,
                Some(ProblemTransformation::DiamondsSwitch),
            ),
            ("Grand", Game::Grand, None),
            ("Null", Game::Null, None),
        ];

        let mut scores = Vec::new();

        for (_, (name, game_type, transform)) in game_configs.iter().enumerate() {
            // Apply transformations
            let my_c = if let Some(t) = transform {
                GameContext::get_switched_cards(my_cards_with_skat, *t)
            } else {
                my_cards_with_skat
            };
            let left_c = if let Some(t) = transform {
                GameContext::get_switched_cards(left_mask, *t)
            } else {
                left_mask
            };
            let right_c = if let Some(t) = transform {
                GameContext::get_switched_cards(right_mask, *t)
            } else {
                right_mask
            };

            let solve_ret = solve_with_skat(
                left_c,
                right_c,
                my_c,
                *game_type,
                start_player,
                AccelerationMode::AlphaBetaAccelerating,
            );

            let mut pushed_skat_str = String::new();
            let mut hand_str = String::new();

            let best_val = if let Some(best) = solve_ret.best_skat {
                let pushed_load = best.skat_card_1 | best.skat_card_2;
                pushed_skat_str = pushed_load.__str();

                // The hand we play with is my_cards_with_skat minus pushed cards
                // Note: if transform is active, we must apply inverse or just use untransformed logic?
                // `my_cards_with_skat` is untransformed. `my_c` was transformed.
                // `best.skat_card_1` are relative to the input `my_c`.
                // So if we transformed `my_c` to Clubs, the returned skat cards are in that space.
                // We need to transform them back to display correct card names if we transformed.

                // Wait, solve_with_skat returns cards in the same domain as input.
                // If we passed a transformed context (e.g. Spades -> Clubs), the result cards are "Clubs".
                // We must untransform them to show real card names.

                // However, `calc_all_games` logic was:
                // `problem_farbe_gruen` is transformed.
                // If we use standard `solve_with_skat`, it returns indices/masks.

                // Let's look at `ProblemTransformation`.
                // We should probably just print the raw result if it's confusing, OR implement reverse transform.
                // BUT: `GameContext::get_switched_cards` is symmetric for Spades/Hearts/Diamonds switches if they are just swaps.
                // Checking `context.rs`:
                // SpadesSwitch: Spades <-> Clubs.
                // HeartsSwitch: Hearts <-> Clubs.
                // DiamondsSwitch: Diamonds <-> Clubs.
                // Yes, they are their own inverses.

                let real_pushed = if let Some(t) = transform {
                    GameContext::get_switched_cards(pushed_load, *t)
                } else {
                    pushed_load
                };

                pushed_skat_str = real_pushed.__str();

                let real_hand = my_cards_with_skat ^ real_pushed;
                hand_str = real_hand.__str();

                best.value
            } else {
                0
            };

            scores.push(best_val);

            // Logging
            if let Some(w) = &mut log_writer {
                writeln!(
                    w,
                    "  {:<9}: Score {:>3}, Hand: {}, Pushed: {}",
                    name, best_val, hand_str, pushed_skat_str
                )
                .unwrap();
            }
        }

        let duration = start_time.elapsed();

        if let Some(w) = &mut log_writer {
            writeln!(w, "Duration: {:.2?}", duration).unwrap();
            writeln!(w, "--------------------------------------------------").unwrap();
        }

        // Updates
        if scores[0] > 60 {
            wins_clubs += 1;
        }
        if scores[1] > 60 {
            wins_spades += 1;
        }
        if scores[2] > 60 {
            wins_hearts += 1;
        }
        if scores[3] > 60 {
            wins_diamonds += 1;
        }
        if scores[4] > 60 {
            wins_grand += 1;
        }
        if scores[5] == 0 {
            wins_null += 1;
        }
    }

    if verbose {
        println!(); // Newline after dots
    }

    let f_samples = samples as f32;
    let mut results_vec = vec![
        ("Clubs".to_string(), wins_clubs as f32 / f_samples),
        ("Spades".to_string(), wins_spades as f32 / f_samples),
        ("Hearts".to_string(), wins_hearts as f32 / f_samples),
        ("Diamonds".to_string(), wins_diamonds as f32 / f_samples),
        ("Grand".to_string(), wins_grand as f32 / f_samples),
        ("Null".to_string(), wins_null as f32 / f_samples),
    ];

    results_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results_vec
}
