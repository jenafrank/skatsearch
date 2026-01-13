mod args;

use clap::Parser;
use skat_aug23::extensions::solver::{solve, solve_optimum, solve_win, OptimumMode};
use skat_aug23::pimc::facts::Facts;
use skat_aug23::pimc::pimc_problem_builder::PimcProblemBuilder;
use skat_aug23::pimc::pimc_search::PimcSearch;
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::defs::Player;
use skat_aug23::skat::defs::{CLUBS, DIAMONDS, HEARTS, SPADES};
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::traits::{BitConverter, Points, StringConverter};
use std::fs;

fn main() {
    let output = args::Cli::parse();

    match output.command {
        args::Commands::ValueCalc {
            context,
            optimum_mode,
        } => {
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
                eprintln!("DEBUG: ValueCalc Found trick_suit: '{}'", trick_suit_str);
                let suit = match trick_suit_str.to_lowercase().as_str() {
                    "clubs" | "c" => CLUBS,
                    "spades" | "s" => SPADES,
                    "hearts" | "h" => HEARTS,
                    "diamonds" | "d" => DIAMONDS,
                    "trump" | "t" => {
                        eprintln!(
                            "DEBUG: ValueCalc Matching Trump. GameType: {:?}",
                            game_context.game_type()
                        );
                        match game_context.game_type() {
                            skat_aug23::skat::defs::Game::Grand => {
                                eprintln!("DEBUG: ValueCalc Using TRUMP_GRAND");
                                skat_aug23::consts::bitboard::TRUMP_GRAND
                            }
                            _ => skat_aug23::consts::bitboard::TRUMP_SUIT,
                        }
                    }
                    _ => {
                        eprintln!("DEBUG: ValueCalc Match Failed for '{}'", trick_suit_str);
                        0
                    }
                };
                eprintln!("DEBUG: ValueCalc Parsed suit value: {}", suit);
                game_context.set_trick_suit(suit);
            } else {
                eprintln!("DEBUG: ValueCalc No trick_suit in input.");
            }

            if let Some(points) = input.declarer_start_points {
                game_context.set_declarer_start_points(points);
            } else {
                let total_cards = game_context.declarer_cards().count_ones()
                    + game_context.left_cards().count_ones()
                    + game_context.right_cards().count_ones()
                    + game_context.trick_cards().count_ones();

                if total_cards == 30
                    && game_context.game_type() != skat_aug23::skat::defs::Game::Null
                {
                    let skat = game_context.get_skat();
                    let points = skat.points();
                    println!(
                        "Auto-Skat: Determined {} points in Skat. Adding to declarer start points.",
                        points
                    );
                    game_context.set_declarer_start_points(points);
                }
            }

            let _threshold_upper = match input.mode.as_ref() {
                Some(args::SearchMode::Win) => 61,
                Some(args::SearchMode::Value) => 120,
                None => 120,
            };

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let mut engine = SkatEngine::new(game_context, None);

            if let Some(opt_str) = optimum_mode {
                let opt_mode = match opt_str.to_lowercase().as_str() {
                    "best_value" => OptimumMode::BestValue,
                    "all_winning" => OptimumMode::AllWinning,
                    _ => {
                        eprintln!(
                            "Invalid optimum mode: {}. Use 'best_value' or 'all_winning'.",
                            opt_str
                        );
                        std::process::exit(1);
                    }
                };
                println!("Running Optimum Search (Mode: {:?})...", opt_mode);
                match solve_optimum(&mut engine, opt_mode) {
                    Ok((best, score, val)) => println!(
                        "Optimum Best Move: {}, Score: {}, Value: {}",
                        best.__str(),
                        score,
                        val
                    ),
                    Err(e) => println!("Optimum Search Failed: {}", e),
                }
            } else {
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
        }
        args::Commands::PimcBestGame {
            context,
            samples,
            log_file,
        } => {
            println!("Calculating Best Game for context: {}", context);
            if let Some(path) = &log_file {
                println!("Logging detailed sample info to: {}", path);
                // clear file if exists
                fs::write(path, "").unwrap_or(());
            }

            let content = fs::read_to_string(&context).expect("Could not read file");
            let input: args::PimcBestGameInput =
                serde_json::from_str(&content).expect("JSON was not well-formatted");

            println!("My Cards: {}", input.my_cards);
            println!("Start Player: {:?}", input.start_player);
            println!("Samples per game: {}", samples);

            let results = skat_aug23::pimc::best_game::calculate_best_game(
                &input.my_cards,
                input.start_player,
                samples,
                log_file.clone(),
                true,
            );

            println!("\nBest Game:");
            println!("--------------------------------------------------");
            println!("{:<15} | {:<10}", "Game", "Win Prob");
            println!("--------------------------------------------------");
            for (game, prob) in results {
                println!("{:<15} | {:.4}", format!("{:?}", game), prob);
            }
            println!("--------------------------------------------------");
        }
        args::Commands::Playout { context, samples } => {
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

            if let Some(points) = input.declarer_start_points {
                game_context.set_declarer_start_points(points);
            } else {
                let total_cards = game_context.declarer_cards().count_ones()
                    + game_context.left_cards().count_ones()
                    + game_context.right_cards().count_ones()
                    + game_context.trick_cards().count_ones();

                if total_cards == 30
                    && game_context.game_type() != skat_aug23::skat::defs::Game::Null
                {
                    let skat = game_context.get_skat();
                    let points = skat.points();
                    println!(
                        "Auto-Skat: Determined {} points in Skat. Adding to declarer start points.",
                        points
                    );
                    game_context.set_declarer_start_points(points);
                }
            }

            game_context.set_threshold_upper(61);

            if let Err(e) = game_context.validate() {
                eprintln!("Validation Error: {}", e);
                std::process::exit(1);
            }

            let n_samples = samples.or(input.samples).unwrap_or(20);
            println!(
                "Calling skat_aug23::pimc::playout::playout with {} samples...",
                n_samples
            );
            skat_aug23::pimc::playout::playout(game_context, n_samples);
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

            if let Some(points) = input.declarer_start_points {
                game_context.set_declarer_start_points(points);
            } else {
                // Check for Auto-Skat-Points
                let total_cards = game_context.declarer_cards().count_ones()
                    + game_context.left_cards().count_ones()
                    + game_context.right_cards().count_ones()
                    + game_context.trick_cards().count_ones();

                if total_cards == 30
                    && game_context.game_type() != skat_aug23::skat::defs::Game::Null
                {
                    let skat = game_context.get_skat();
                    let points = skat.points();
                    println!(
                        "Auto-Skat: Determined {} points in Skat. Adding to declarer start points.",
                        points
                    );
                    game_context.set_declarer_start_points(points);
                }
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

            if let Some(points) = input.declarer_start_points {
                game_context.set_declarer_start_points(points);
            } else {
                let total_cards = game_context.declarer_cards().count_ones()
                    + game_context.left_cards().count_ones()
                    + game_context.right_cards().count_ones()
                    + game_context.trick_cards().count_ones();

                if total_cards == 30
                    && game_context.game_type() != skat_aug23::skat::defs::Game::Null
                {
                    let skat = game_context.get_skat();
                    let points = skat.points();
                    println!(
                        "Auto-Skat: Determined {} points in Skat. Adding to declarer start points.",
                        points
                    );
                    game_context.set_declarer_start_points(points);
                }
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

            if let Some(points) = input.declarer_start_points {
                game_context.set_declarer_start_points(points);
            } else {
                let total_cards = game_context.declarer_cards().count_ones()
                    + game_context.left_cards().count_ones()
                    + game_context.right_cards().count_ones()
                    + game_context.trick_cards().count_ones();

                if total_cards == 30
                    && game_context.game_type() != skat_aug23::skat::defs::Game::Null
                {
                    let skat = game_context.get_skat();
                    let points = skat.points();
                    println!(
                        "Auto-Skat: Determined {} points in Skat. Adding to declarer start points.",
                        points
                    );
                    game_context.set_declarer_start_points(points);
                }
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
        args::Commands::BestGame { context, mode } => {
            println!("Reading context file: {}", context);
            let context_content = fs::read_to_string(context).expect("Unable to read context file");
            let input: args::GameContextInput =
                serde_json::from_str(&context_content).expect("JSON was not well-formatted");

            use skat_aug23::skat::context::ProblemTransformation;
            use skat_aug23::traits::BitConverter;

            let declarer_cards = input.declarer_cards.__bit();
            let left_cards = input.left_cards.__bit();
            let right_cards = input.right_cards.__bit();
            let start_player = input.start_player;

            if declarer_cards.count_ones() != 12 {
                eprintln!("Declarer must have 12 cards for BestGame.");
                std::process::exit(1);
            }

            let acc_mode = match mode.to_lowercase().as_str() {
                "best" => {
                    skat_aug23::extensions::skat_solving::AccelerationMode::AlphaBetaAccelerating
                }
                "win" => skat_aug23::extensions::skat_solving::AccelerationMode::WinningOnly,
                _ => {
                    eprintln!("Invalid mode: {}. Use 'best' or 'win'.", mode);
                    std::process::exit(1);
                }
            };

            println!("Calculating best game (Mode: {})...", mode);

            let games_to_check = vec![
                (skat_aug23::skat::defs::Game::Grand, None, "Grand"),
                (skat_aug23::skat::defs::Game::Null, None, "Null"),
                (skat_aug23::skat::defs::Game::Suit, None, "Clubs"),
                (
                    skat_aug23::skat::defs::Game::Suit,
                    Some(ProblemTransformation::SpadesSwitch),
                    "Spades",
                ),
                (
                    skat_aug23::skat::defs::Game::Suit,
                    Some(ProblemTransformation::HeartsSwitch),
                    "Hearts",
                ),
                (
                    skat_aug23::skat::defs::Game::Suit,
                    Some(ProblemTransformation::DiamondsSwitch),
                    "Diamonds",
                ),
            ];

            let mut results = Vec::new();

            for (game_type, transformation, label) in games_to_check {
                // Apply transformation if needed
                let d_cards = if let Some(trans) = transformation {
                    GameContext::get_switched_cards(declarer_cards, trans)
                } else {
                    declarer_cards
                };
                let l_cards = if let Some(trans) = transformation {
                    GameContext::get_switched_cards(left_cards, trans)
                } else {
                    left_cards
                };
                let r_cards = if let Some(trans) = transformation {
                    GameContext::get_switched_cards(right_cards, trans)
                } else {
                    right_cards
                };

                let ret = skat_aug23::extensions::skat_solving::solve_with_skat(
                    l_cards,
                    r_cards,
                    d_cards,
                    game_type,
                    start_player,
                    acc_mode,
                );

                if let Some(best) = ret.best_skat {
                    // Transform skat cards back if needed
                    let (s1, s2) = if let Some(trans) = transformation {
                        // Switching back is the same operation (XOR swap logic)
                        (
                            GameContext::get_switched_cards(best.skat_card_1, trans),
                            GameContext::get_switched_cards(best.skat_card_2, trans),
                        )
                    } else {
                        (best.skat_card_1, best.skat_card_2)
                    };

                    results.push((label, s1, s2, best.value, game_type));
                }
            }

            // Output based on mode
            if mode.to_lowercase() == "win" {
                println!("Win/Loss Analysis:");
                results.sort_by(|a, b| b.3.cmp(&a.3)); // Sort by value anyway (high value wins usually)

                let is_win = |val: u8, game: skat_aug23::skat::defs::Game| -> bool {
                    match game {
                        skat_aug23::skat::defs::Game::Null => val == 0,
                        _ => val >= 61,
                    }
                };

                let mut found_win = false;
                for (label, s1, s2, val, g_type) in results {
                    if is_win(val, g_type) {
                        println!(
                            "Game: {:<10} Skat: {} {} -> WIN (Value: {})",
                            label,
                            s1.__str(),
                            s2.__str(),
                            val
                        );
                        found_win = true;
                    }
                }
                if !found_win {
                    println!("LOOSING (No winning game found)");
                }
            } else {
                // Best mode (default)
                results.sort_by(|a, b| b.3.cmp(&a.3));
                println!("Best Games Ranking:");
                for (label, s1, s2, val, _) in results {
                    println!(
                        "Game: {:<10} Skat: {} {} -> Value: {}",
                        label,
                        s1.__str(),
                        s2.__str(),
                        val
                    );
                }
            }
        }
        args::Commands::PimcCalc {
            context,
            mode,
            log_file,
        } => {
            println!("Reading context file: {}", context);
            let context_content = fs::read_to_string(context).expect("Unable to read context file");
            let input: args::PimcContextInput =
                serde_json::from_str(&context_content).expect("JSON was not well-formatted");

            let mut builder = PimcProblemBuilder::new(input.game_type)
                .my_player(input.my_player)
                .turn(input.my_player) // Assuming it's my turn if calculating? Or should it be separate?
                // Actually, for value calc, user usually wants to know value for current player.
                // Input has my_player.
                .cards(input.my_player, &input.my_cards)
                .remaining_cards(&input.remaining_cards);

            println!(
                "Context: {} - {}",
                input.game_type.convert_to_string(),
                input.my_player.str()
            );
            println!("My Cards: {}", input.my_cards.__bit().__str());
            println!("Remaining: {}", input.remaining_cards.__bit().__str());
            if let Some(_f) = &input.facts {
                println!("Facts: Present");
            }

            // Auto-Skat Logic
            let my_cards_bit = input.my_cards.__bit();
            let remaining_cards_bit = input.remaining_cards.__bit();
            let cards_in_play = my_cards_bit | remaining_cards_bit;

            if cards_in_play.count_ones() == 30 {
                let skat_cards = skat_aug23::consts::bitboard::ALLCARDS & !cards_in_play;
                let skat_points = skat_cards.points() as u8;
                println!("Auto-Skat: {} ({} Points)", skat_cards.__str(), skat_points);
                builder = builder.declarer_start_points(skat_points);
            } else if cards_in_play.count_ones() == 32 {
                // 32 cards? Skat might be in remaining?
                // Standard PIMC expects "remaining_cards" to be the UNKNOWN pool.
                // If Skat is unknown, it's just part of the game.
                // But valid board state for PIMC usually requires divisibility by 3.
                // 32 is not divisible by 3.
                // If 32 cards are provided, we assume 2 are Skat and 30 are in hands.
                // But we don't know WHICH are Skat.
                // This is a different problem (Hand Distribution inference).
                // We stick to 30-card auto-skat.
                println!("Note: 32 cards provided. Assuming Skat is unknown/distributed.");
            }

            if let Some(threshold) = input.threshold {
                builder = builder.threshold(threshold);
            } else {
                if input.game_type == skat_aug23::skat::defs::Game::Null {
                    builder = builder.threshold(1);
                } else {
                    builder = builder.threshold(61);
                }
            }

            if let Some(trick_str) = &input.trick_cards {
                let cards: Vec<&str> = trick_str.split_whitespace().collect();
                if !cards.is_empty() {
                    let prev = cards[0];
                    let next = if cards.len() > 1 { cards[1] } else { "" };
                    builder = builder.trick_from_uproblem(prev.to_string(), next.to_string());
                }
            } else if let (Some(prev), Some(next)) = (&input.previous_card, &input.next_card) {
                // Fallback for individual fields if still present (or remove if desired)
                builder = builder.trick_from_uproblem(prev.to_string(), next.to_string());
            } else if let Some(prev) = &input.previous_card {
                builder = builder.trick_from_uproblem(prev.to_string(), "".to_string());
            }

            if let Some(points) = input.declarer_start_points {
                builder = builder.declarer_start_points(points);
            }

            if let Some(facts_input) = input.facts {
                let convert_facts = |f_in: Option<args::PimcFactsInput>| -> Facts {
                    if let Some(f) = f_in {
                        let mut facts = Facts::zero_fact();
                        if let Some(true) = f.no_trump {
                            facts.no_trump = true;
                        }
                        if let Some(true) = f.no_clubs {
                            facts.no_clubs = true;
                        }
                        if let Some(true) = f.no_spades {
                            facts.no_spades = true;
                        }
                        if let Some(true) = f.no_hearts {
                            facts.no_hearts = true;
                        }
                        if let Some(true) = f.no_diamonds {
                            facts.no_diamonds = true;
                        }
                        facts
                    } else {
                        Facts::zero_fact()
                    }
                };

                builder = builder.facts(Player::Declarer, convert_facts(facts_input.declarer));
                builder = builder.facts(Player::Left, convert_facts(facts_input.left));
                builder = builder.facts(Player::Right, convert_facts(facts_input.right));
            }

            let problem = builder.build();
            let samples = input.samples.unwrap_or(100);
            let search = PimcSearch::new(problem, samples, log_file);

            match mode.to_lowercase().as_str() {
                "win" => {
                    println!("Estimating Win Probability ({} samples)...", samples);
                    let (prob, _) = search.estimate_win(false); // info=false for clean output
                    println!("Win Probability: {:.4}", prob);
                }
                "best" => {
                    println!("Estimating Best Move Values ({} samples)...", samples);
                    let results = search.estimate_probability_of_all_cards(false); // info=false for clean output
                    println!("Aggregate Results:");
                    for (card, prob) in results {
                        println!("Card: {} -> Win Prob: {:.4}", card.__str(), prob);
                    }
                }
                _ => {
                    eprintln!("Invalid mode: {}. Use 'best' or 'win'.", mode);
                }
            }
        }
    }
}
