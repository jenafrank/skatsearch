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
            let solver = Solver::create(concrete_problem);
            let search_result = solver.solve_win();

            if info {
                println!("Game {}", i);
                println!("Declarer cards: {}", solver.problem.declarer_cards_all.__str());
                println!("Left cards    : {}", solver.problem.left_cards_all.__str());
                println!("Right cards   : {}", solver.problem.right_cards_all.__str());
                println!("Best card: {} Win: {}", search_result.best_card.__str(), search_result.declarer_wins);
                println!();
            }
            
            sum += search_result.declarer_wins as u32 as f32;
        }
        (sum / self.sample_size as f32, sum as u32)
    }

    pub fn estimate_probability_of_all_cards(&self, info: bool) -> HashMap<u32, f32> {
        let mut global_dict = HashMap::new(); 
        
        for i in 0..self.sample_size {
            let concrete_problem = self.uproblem.generate_concrete_problem();                        
            let solver = Solver::create(concrete_problem);
            let x = self.uproblem.upper_bound_of_null_window;
            let search_result = solver.solve_all_cards(x-1, x);
            let mut local_dict = HashMap::new(); 

            for line in search_result.card_list.iter() {
                let card = line.investigated_card;
                let value = line.value;
                let mut winvalue:u8 = 0;

                if self.uproblem.game_type == Game::Null {
                    if value == 0 {
                        winvalue = 1;
                    }
                } else {
                    if value >= self.uproblem.upper_bound_of_null_window {
                        winvalue = 1;
                    }
                }

                local_dict.insert(card, winvalue);

                let entry = global_dict.entry(card).or_insert(0);
                *entry += winvalue;
            }

            if info {
                println!("Game {}", i);
                println!("Declarer cards: {}", solver.problem.declarer_cards_all.__str());
                println!("Left cards    : {}", solver.problem.left_cards_all.__str());
                println!("Right cards   : {}", solver.problem.right_cards_all.__str());
                println!("All moves: {}", local_dict.iter().map(|(k, v)| format!("{}: {}", k.__str(), v)).collect::<Vec<_>>().join(", "));
                println!();
            }
        }

        let mut ret_dict = HashMap::new();

        for (k, v) in global_dict.iter() {
            let entry = ret_dict.entry(*k).or_insert(0.0);
            *entry = *v as f32 / self.sample_size as f32;
        }

        ret_dict
    }

}

#[cfg(test)]
mod tests {
    use crate::{uncertain::uncertain_problem::{UncertainProblem, Facts}, types::{player::Player, game::Game}, traits::BitConverter};

    
    #[test]
    fn test_uproblem_1() {
        
        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            my_cards: "SA SK S9".__bit(),
            cards_on_table: 0,
            all_cards: "SA SK S9 ST SQ S8 C7 H7 D7".__bit(),
            active_suit: 0,
            upper_bound_of_null_window: 21,
            facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
        };

        let estimator = super::Estimator::new(uproblem, 10);
        let (probability, _) = estimator.estimate_win(false);

        println!("Probability of win: {}", probability);
    }
    
    #[test]
    fn test_uproblem_full() {
        
        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            my_cards: "SA SK S9".__bit(),
            cards_on_table: 0,
            all_cards: "SA SK S9 ST SQ S8 C7 H7 D7".__bit(), 
            active_suit: 0,
            upper_bound_of_null_window: 21,
            facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
        };

        let estimator = super::Estimator::new(uproblem, 1000);
        let (probability, _) = estimator.estimate_win(true);

        println!("Probability of win: {}", probability);
    }
}