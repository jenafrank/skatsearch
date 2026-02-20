mod args;

use clap::Parser;
use rand::seq::SliceRandom;
use skat_aug23::consts::bitboard::*;
use skat_aug23::extensions::solver::{solve, solve_optimum, solve_win, OptimumMode};
use skat_aug23::pimc::analysis::{
    analyze_general_pre_discard, analyze_hand, analyze_hand_with_pickup, analyze_null_detailed,
};
use skat_aug23::pimc::facts::Facts;
use skat_aug23::pimc::pimc_problem_builder::PimcProblemBuilder;
use skat_aug23::pimc::pimc_search::PimcSearch;
use skat_aug23::skat::context::GameContext;
use skat_aug23::skat::defs::Player;
use skat_aug23::skat::defs::{CLUBS, DIAMONDS, HEARTS, SPADES};
use skat_aug23::skat::engine::SkatEngine;
use skat_aug23::skat::signature::HandSignature;
use skat_aug23::traits::{BitConverter, Bitboard, Points, StringConverter};
use std::fs;
use std::io::Write;

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
        args::Commands::AnalyzeNull {
            count,
            samples,
            output,
            hand,
        } => {
            println!(
                "Analyzing Null Hands (Count: {}, Samples: {}, HandGame: {})...",
                count, samples, hand
            );

            let file = std::fs::File::create(output).expect("Could not create output file");
            writeln!(&file, "Hand,Skat,Won,Points,Moves,DurationMs,StartPlayer").unwrap();

            use std::sync::{Arc, Mutex};
            let file_mutex = Arc::new(Mutex::new(file));

            analyze_null_detailed(
                count,
                samples,
                hand,
                move |(hand_val, skat_val, won, points, moves, probs, duration, start_player)| {
                    let mut moves_probs_str = String::new();
                    for (i, card_val) in moves.iter().enumerate() {
                        if i > 0 {
                            moves_probs_str.push_str(", ");
                        }
                        let card_str = card_val.__str();
                        let prob = if i < probs.len() {
                            (probs[i] * 100.0).round() as i32
                        } else {
                            0 // Should not happen if lengths match
                        };

                        moves_probs_str.push_str(&format!("{}:{}", card_str, prob));
                    }

                    // --- SORTING HELPER ---
                    let format_null_cards = |cards_bit: u32| -> String {
                        let (list_arr, _) = cards_bit.__decompose();
                        let mut list: Vec<u32> =
                            list_arr.iter().cloned().filter(|&c| c != 0).collect();
                        list.sort_by(|&a, &b| {
                            // Suit Order: C > S > H > D (3, 2, 1, 0)
                            let suit_a = skat_aug23::skat::rules::get_suit_for_card(
                                a,
                                skat_aug23::skat::defs::Game::Null,
                            );
                            let suit_b = skat_aug23::skat::rules::get_suit_for_card(
                                b,
                                skat_aug23::skat::defs::Game::Null,
                            );

                            if suit_a != suit_b {
                                // Suits differ. Descending Suit order.
                                suit_b.cmp(&suit_a)
                            } else {
                                // Same Suit. Null Rank Order.
                                // 7 < 8 < 9 < 10 < J < Q < K < A
                                let get_rank = |c: u32| -> u8 {
                                    if (c & SEVENS) != 0 {
                                        0
                                    } else if (c & EIGHTS) != 0 {
                                        1
                                    } else if (c & NINES) != 0 {
                                        2
                                    } else if (c & TENS) != 0 {
                                        3
                                    } else if (c & JACKS) != 0 {
                                        4
                                    } else if (c & QUEENS) != 0 {
                                        5
                                    } else if (c & KINGS) != 0 {
                                        6
                                    } else if (c & ACES) != 0 {
                                        7
                                    } else {
                                        0
                                    }
                                };
                                get_rank(a).cmp(&get_rank(b))
                            }
                        });

                        let mut s = String::new();
                        for (i, c) in list.iter().enumerate() {
                            if i > 0 {
                                s.push(' ');
                            }
                            s.push_str(&c.__str());
                        }
                        s
                    };

                    let hand_str = format_null_cards(hand_val);
                    let skat_str = format_null_cards(skat_val);

                    let mut f = file_mutex.lock().unwrap();
                    writeln!(
                        f,
                        "\"{}\",\"{}\",{},{},\"{}\",{},{:?}",
                        hand_str, skat_str, won, points, moves_probs_str, duration, start_player
                    )
                    .unwrap();
                },
            );
            println!("Analysis complete.");
        }

        args::Commands::AnalyzeGrand {
            count,
            samples,
            output,
            hand,
            post_discard,
        } => {
            if hand {
                println!(
                    "Analyzing Grand Hand (Count: {}, Samples: {})...",
                    count, samples
                );
            } else if post_discard {
                println!(
                    "Analyzing Grand Post-Discard (Count: {}, Samples: {})...",
                    count, samples
                );
            } else {
                println!(
                    "Analyzing Grand with Pickup (Count: {}, Samples: {})...",
                    count, samples
                );
            }

            let mut writer: Box<dyn Write> = if let Some(path) = output {
                let file = std::fs::File::create(path).expect("Could not create output file");
                Box::new(file)
            } else {
                Box::new(std::io::stdout())
            };

            writeln!(writer, "{}", HandSignature::to_csv_header()).unwrap();

            let mut rng = rand::thread_rng();
            // We need 10 cards from 32.
            // Loop count times

            let mut vec: Vec<usize> = (0..32).collect();

            use std::time::Instant;

            let start_time = Instant::now();

            for i in 0..count {
                vec.shuffle(&mut rng);

                let mut my_hand = 0u32;
                for el in &vec[0..10] {
                    my_hand |= 1u32 << *el;
                }

                let mut skat = 0u32;
                for el in &vec[10..12] {
                    skat |= 1u32 << *el;
                }

                let (sig, prob, _) = if hand {
                    let (s, p) = analyze_hand(my_hand, samples);
                    (s, p, 0) // analyze_hand returns 2 values
                } else {
                    analyze_hand_with_pickup(my_hand, skat, samples, post_discard)
                };

                writeln!(writer, "{}", sig.to_csv_row(my_hand, skat, prob)).unwrap();

                // Progress update every 10 iterations or if count is small
                if (i + 1) % 10 == 0 || i + 1 == count {
                    let elapsed = start_time.elapsed();
                    let completed = i + 1;
                    let rate = completed as f64 / elapsed.as_secs_f64(); // items per second
                    let remaining_items = count - completed;
                    let eta_secs = if rate > 0.0 {
                        remaining_items as f64 / rate
                    } else {
                        0.0
                    };

                    let elapsed_str = format!(
                        "{:02}:{:02}:{:02}",
                        elapsed.as_secs() / 3600,
                        (elapsed.as_secs() % 3600) / 60,
                        elapsed.as_secs() % 60
                    );

                    let eta_str = format!(
                        "{:02}:{:02}:{:02}",
                        (eta_secs as u64) / 3600,
                        ((eta_secs as u64) % 3600) / 60,
                        (eta_secs as u64) % 60
                    );

                    let percent = (completed as f64 / count as f64) * 100.0;

                    print!("\rProgress: {:3.1}% | {}/{} | Elapsed: {} | ETA: {} | Rate: {:.2} games/s   ", 
                        percent, completed, count, elapsed_str, eta_str, rate);
                    std::io::stdout().flush().unwrap();
                }
            }
            println!("\nAnalysis complete.");
        }
        args::Commands::AnalyzeGeneral {
            count,
            samples,
            output,
        } => {
            println!(
                "Running General Pre-Discard Analysis with {} hands, {} samples...",
                count, samples
            );

            let mut file = std::fs::File::create(&output).expect("Could not create CSV file");
            // Custom header with specific order and alignment
            writeln!(
                file,
                "{:<35}, {:<10}, {:<35}, {:<10}, {:>7}, {:>15}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>15}, {:>5}, {:>5}, {:>5}, {:>6}, {:>6}, {:>6}, {:>5}, {:>7}, {:>5}, {:>8}, {:>10}, {:>10}, {:>10}, {:>11}, {:>13}, {:>8}, {:<8}, {:>8}, {:>9}, {:>12}",
                "InitHand", "InitSkat", "FinalHand", "SkatCards", "ISkFull", "JacksMask", "CntJ", "Aces", "Tens", "Att10", "Blk10", "MxLen", "TKS", "PostJacksMask", "PCntJ", "PAces", "PTens", "PAtt10", "PBlk10", "PMxLen", "PTKS", "PSkFull", "SkPts", "WinProb", "ProbGrand", "ProbClubs", "ProbSpades", "ProbHearts", "ProbDiamonds", "ProbNull", "WonMask", "MaxProb", "BestGame", "DurationMs"
            )
            .unwrap();

            // Wrap file in Mutex for thread safety (generic F requires Sync+Send in analysis.rs)
            use std::sync::{Arc, Mutex};
            let file_mutex = Arc::new(Mutex::new(file));

            analyze_general_pre_discard(
                count,
                samples,
                |(hand, skat, discard, sig, probs, prob_null, best_variant, duration_micros)| {
                    use skat_aug23::skat::formatter::format_hand_for_game;

                    // WonMask Logic (> 0.66)
                    let mut won_mask = String::with_capacity(6);
                    won_mask.push(if probs[0] > 0.66 { 'G' } else { '-' }); // Grand
                    won_mask.push(if probs[1] > 0.66 { 'C' } else { '-' }); // Clubs
                    won_mask.push(if probs[2] > 0.66 { 'S' } else { '-' }); // Spades
                    won_mask.push(if probs[3] > 0.66 { 'H' } else { '-' }); // Hearts
                    won_mask.push(if probs[4] > 0.66 { 'D' } else { '-' }); // Diamonds
                    won_mask.push(if prob_null > 0.66 { 'N' } else { '-' }); // Null

                    // BestGame Name and Prob from best_variant
                    let best_game_name = match best_variant {
                        0 => "Grand",
                        1 => "Clubs",
                        2 => "Spades",
                        3 => "Hearts",
                        4 => "Diamonds",
                        5 => "Null",
                        _ => "Unknown",
                    };

                    let final_max_prob = if best_variant == 5 {
                        prob_null
                    } else {
                        probs[best_variant as usize]
                    };

                    let final_hand = (hand | skat) ^ discard;
                    let init_hand_str = hand.__str(); // Use hand instead of cards_str
                    let init_skat_str = skat.__str();
                    let final_hand_str = format_hand_for_game(final_hand, best_game_name);
                    let discard_str = format_hand_for_game(discard, best_game_name);

                    // 1. InitSkatFulls
                    let init_skat_fulls = (skat & (ACES | TENS)).count_ones();

                    // 2. PostSkatFulls (Discard)
                    let post_skat_fulls = (discard & (ACES | TENS)).count_ones();

                    let mut skat_points = 0;
                    use skat_aug23::consts::bitboard::GRAND_CONN;
                    for (mask, val) in GRAND_CONN {
                        if (discard & mask) != 0 {
                            skat_points += val;
                        }
                    }

                    // Manually format the metric columns from sig
                    let jacks_str = sig.jacks_string();

                    // Row format matching header:
                    // InitHand,InitSkat,JacksMask,JackCount,Aces,Tens,AttachedTens,BlankTens,MaxSuitLen,TenKingSmall,FinalHand,SkatCards,SkatFulls,SkatPoints,WinProb,ProbGrand...

                    // Calculate Post-Discard metrics from final_hand
                    let post_sig = HandSignature::from_hand_and_skat_suit(final_hand, 0, None);

                    // Row format matching header:
                    // InitHand,InitSkat,
                    // JacksMask,JackCount,Aces,Tens,AttachedTens,BlankTens,MaxSuitLen,TenKingSmall,
                    // FinalHand,
                    // PostAces,PostTens,PostAttachedTens,PostBlankTens,PostMaxSuitLen,PostTenKingSmall,
                    // SkatCards,SkatFulls,SkatPoints,WinProb,ProbGrand...

                    let duration_ms = duration_micros as f64 / 1000.0;

                    // 4. Formatted Row with Padding
                    let row_str = format!(
                    "{:<35}, {:<10}, {:<35}, {:<10}, {:>7}, {:>15}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>15}, {:>5}, {:>5}, {:>5}, {:>6}, {:>6}, {:>6}, {:>5}, {:>7}, {:>5}, {:>8.4}, {:>10.4}, {:>10.4}, {:>10.4}, {:>11.4}, {:>13.4}, {:>8.4}, {:<8}, {:>8.4}, {:>9}, {:>12.2}",
                    init_hand_str,
                    init_skat_str,
                    final_hand_str,
                    discard_str,
                    init_skat_fulls,
                    jacks_str,
                    sig.trump_count,
                    sig.aces,
                    sig.tens,
                    sig.attached_tens,
                    sig.blank_tens,
                    sig.max_suit_len,
                    sig.ten_king_small,
                    post_sig.jacks_string(), // PostJacksMask
                    post_sig.trump_count,    // PostJackCount
                    post_sig.aces,
                    post_sig.tens,
                    post_sig.attached_tens,
                    post_sig.blank_tens,
                    post_sig.max_suit_len,
                    post_sig.ten_king_small,
                    post_skat_fulls,
                    skat_points,
                    final_max_prob,
                    probs[0], probs[1], probs[2], probs[3], probs[4], prob_null, won_mask, final_max_prob, best_game_name,
                    duration_ms
                );

                    if let Ok(mut f) = file_mutex.lock() {
                        writeln!(f, "{}", row_str).unwrap();
                    }
                },
            );
            println!("Analysis complete. Results written to {}", output);
        }
        args::Commands::GenerateJson {
            count,
            min_win,
            output_dir,
        } => {
            println!(
                "Searching for {} winning scenarios (1 Jack + 2 Aces, Win > {:.2})...",
                count, min_win
            );
            fs::create_dir_all(&output_dir).expect("Could not create output directory");

            let mut rng = rand::thread_rng();
            let mut vec: Vec<usize> = (0..32).collect();
            let mut found = 0;
            let mut attempts = 0;

            use skat_aug23::pimc::analysis::analyze_hand_with_pickup;
            use skat_aug23::skat::signature::HandSignature;

            while found < count {
                attempts += 1;
                vec.shuffle(&mut rng);

                let mut my_hand = 0u32;
                for el in &vec[0..10] {
                    my_hand |= 1u32 << *el;
                }

                let mut skat = 0u32;
                for el in &vec[10..12] {
                    skat |= 1u32 << *el;
                }

                // Check signature: 1 Jack, 2 Aces
                let sig = HandSignature::from_hand(my_hand);

                // Count Jacks
                let jacks_count = sig.jacks.count_ones();

                if jacks_count == 1 && sig.aces == 2 {
                    // Potential candidate, run analysis
                    let (_, prob, _) = analyze_hand_with_pickup(my_hand, skat, 20, false); // Fast check

                    if prob >= min_win {
                        // Double check with more samples
                        let (_, prob_refined, _) =
                            analyze_hand_with_pickup(my_hand, skat, 100, false);

                        if prob_refined >= min_win {
                            found += 1;
                            println!(
                                "Found scenario #{}: Win {:.1}% (Attempts: {})",
                                found,
                                prob_refined * 100.0,
                                attempts
                            );

                            // Reconstruct full context
                            let mut left = 0u32;
                            for el in &vec[12..22] {
                                left |= 1u32 << *el;
                            }
                            let mut right = 0u32;
                            for el in &vec[22..32] {
                                right |= 1u32 << *el;
                            }

                            // Create PimcContextInput (compatible with web/standard playout)
                            use skat_aug23::traits::StringConverter;

                            let context = args::GameContextInput {
                                declarer_cards: my_hand.__str(),
                                left_cards: left.__str(),
                                right_cards: right.__str(),
                                game_type: skat_aug23::skat::defs::Game::Grand,
                                start_player: skat_aug23::skat::defs::Player::Declarer,
                                mode: Some(args::SearchMode::Win),
                                trick_cards: None,
                                trick_suit: None,
                                declarer_start_points: None,
                                samples: Some(100),
                                god_players: None,
                            };

                            let filename = format!(
                                "{}/scenario_{}_win_{:.0}.json",
                                output_dir,
                                found,
                                prob_refined * 100.0
                            );
                            let json = serde_json::to_string_pretty(&context).unwrap();
                            fs::write(&filename, json).expect("Unable to write file");

                            // Also write a little .txt info file about the Skat
                            let skat_str = skat.__str();
                            fs::write(
                                format!("{}/scenario_{}_info.txt", output_dir, found),
                                format!("Skat Cards: {}\nWin Prob: {:.2}", skat_str, prob_refined),
                            )
                            .expect("Unable to write info file");
                        }
                    }
                }
            }
        }
        args::Commands::AnalyzeSuit {
            count,
            samples,
            output,
            hand,
            post_discard,
        } => {
            // Assume Clubs (0) for now as requested.
            // Future improvement: allow specifying suit.
            let suit_id = 0; // Clubs
            println!(
                "Analyzing Suit (Clubs) (Count: {}, Samples: {}, Hand: {}, Post-Discard: {})...",
                count, samples, hand, post_discard
            );

            use std::time::Instant;
            let start_time = Instant::now();

            use std::io::Write;
            let mut writer: Box<dyn Write> = if let Some(path) = output {
                let file = std::fs::File::create(path).expect("Could not create output file");
                Box::new(file)
            } else {
                Box::new(std::io::stdout())
            };

            // Header
            // Header
            writeln!(
                writer,
                "{},PlayedCards,DiscardedCards,PostJacksMask,PostTrumpCount,PostAces,PostTens,PostAttachedTens,PostTenKingSmall,PostSkatFulls",
                HandSignature::to_csv_header()
            )
            .unwrap();

            let mut found = 0;
            while found < count {
                // Generate random hand
                let mut my_hand_val: u32 = 0;
                let mut skat_val: u32 = 0;
                {
                    let mut rng = rand::thread_rng();
                    let mut all_cards_lists = (0..32).collect::<Vec<u8>>();
                    all_cards_lists.shuffle(&mut rng);

                    for i in 0..10 {
                        my_hand_val |= 1 << all_cards_lists[i];
                    }
                    // Skat for pickup
                    skat_val |= 1 << all_cards_lists[10];
                    skat_val |= 1 << all_cards_lists[11];
                }

                if hand {
                    // Hand game (no pickup) - behavior unchanged, Post cols will be 0/Empty
                    let (sig, prob) =
                        skat_aug23::pimc::analysis::analyze_suit(my_hand_val, suit_id, samples);

                    use skat_aug23::traits::StringConverter;
                    writeln!(
                        writer,
                        "{},{},{},-,-,-,-,-,-,-",
                        sig.to_csv_row(my_hand_val, skat_val, prob),
                        my_hand_val.__str(),
                        ""
                    )
                    .unwrap();
                } else {
                    let (pre_sig, post_sig, prob, played, discarded) =
                        skat_aug23::pimc::analysis::analyze_suit_with_pickup(
                            my_hand_val,
                            skat_val,
                            suit_id,
                            samples,
                            post_discard,
                        );

                    use skat_aug23::traits::StringConverter;
                    let played_str = played.__str();
                    let discarded_str = discarded.__str();

                    // Pre-Sig row + played cards + Post-Sig details

                    // We extract Post Sig details manually or via helper?
                    // HandSignature doesn't have a "to_csv_partial" method, it has "to_csv_row" which includes Cards and WinProb.
                    // We only want the fields: JacksMask, TrumpCount, Aces, Tens, AttachedTens, TenKingSmall, SkatFulls

                    let post_row = format!(
                        "{},{},{},{},{},{},{}",
                        post_sig.jacks_string(),
                        post_sig.trump_count,
                        post_sig.aces,
                        post_sig.tens,
                        post_sig.attached_tens,
                        post_sig.ten_king_small,
                        post_sig.skat_fulls
                    );

                    writeln!(
                        writer,
                        "{},{},{},{}",
                        pre_sig.to_csv_row(my_hand_val, skat_val, prob),
                        played_str,
                        discarded_str,
                        post_row
                    )
                    .unwrap();
                }

                found += 1;
                // Progress update
                if found % 10 == 0 || found == count {
                    let elapsed = start_time.elapsed();
                    let completed = found;
                    let rate = completed as f64 / elapsed.as_secs_f64(); // items per second
                    let remaining_items = count - completed;
                    let eta_secs = if rate > 0.0 {
                        remaining_items as f64 / rate
                    } else {
                        0.0
                    };

                    let elapsed_str = format!(
                        "{:02}:{:02}:{:02}",
                        elapsed.as_secs() / 3600,
                        (elapsed.as_secs() % 3600) / 60,
                        elapsed.as_secs() % 60
                    );

                    let eta_str = format!(
                        "{:02}:{:02}:{:02}",
                        (eta_secs as u64) / 3600,
                        ((eta_secs as u64) % 3600) / 60,
                        (eta_secs as u64) % 60
                    );

                    let percent = (completed as f64 / count as f64) * 100.0;

                    print!("\rProgress: {:3.1}% | {}/{} | Elapsed: {} | ETA: {} | Rate: {:.2} games/s   ", 
                        percent, completed, count, elapsed_str, eta_str, rate);
                    std::io::stdout().flush().unwrap();
                }
            }
            println!(" Analysis Complete");
        }
        args::Commands::AnalyzeGeneralHand {
            count,
            samples,
            output,
        } => {
            use skat_aug23::pimc::analysis::analyze_general_hand;

            println!(
                "Running General Hand Game Analysis (Best Game) with {} hands, {} samples...",
                count, samples
            );

            let mut file = std::fs::File::create(&output).expect("Could not create CSV file");
            // Header for Hand Analysis
            // Similar to GeneralPreDiscard but specific columns
            // InitHand, InitSkat (0/Unknown), BestGameName, WinProb, [Probs...], Duration
            // And Signature columns (JacksMask, Aces, Ten, etc.)

            writeln!(
                file,
                "{:<35}, {:<10}, {:>15}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>10}, {:>8}, {:>10}, {:>10}, {:>10}, {:>11}, {:>13}, {:>12}, {:<8}, {:>12}",
                "Hand", "Skat", "JacksMask", "CntJ", "Aces", "Tens", "Att10", "MxLen", "TKS", "BestGame", "WinProb", "ProbGrand", "ProbClubs", "ProbSpades", "ProbHearts", "ProbDiamonds", "ProbNull", "WonMask", "DurationMs"
            )
            .unwrap();

            use std::sync::{Arc, Mutex};
            let file_mutex = Arc::new(Mutex::new(file));

            analyze_general_hand(
                count,
                samples,
                |(
                    hand_val,
                    _skat,
                    _discard,
                    sig,
                    probs,
                    prob_null,
                    best_variant,
                    duration_micros,
                )| {
                    // hand_val is u32
                    // Skat is unknown (or true skat if we passed it), but for Hand game analysis implies unknown.
                    // We print "-" for Skat to indicate Hand game context or printing 0?
                    // Let's print "Unknown" or just empty.
                    let skat_str = "-";

                    let best_game_name = match best_variant {
                        0 => "Grand",
                        1 => "Clubs",
                        2 => "Spades",
                        3 => "Hearts",
                        4 => "Diamonds",
                        5 => "Null",
                        _ => "Unknown",
                    };

                    use skat_aug23::skat::formatter::format_hand_for_game;
                    let hand_str = format_hand_for_game(hand_val, best_game_name);

                    let final_max_prob = if best_variant == 5 {
                        prob_null
                    } else {
                        probs[best_variant as usize]
                    };
                    let jacks_str = sig.jacks_string();

                    // WonMask Logic (> 0.66)
                    let mut won_mask = String::with_capacity(6);
                    won_mask.push(if probs[0] > 0.66 { 'G' } else { '-' }); // Grand
                    won_mask.push(if probs[1] > 0.66 { 'C' } else { '-' }); // Clubs
                    won_mask.push(if probs[2] > 0.66 { 'S' } else { '-' }); // Spades
                    won_mask.push(if probs[3] > 0.66 { 'H' } else { '-' }); // Hearts
                    won_mask.push(if probs[4] > 0.66 { 'D' } else { '-' }); // Diamonds
                    won_mask.push(if prob_null > 0.66 { 'N' } else { '-' }); // Null

                    let duration_ms = duration_micros as f64 / 1000.0;

                    let row_str = format!(
                        "{:<35}, {:<10}, {:>15}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>5}, {:>10}, {:>8.4}, {:>10.4}, {:>10.4}, {:>10.4}, {:>11.4}, {:>13.4}, {:>12.4}, {:<8}, {:>12.2}",
                        hand_str,
                        skat_str,
                        jacks_str,
                        sig.trump_count, // Is this NumJacks or TrumpCount? For Grand it's Jacks.
                        // Wait, sig is "from_hand_and_skat_suit(..., 0, None)". 
                        // So trump_count is Jacks count.
                        // max_suit_len is max length of side suits? 
                        // Actually max_suit_len is useful.
                        sig.aces,
                        sig.tens,
                        sig.attached_tens,
                        sig.max_suit_len,
                        sig.ten_king_small,
                        best_game_name,
                        final_max_prob,
                        probs[0], probs[1], probs[2], probs[3], probs[4], prob_null,
                        won_mask,
                        duration_ms
                    );

                    if let Ok(mut f) = file_mutex.lock() {
                        writeln!(f, "{}", row_str).unwrap();
                    }
                },
            );
            println!("Analysis complete. Results written to {}", output);
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
        args::Commands::Playout {
            game_type,
            start_player,
            context,
            samples,
        } => {
            if let Some(ctx_path) = context {
                println!("Reading context file: {}", ctx_path);
                let context_content =
                    fs::read_to_string(ctx_path).expect("Unable to read context file");
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

                game_context.set_threshold_upper(120); // Playout uses value not win check, but good to set.

                if let Err(e) = game_context.validate() {
                    eprintln!("Validation Error: {}", e);
                    std::process::exit(1);
                }

                println!("Calling skat_aug23::extensions::cli_playout::run_playout...");
                skat_aug23::extensions::cli_playout::run_playout(
                    game_context,
                    input.game_type,
                    input.start_player,
                    samples,
                );
            } else {
                println!("No context file provided. Generating random deal...");
                let (game_context, g, p) =
                    skat_aug23::extensions::cli_playout::generate_random_deal(
                        game_type,
                        start_player,
                    );

                skat_aug23::extensions::cli_playout::run_playout(game_context, g, p, samples);
            }
        }
        args::Commands::PointsPlayout {
            game_type,
            start_player,
            context,
            samples,
        } => {
            if let Some(ctx_path) = context {
                println!("Reading context file: {}", ctx_path);
                let context_content =
                    fs::read_to_string(ctx_path).expect("Unable to read context file");
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

                skat_aug23::extensions::cli_playout::run_points_playout(
                    game_context,
                    input.game_type,
                    input.start_player,
                    samples,
                );
            } else {
                println!("No context file provided. Generating random deal...");
                let (game_context, g, p) =
                    skat_aug23::extensions::cli_playout::generate_random_deal(
                        game_type,
                        start_player,
                    );

                skat_aug23::extensions::cli_playout::run_points_playout(
                    game_context,
                    g,
                    p,
                    samples,
                );
            }
        }
        args::Commands::SmartPointsPlayout { samples } => {
            use skat_aug23::extensions::cli_playout::generate_smart_deal;
            match generate_smart_deal() {
                None => {
                    // Deal did not qualify (best game < 50 pts or no Grand/Suit).
                    // Print a short marker so the Python loop can detect the skip.
                    println!("SMART_DEAL_SKIP: deal did not qualify.");
                    std::process::exit(2);
                }
                Some((ctx, game_type, start_player, label, discard)) => {
                    println!("SMART_DEAL_OK: game={} discard={}", label, discard);
                    skat_aug23::extensions::cli_playout::run_points_playout(
                        ctx,
                        game_type,
                        start_player,
                        samples,
                    );
                }
            }
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

            let results_info = skat_aug23::extensions::skat_solving::solve_best_game_all_variants(
                declarer_cards,
                left_cards,
                right_cards,
                start_player,
                acc_mode,
            );

            // Convert to tuple format for existing printing logic (temporary adapter)
            // Or better, update printing logic. Let's update printing logic below.
            let mut results: Vec<_> = results_info
                .iter()
                .map(|info| {
                    (
                        info.label.clone(),
                        info.skat_1,
                        info.skat_2,
                        info.value,
                        info.game_type,
                    )
                })
                .collect();

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
