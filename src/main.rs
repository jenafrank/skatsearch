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
                    "trump" | "t" => match game_context.game_type() {
                        skat_aug23::skat::defs::Game::Grand => {
                            skat_aug23::consts::bitboard::TRUMP_GRAND
                        }
                        _ => skat_aug23::consts::bitboard::TRUMP_SUIT,
                    },
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
                    "trump" | "t" => match game_context.game_type() {
                        skat_aug23::skat::defs::Game::Grand => {
                            skat_aug23::consts::bitboard::TRUMP_GRAND
                        }
                        _ => skat_aug23::consts::bitboard::TRUMP_SUIT,
                    },
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
                    "trump" | "t" => match game_context.game_type() {
                        skat_aug23::skat::defs::Game::Grand => {
                            skat_aug23::consts::bitboard::TRUMP_GRAND
                        }
                        _ => skat_aug23::consts::bitboard::TRUMP_SUIT,
                    },
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
                    "trump" | "t" => match game_context.game_type() {
                        skat_aug23::skat::defs::Game::Grand => {
                            skat_aug23::consts::bitboard::TRUMP_GRAND
                        }
                        _ => skat_aug23::consts::bitboard::TRUMP_SUIT,
                    },
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
        args::Commands::SkatCalc { context, mode } => {
            println!("Reading context file: {}", context);
            let context_content = fs::read_to_string(context).expect("Unable to read context file");
            let input: args::GameContextInput =
                serde_json::from_str(&context_content).expect("JSON was not well-formatted");

            let acc_mode = match mode.to_lowercase().as_str() {
                "best" => {
                    skat_aug23::extensions::skat_solving::AccelerationMode::AlphaBetaAccelerating
                }
                "all" => skat_aug23::extensions::skat_solving::AccelerationMode::NotAccelerating,
                "win" => skat_aug23::extensions::skat_solving::AccelerationMode::WinningOnly,
                _ => {
                    eprintln!("Invalid mode: {}. Use 'best', 'all', or 'win'.", mode);
                    std::process::exit(1);
                }
            };

            use skat_aug23::traits::BitConverter;
            let declarer_cards = input.declarer_cards.__bit();
            let left_cards = input.left_cards.__bit();
            let right_cards = input.right_cards.__bit();
            let game_type = input.game_type;
            let start_player = input.start_player;

            // Basic card count validation for Declarer (12 cards expected)
            if declarer_cards.count_ones() != 12 {
                eprintln!(
                    "Declarer must have 12 cards for SkatCalc. Found: {}",
                    declarer_cards.count_ones()
                );
                std::process::exit(1);
            }

            println!("Calculating best skat (Mode: {})...", mode);
            let ret = skat_aug23::extensions::skat_solving::solve_with_skat(
                left_cards,
                right_cards,
                declarer_cards,
                game_type,
                start_player,
                acc_mode,
            );

            match mode.to_lowercase().as_str() {
                "best" => {
                    if let Some(best) = ret.best_skat {
                        println!(
                            "Best Skat: {} {}, Value: {}",
                            best.skat_card_1.__str(),
                            best.skat_card_2.__str(),
                            best.value
                        );
                    } else {
                        println!("No skat solution found.");
                    }
                }
                "all" => {
                    println!("All Skat Combinations:");
                    let mut sorted = ret.all_skats.clone();
                    sorted.sort_by(|a, b| b.value.cmp(&a.value));
                    for line in sorted {
                        println!(
                            "Skat: {} {}, Value: {}",
                            line.skat_card_1.__str(),
                            line.skat_card_2.__str(),
                            line.value
                        );
                    }
                }
                "win" => {
                    println!("Win/Loss Analysis:");

                    let is_win = |val: u8, game: skat_aug23::skat::defs::Game| -> bool {
                        match game {
                            skat_aug23::skat::defs::Game::Null => val == 0,
                            _ => val >= 61,
                        }
                    };

                    if let Some(best) = ret.best_skat {
                        if is_win(best.value, game_type) {
                            println!(
                                "Skat: {} {} -> WIN (Value: {})",
                                best.skat_card_1.__str(),
                                best.skat_card_2.__str(),
                                best.value
                            );
                        } else {
                            println!("LOOSING");
                        }
                    } else {
                        println!("LOOSING");
                    }
                }
                _ => {}
            }
        }
    }
}
