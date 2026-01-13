use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

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
        let mut global_dict = HashMap::new();
        let my_player = self.uproblem.my_player();

        for i in 0..self.sample_size {
            let concrete_problem = self.uproblem.generate_concrete_problem();
            let mut solver = SkatEngine::new(concrete_problem, None);
            let x = self.uproblem.threshold();
            let search_result = solve_all_cards(&mut solver, x - 1, x);
            let mut local_dict = HashMap::new();

            for line in search_result.results.iter() {
                let card = line.0;
                let value = line.2;
                let mut winvalue: u8 = 0;

                let mut declarer_wins = false;
                if self.uproblem.game_type() == Game::Null {
                    if value == 0 {
                        declarer_wins = true;
                    }
                } else {
                    if value >= self.uproblem.threshold() {
                        declarer_wins = true;
                    }
                }

                if my_player == Player::Declarer {
                    if declarer_wins {
                        winvalue = 1;
                    }
                } else {
                    // Defender wins if Declarer Loses
                    if !declarer_wins {
                        winvalue = 1;
                    }
                }

                local_dict.insert(card, winvalue);

                let entry = global_dict.entry(card).or_insert(0);
                *entry += winvalue;
            }

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
                let mut wins: Vec<String> = local_dict
                    .iter()
                    .filter(|(_, &v)| v > 0)
                    .map(|(k, _)| k.__str())
                    .collect();
                let mut losses: Vec<String> = local_dict
                    .iter()
                    .filter(|(_, &v)| v == 0)
                    .map(|(k, _)| k.__str())
                    .collect();

                // Sort for consistent output
                wins.sort();
                losses.sort();

                writeln!(
                    file,
                    "Moves   : WINS: {} | LOSS: {}",
                    wins.join(" "),
                    losses.join(" ")
                )
                .unwrap();
                writeln!(file, "--------------------------------------------------").unwrap();
            }

            if info {
                println!("Game {}", i);
                println!(
                    "Declarer cards: {}",
                    solver.context.declarer_cards().__str()
                );
                println!("Left cards    : {}", solver.context.left_cards().__str());
                println!("Right cards   : {}", solver.context.right_cards().__str());
                println!(
                    "All moves: {}",
                    local_dict
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k.__str(), v))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!();
            }
        }

        let mut ret_dict = HashMap::new();

        for (k, v) in global_dict.iter() {
            let entry = ret_dict.entry(*k).or_insert(0.0);
            *entry = *v as f32 / self.sample_size as f32;
        }

        let mut sorted_entries = ret_dict.iter().collect::<Vec<_>>();
        sorted_entries.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        let mut ret: Vec<(u32, f32)> = Vec::new();

        for entry in sorted_entries {
            ret.push((*entry.0, *entry.1));
        }

        ret
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
