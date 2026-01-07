mod args;

use clap::Parser;
use skat_aug23::extensions::solver::{solve, solve_win};
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::defs::{CLUBS, DIAMONDS, HEARTS, SPADES};
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::traits::{BitConverter, StringConverter};
use std::fs;

fn main() {
    let output = args::Cli::parse();

    match output.command {
        args::Commands::ValueCalc { context } => {
            let context_content = fs::read_to_string(context).expect("Unable to read context file");
            let input: args::GameContextInput =
                serde_json::from_str(&context_content).expect("JSON was not well-formatted");

            let mut game_context = GameContext::create(
                input.declarer_cards.__bit(),
                input.left_cards.__bit(),
                input.right_cards.__bit(),
                input.game_type,
                input.start_player,
            );

            if let Some(trick_cards_str) = input.trick_cards {
                game_context.set_trick_cards(trick_cards_str.__bit());
            }

            if let Some(trick_suit_str) = input.trick_suit {
                let suit = match trick_suit_str.to_lowercase().as_str() {
                    "clubs" | "c" => CLUBS,
                    "spades" | "s" => SPADES,
                    "hearts" | "h" => HEARTS,
                    "diamonds" | "d" => DIAMONDS,
                    _ => 0, // Or handle error
                };
                game_context.set_trick_suit(suit);
            }

            let threshold_upper = match input.mode {
                args::SearchMode::Win => 61, // Example for win check
                args::SearchMode::Value => 120,
            };
            game_context.set_threshold_upper(threshold_upper);

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let mut engine = SkatEngine::new(game_context, None);

            match input.mode {
                args::SearchMode::Win => {
                    let result = solve_win(&mut engine);
                    println!(
                        "Declarer Wins: {}, Best Card: {}",
                        result.declarer_wins,
                        result.best_card.__str()
                    );
                }
                args::SearchMode::Value => {
                    let result = solve(&mut engine);
                    println!(
                        "Value: {}, Best Card: {}",
                        result.best_value,
                        result.best_card.__str()
                    );
                }
            }
        }
    }
}
