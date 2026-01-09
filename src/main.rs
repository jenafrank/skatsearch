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

            let mut threshold_upper = match input.mode.as_ref() {
                Some(args::SearchMode::Win) => 61,
                Some(args::SearchMode::Value) => 120,
                None => 120,
            };

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let mut engine = SkatEngine::new(game_context, None);

            let mode = input.mode.unwrap_or(args::SearchMode::Value);

            match mode {
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
        args::Commands::Playout { context } => {
            println!("Reading context file: {}", context);
            let context_content = fs::read_to_string(context).expect("Unable to read context file");
            println!("Context content read. Parsing JSON...");
            let input: args::GameContextInput =
                serde_json::from_str(&context_content).expect("JSON was not well-formatted");
            println!("JSON parsed successfully.");

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
                    _ => 0,
                };
                game_context.set_trick_suit(suit);
            }

            game_context.set_threshold_upper(61);

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            println!("Calling skat_aug23::pimc::playout::playout...");
            skat_aug23::pimc::playout::playout(game_context, 20);
        }
        args::Commands::StandardPlayout { context } => {
            println!("Reading context file: {}", context);
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
                    _ => 0,
                };
                game_context.set_trick_suit(suit);
            }

            // Standard playout uses full information, so threshold might matter less for 'playout' line by line,
            // but engine needs it.
            game_context.set_threshold_upper(120);

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let mut engine = SkatEngine::new(game_context, None);
            println!("Calling skat_aug23::extensions::playout::playout...");
            let lines = skat_aug23::extensions::playout::playout(&mut engine);

            for line in lines {
                println!(
                    "Player {:?} played {}. (Decl: {}, Team: {})",
                    line.player,
                    line.card.__str(),
                    line.declarer_points,
                    line.team_points
                );
            }
        }
        args::Commands::AnalysisPlayout { context } => {
            println!("Reading context file: {}", context);
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
                    _ => 0,
                };
                game_context.set_trick_suit(suit);
            }

            game_context.set_threshold_upper(120);

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let mut engine = SkatEngine::new(game_context, None);
            println!("Calling skat_aug23::extensions::playout::playout_all_cards...");
            let lines = skat_aug23::extensions::playout::playout_all_cards(&mut engine);

            for line in lines {
                println!(
                    "PLAYER {:?}: Best Card: {} (Decl: {})",
                    line.player,
                    line.best_card.__str(),
                    line.declarer_points,
                );

                let mut sorted_results = line.all_cards.results.clone();
                // Sort by value descending (for display)
                sorted_results.sort_by(|a, b| b.2.cmp(&a.2));

                println!("  Analysis ({:?}):", line.player);
                for (card, _resp, val) in sorted_results {
                    println!("    Card: {} -> Value: {}", card.__str(), val);
                }
                println!("--------------------------------------------------");
            }
        }
        args::Commands::Analysis { context } => {
            println!("Reading context file: {}", context);
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
                    _ => 0,
                };
                game_context.set_trick_suit(suit);
            }

            game_context.set_threshold_upper(120);

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let mut engine = SkatEngine::new(game_context, None);
            println!("Calling skat_aug23::extensions::solver::solve_all_cards...");
            let result = skat_aug23::extensions::solver::solve_all_cards(&mut engine, 0, 120);

            let mut sorted_results = result.results.clone();
            // Sort by value descending
            sorted_results.sort_by(|a, b| b.2.cmp(&a.2));

            println!("Analysis for Player {:?}:", engine.context.start_player);
            for (card, _resp, val) in sorted_results {
                println!("    Card: {} -> Value: {}", card.__str(), val);
            }
        }
    }
}
