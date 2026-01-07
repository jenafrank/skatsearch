use super::pimc_search::PimcSearch;
use crate::pimc::pimc_problem::PimcProblem;
use crate::traits::StringConverter;

impl PimcSearch {
    pub fn playout(initial_problem: PimcProblem) {
        let first_uproblem = initial_problem;
        let estimator = PimcSearch::new(first_uproblem, 100);

        println!("Start calculating 1...");
        let result = estimator.estimate_probability_of_all_cards(false);

        let best_card = result[0].0.__str();
        let best_card_probability = result[0].1;

        println!("Playing {} ({}) ...", best_card, best_card_probability);

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
    use crate::{
        pimc::{pimc_problem_builder::PimcProblemBuilder, pimc_search::PimcSearch},
        skat::defs::Player,
    };

    #[test]
    pub fn test() {
        let up = PimcProblemBuilder::new_farbspiel()
            .cards(Player::Declarer, "CJ SJ D7")
            .remaining_cards("HJ DJ DA DT H7 H8")
            .threshold_half()
            .build();

        PimcSearch::playout(up);
    }
}
