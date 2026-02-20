use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

use rayon::prelude::*;

use super::pimc_problem::PimcProblem;
use crate::extensions::solver::{solve_all_cards, solve_win};
use crate::skat::defs::Game;
use crate::skat::defs::Player;
use crate::skat::engine::SkatEngine;
use crate::traits::StringConverter;

pub struct PimcSearch {
    pub uproblem: PimcProblem,
    pub sample_size: u32,
    pub log_file: Option<String>,
    pub verbose_progress: bool,
}

impl PimcSearch {
    pub fn new(uproblem: PimcProblem, sample_size: u32, log_file: Option<String>) -> PimcSearch {
        PimcSearch {
            uproblem,
            sample_size,
            log_file,
            verbose_progress: false,
        }
    }

    pub fn estimate_win(&self, info: bool) -> (f32, u32) {
        let mut sum: f32 = 0.0;
        let my_player = self.uproblem.my_player();

        for i in 0..self.sample_size {
            if self.verbose_progress {
                print!(".");
                std::io::stdout().flush().unwrap();
            }

            let concrete_problem = self.uproblem.generate_concrete_problem();
            let mut solver = SkatEngine::new(concrete_problem, None);
            let search_result = solve_win(&mut solver);

            let declarer_wins = search_result.declarer_wins;
            let i_win = if my_player == Player::Declarer {
                declarer_wins
            } else {
                !declarer_wins
            };

            if info {
                println!("Game {}", i);
                println!(
                    "Declarer cards: {}",
                    solver.context.declarer_cards().__str()
                );
                println!("Left cards    : {}", solver.context.left_cards().__str());
                println!("Right cards   : {}", solver.context.right_cards().__str());
                println!(
                    "Best card: {} DeclWin: {} MyWin: {}",
                    search_result.best_card.__str(),
                    declarer_wins,
                    i_win
                );
                println!();
            }

            if i_win {
                sum += 1.0;
            }

            if let Some(path) = &self.log_file {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();

                writeln!(file, "Sample {}:", i).unwrap();
                writeln!(file, "Game Type: {:?}", solver.context.game_type()).unwrap();
                writeln!(
                    file,
                    "Declarer : {}",
                    solver.context.declarer_cards().__str()
                )
                .unwrap();
                writeln!(file, "Left     : {}", solver.context.left_cards().__str()).unwrap();
                writeln!(file, "Right    : {}", solver.context.right_cards().__str()).unwrap();

                let skat = crate::skat::defs::ALLCARDS
                    ^ solver.context.declarer_cards()
                    ^ solver.context.left_cards()
                    ^ solver.context.right_cards()
                    ^ solver.context.trick_cards();

                writeln!(file, "Skat     : {}", skat.__str()).unwrap();

                writeln!(file, "Result   : {}", if i_win { "Win" } else { "Loss" }).unwrap();
                writeln!(file, "--------------------------------------------------").unwrap();
            }
        }
        (sum / self.sample_size as f32, sum as u32)
    }

    pub fn estimate_probability_of_all_cards(&self, info: bool) -> Vec<(u32, f32)> {
        let my_player = self.uproblem.my_player();
        let threshold = self.uproblem.threshold();
        let game_type = self.uproblem.game_type();

        // Shared accumulator: card → total win-count across all samples.
        let global_dict: Mutex<HashMap<u32, u32>> = Mutex::new(HashMap::new());

        (0..self.sample_size).into_par_iter().for_each(|i| {
            let concrete_problem = self.uproblem.generate_concrete_problem();
            let mut solver = SkatEngine::new(concrete_problem, None);
            let search_result = solve_all_cards(&mut solver, threshold - 1, threshold);

            let mut local_dict: HashMap<u32, u8> = HashMap::new();
            for line in search_result.results.iter() {
                let card = line.0;
                let value = line.2;

                let declarer_wins = if game_type == Game::Null {
                    value == 0
                } else {
                    value >= threshold
                };
                let winvalue: u8 = if my_player == Player::Declarer {
                    if declarer_wins {
                        1
                    } else {
                        0
                    }
                } else {
                    if !declarer_wins {
                        1
                    } else {
                        0
                    }
                };
                local_dict.insert(card, winvalue);
            }

            // Merge into global accumulator.
            {
                let mut global = global_dict.lock().unwrap();
                for (&card, &wv) in local_dict.iter() {
                    *global.entry(card).or_insert(0) += wv as u32;
                }
            }

            // Optional per-sample log file (serialised via re-opening the file).
            if let Some(path) = &self.log_file {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();
                writeln!(file, "Sample {}:", i).unwrap();
                writeln!(
                    file,
                    "Declarer: {}",
                    solver.context.declarer_cards().__str()
                )
                .unwrap();
                writeln!(file, "Left    : {}", solver.context.left_cards().__str()).unwrap();
                writeln!(file, "Right   : {}", solver.context.right_cards().__str()).unwrap();
                writeln!(file, "--------------------------------------------------").unwrap();
            }

            if info {
                println!(
                    "Sample {}: Declarer={} Left={} Right={}",
                    i,
                    solver.context.declarer_cards().__str(),
                    solver.context.left_cards().__str(),
                    solver.context.right_cards().__str(),
                );
            }
        });

        let global = global_dict.into_inner().unwrap();
        let mut sorted: Vec<(u32, f32)> = global
            .iter()
            .map(|(&card, &wins)| (card, wins as f32 / self.sample_size as f32))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted
    }

    /// Like `estimate_probability_of_all_cards` but returns the **average declarer point value**
    /// (0–120) instead of a binary win/loss probability.
    ///
    /// From the perspective of the current player:
    /// - Declarer: higher average score = better.
    /// - Defender: score is inverted to `120 - decl_points`, so higher still means better.
    /// - Null games: declarer_wins (value==0) → score 120; else 0.
    ///
    /// Returns `Vec<(card, avg_score)>` sorted descending by avg_score.
    pub fn estimate_avg_points_of_all_cards(&self, info: bool) -> Vec<(u32, f32)> {
        let my_player = self.uproblem.my_player();
        let game_type = self.uproblem.game_type();

        // Shared accumulator: card → (sum_of_scores, count).
        let global: Mutex<HashMap<u32, (f32, u32)>> = Mutex::new(HashMap::new());

        (0..self.sample_size).into_par_iter().for_each(|i| {
            let concrete_problem = self.uproblem.generate_concrete_problem();
            let mut solver = SkatEngine::new(concrete_problem, None);
            // Full-range solve to get exact point values (0–120).
            let search_result = solve_all_cards(&mut solver, 0, 120);

            let mut local: Vec<(u32, f32)> = Vec::new();
            for line in search_result.results.iter() {
                let card = line.0;
                let decl_points = line.2; // u8, 0–120

                let player_score: f32 = if game_type == Game::Null {
                    if decl_points == 0 {
                        if my_player == Player::Declarer {
                            120.0
                        } else {
                            0.0
                        }
                    } else {
                        if my_player == Player::Declarer {
                            0.0
                        } else {
                            120.0
                        }
                    }
                } else if my_player == Player::Declarer {
                    decl_points as f32
                } else {
                    120.0 - decl_points as f32
                };
                local.push((card, player_score));
            }

            // Merge into global accumulator.
            {
                let mut g = global.lock().unwrap();
                for (card, score) in local {
                    let entry = g.entry(card).or_insert((0.0, 0));
                    entry.0 += score;
                    entry.1 += 1;
                }
            }

            if info {
                println!(
                    "Sample {}: Declarer={} Left={} Right={}",
                    i,
                    solver.context.declarer_cards().__str(),
                    solver.context.left_cards().__str(),
                    solver.context.right_cards().__str(),
                );
            }
        });

        // Compute averages and sort descending.
        let g = global.into_inner().unwrap();
        let mut entries: Vec<(u32, f32)> = g
            .iter()
            .map(|(&card, &(sum, cnt))| (card, sum / cnt.max(1) as f32))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries
    }
}

#[cfg(test)]
mod tests {
    use crate::{pimc::pimc_problem_builder::PimcProblemBuilder, skat::defs::Player};

    #[test]
    fn test_uproblem_three_cards() {
        let uproblem = PimcProblemBuilder::new_farbspiel()
            .cards(Player::Declarer, "SA SK S9")
            .remaining_cards("ST SQ S8 C7 H7 D7")
            .threshold(21)
            .build();

        let estimator = super::PimcSearch::new(uproblem, 10, None);
        let (probability, _) = estimator.estimate_win(false);

        println!("Probability of win: {}", probability);
    }

    #[test]
    fn test_uproblem_three_cards_with_debug_info() {
        let uproblem = PimcProblemBuilder::new_farbspiel()
            .cards(Player::Declarer, "SA SK S9")
            .remaining_cards("ST SQ S8 C7 H7 D7")
            .threshold(21)
            .build();

        let estimator = super::PimcSearch::new(uproblem, 1000, None);
        let (probability, _) = estimator.estimate_win(true);

        println!("Probability of win: {}", probability);
    }

    #[test]
    fn test_pimc_start_points() {
        // Scenario: Declarer has 50 points. Remaining cards allow max 15 points.
        // Threshold: 61.
        // Declarer (Me) has CA(11). Ops have C7(0), C8(0).
        // 3 cards total.

        // Declarer CA. Left C7. Right C8.
        // Declarer leads? Or Left leads?
        // Let's sets `turn`. Defaults to `my_player`.
        // Declarer plays CA. Left C7. Right C8. Declarer wins 11.
        // Total 50+11 = 61. WIN.

        let uproblem = PimcProblemBuilder::new_farbspiel() // Clubs
            .cards(Player::Declarer, "CA")
            .remaining_cards("C7 C8")
            .threshold(61)
            .declarer_start_points(50)
            .build();

        let search = super::PimcSearch::new(uproblem, 20, None);
        let (prob, _) = search.estimate_win(false);

        println!("Prob with start_points=50: {}", prob);
        assert!(prob > 0.9, "Should win with start points");
    }

    #[test]
    fn test_defender_perspective() {
        // Scenario: Defender (Left) vs Declarer (Right).
        // Declarer needs 11 pts to win (has 50).
        // Left (Me) has SA (11). Right has S7 (0). Partner (Decl: None? Player::Right is Partner?).
        // Skat/3-player game:
        // Left (Me): SA (11)
        // Right (Partner): S7 (0)
        // Declarer (Player::Declarer): S8 (0).
        // 3 cards.

        // Left leads.
        // Left plays SA.
        // Right (Partner) S7. Declarer S8.
        // SA wins (11).
        // Declarer gets 0. Total 50. Loss.
        // Left (Defender) WINS.

        // So PIMC Win Prob for Left should be 1.0.

        let uproblem = PimcProblemBuilder::new_grand()
            .my_player(Player::Left)
            .cards(Player::Left, "SA")
            .declarer_start_points(50)
            .threshold(61)
            .remaining_cards("S7 S8")
            .build();

        let search = super::PimcSearch::new(uproblem, 20, None);

        // estimate_win now returns "My Win Prob".
        let (prob, _) = search.estimate_win(false);

        println!("Defender Win Prob (My Win): {}", prob);

        // Should be high (I win).
        assert!(
            prob > 0.9,
            "Defender should have high win prob (Declarer leads to loss)"
        );
    }
}
