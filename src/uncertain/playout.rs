use crate::traits::StringConverter;
use crate::uncertain::uncertain_problem::UncertainProblem;
use super::estimator::Estimator;

impl Estimator {

    pub fn playout(initial_problem: UncertainProblem)
    {
        let first_uproblem = initial_problem;
        let estimator = Estimator::new(first_uproblem, 100);

        println!("Start calculating 1...");
        let result = estimator.estimate_probability_of_all_cards(false);

        let best_card = result[0].0.__str();
        let best_card_probability = result[0].1;

        println!("Playing {} ({}) ...",best_card, best_card_probability);
        
        // advance situation
        
        /* 
        let second_uproblem = initial_problem.advance(best_card);        

        println!("Start calculating 2...");
        let result = estimator.estimate_probability_of_all_cards(false);

        let best_card = result[0].0.__str();
        let best_card_probability = result[0].1;

        println!("Playing {} ({}) ...",best_card, best_card_probability);
        */

    }

}

#[cfg(test)]
mod tests {
    use crate::{uncertain::{uproblem_builder::UProblemBuilder, estimator::Estimator}, types::player::Player};

    #[test]
    pub fn test() {
        
        let up = UProblemBuilder::new_farbspiel()
        .cards(Player::Declarer, "CJ SJ D7")
        .remaining_cards("HJ DJ DA DT H7 H8")
        .threshold_half()
        .build();

        Estimator::playout(up);
    }
}