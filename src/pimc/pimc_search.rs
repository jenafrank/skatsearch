use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

use super::pimc_problem::PimcProblem;
use crate::extensions::solver::{solve_all_cards, solve_win};
use crate::skat::defs::Game;
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
        for i in 0..self.sample_size {
            if self.verbose_progress {
                print!(".");
                std::io::stdout().flush().unwrap();
            }

            let concrete_problem = self.uproblem.generate_concrete_problem();
            let mut solver = SkatEngine::new(concrete_problem, None);
            let search_result = solve_win(&mut solver);

            if info {
                println!("Game {}", i);
                println!(
                    "Declarer cards: {}",
                    solver.context.declarer_cards().__str()
                );
                println!("Left cards    : {}", solver.context.left_cards().__str());
                println!("Right cards   : {}", solver.context.right_cards().__str());
                println!(
                    "Best card: {} Win: {}",
                    search_result.best_card.__str(),
                    search_result.declarer_wins
                );
                println!();
            }

            sum += search_result.declarer_wins as u32 as f32;

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

                writeln!(
                    file,
                    "Result   : {}",
                    if search_result.declarer_wins {
                        "Win"
                    } else {
                        "Loss"
                    }
                )
                .unwrap();
                writeln!(file, "--------------------------------------------------").unwrap();
            }
        }
        (sum / self.sample_size as f32, sum as u32)
    }

    pub fn estimate_probability_of_all_cards(&self, info: bool) -> Vec<(u32, f32)> {
        let mut global_dict = HashMap::new();

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

                if self.uproblem.game_type() == Game::Null {
                    if value == 0 {
                        winvalue = 1;
                    }
                } else {
                    if value >= self.uproblem.threshold() {
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
}
