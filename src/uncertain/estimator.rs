use crate::{types::solver::Solver, traits::StringConverter};
use super::uncertain_problem::UncertainProblem;

pub struct Estimator {
    pub uproblem: UncertainProblem,
    pub sample_size: u32 
}

impl Estimator {

    pub fn new(uproblem: UncertainProblem, sample_size: u32) -> Estimator {
        Estimator { uproblem, sample_size }
    }

    pub fn estimate_win(&self, info: bool) -> f32 {
        let mut sum: f32 = 0.0;
        for i in 0..self.sample_size {
            let concrete_problem = self.uproblem.generate_concrete_problem();                        
            let mut solver = Solver::new(concrete_problem);
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
        sum / self.sample_size as f32
    }

}

#[cfg(test)]
mod tests {
    use crate::{uncertain::uncertain_problem::{UncertainProblem, Facts}, types::{player::Player, game::Game}, traits::BitConverter};

    
    #[test]
    fn test_uproblem_1() {
        
        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            player: Player::Declarer,
            my_cards: "SA SK S9".__bit(),
            cards_on_table: 0,
            all_cards: "SA SK S9 ST SQ S8 C7 H7 D7".__bit(),
            active_suit: 0,
            points_to_win: 21,
            facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
        };

        let estimator = super::Estimator::new(uproblem, 10);
        let probability = estimator.estimate_win(false);

        println!("Probability of win: {}", probability);
    }

    #[test]
    fn test_uproblem_full() {
        
        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            player: Player::Declarer,
            my_cards: "SA SK S9".__bit(),
            cards_on_table: 0,
            all_cards: "SA SK S9 ST SQ S8 C7 H7 D7".__bit(),
            active_suit: 0,
            points_to_win: 21,
            facts: [Facts::zero_fact(Player::Left), Facts::zero_fact(Player::Right)]
        };

        let estimator = super::Estimator::new(uproblem, 10);
        let probability = estimator.estimate_win(false);

        println!("Probability of win: {}", probability);
    }
}