use crate::traits::StringConverter;
use super::estimator::Estimator;

impl Estimator {

    pub fn playout(&self)     
    {
        let result = self.estimate_probability_of_all_cards(false);
        println!("Play out card {} with probabilty {}.",result[0].0.__str(), result[0].1);
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

        let estimator = Estimator::new(up, 100);

        estimator.playout();                

    }
}