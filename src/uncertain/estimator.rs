use std::collections::HashMap;

use crate::{types::{solver::Solver, game::Game}, traits::StringConverter};
use super::uncertain_problem::UncertainProblem;

pub struct Estimator {
    pub uproblem: UncertainProblem,
    pub sample_size: u32 
}

impl Estimator {

    pub fn new(uproblem: UncertainProblem, sample_size: u32) -> Estimator {
        Estimator { uproblem, sample_size }
    }

    pub fn estimate_win(&self, info: bool) -> (f32, u32) {
        let mut sum: f32 = 0.0;
        for i in 0..self.sample_size {
            let concrete_problem = self.uproblem.generate_concrete_problem();                        
            let mut solver = Solver::new(concrete_problem);
            let search_result = solver.solve_win();

            if info {
                println!("Game {}", i);
                println!("Declarer cards: {}", solver.problem.declarer_cards().__str());
                println!("Left cards    : {}", solver.problem.left_cards().__str());
                println!("Right cards   : {}", solver.problem.right_cards().__str());
                println!("Best card: {} Win: {}", search_result.best_card.__str(), search_result.declarer_wins);
                println!();
            }
            
            sum += search_result.declarer_wins as u32 as f32;
        }
        (sum / self.sample_size as f32, sum as u32)
    }

    pub fn estimate_probability_of_all_cards(&self, info: bool) -> Vec<(u32, f32)> {
        let mut global_dict = HashMap::new(); 
        
        for i in 0..self.sample_size {
            let concrete_problem = self.uproblem.generate_concrete_problem();                        
            let mut solver = Solver::new(concrete_problem);
            let x = self.uproblem.threshold_upper();
            let search_result = solver.solve_all_cards(x-1, x);
            let mut local_dict = HashMap::new(); 

            for line in search_result.card_list.iter() {
                let card = line.investigated_card;
                let value = line.value;
                let mut winvalue:u8 = 0;

                if self.uproblem.game_type() == Game::Null {
                    if value == 0 {
                        winvalue = 1;
                    }
                } else {
                    if value >= self.uproblem.threshold_upper() {
                        winvalue = 1;
                    }
                }

                local_dict.insert(card, winvalue);

                let entry = global_dict.entry(card).or_insert(0);
                *entry += winvalue;
            }

            if info {
                println!("Game {}", i);
                println!("Declarer cards: {}", solver.problem.declarer_cards().__str());
                println!("Left cards    : {}", solver.problem.left_cards().__str());
                println!("Right cards   : {}", solver.problem.right_cards().__str());
                println!("All moves: {}", local_dict.iter().map(|(k, v)| format!("{}: {}", k.__str(), v)).collect::<Vec<_>>().join(", "));
                println!();
            }
        }

        let mut ret_dict = HashMap::new();

        for (k, v) in global_dict.iter() {
            let entry = ret_dict.entry(*k).or_insert(0.0);
            *entry = *v as f32 / self.sample_size as f32;
        }

        let mut sorted_entries = ret_dict.iter().collect::<Vec<_>>();
        sorted_entries.sort_by(|a,b| b.1.partial_cmp(a.1).unwrap());

        let mut ret: Vec<(u32, f32)> = Vec::new();

        for entry in sorted_entries {
            ret.push((*entry.0, *entry.1));
        }

        ret
    }

}

#[cfg(test)]
mod tests {
    use crate::{types::player::Player, uncertain::uproblem_builder::UProblemBuilder};
    
    #[test]
    fn test_uproblem_three_cards() {
        let uproblem = UProblemBuilder::new_farbspiel()
        .cards(Player::Declarer, "SA SK S9")
        .remaining_cards("ST SQ S8 C7 H7 D7")
        .threshold(21)
        .build();
    
        let estimator = super::Estimator::new(uproblem, 10);
        let (probability, _) = estimator.estimate_win(false);

        println!("Probability of win: {}", probability);
    }
    
    #[test]
    fn test_uproblem_three_cards_with_debug_info() {
        
        let uproblem = UProblemBuilder::new_farbspiel() 
        .cards(Player::Declarer, "SA SK S9")
        .remaining_cards("ST SQ S8 C7 H7 D7")
        .threshold(21)
        .build();
  
        let estimator = super::Estimator::new(uproblem, 1000);
        let (probability, _) = estimator.estimate_win(true);

        println!("Probability of win: {}", probability);
    }
}
