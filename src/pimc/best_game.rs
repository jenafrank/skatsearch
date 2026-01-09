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
                AccelerationMode::WinningOnly,
            );

            let best_val = if let Some(best) = solve_ret.best_skat {
                best.value
            } else {
                0 // Should not happen if skat combinations exist
            };

            scores.push(best_val);

            // Logging
            if let Some(w) = &mut log_writer {
                writeln!(w, "  {}: {}", name, best_val).unwrap();
            }
        }

        if let Some(w) = &mut log_writer {
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
