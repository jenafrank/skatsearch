use crate::traits::StringConverter;
use crate::types::game::Game;
use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::problem_builder::ProblemBuilder;

use super::facts::Facts;

pub struct UncertainProblem {
    game_type: Game,
    my_player: Player,
    my_cards: u32,
    next_player: Player,

    // Primary values
    card_on_table_previous_player: u32,
    card_on_table_next_player: u32,
    all_cards: u32,
    active_suit: u32,    
    threshold_upper: u8,

    // Facts
    facts_declarer: Facts,
    facts_left: Facts,
    facts_right: Facts,
}

// Gettter
impl UncertainProblem {
    pub fn game_type(&self) -> Game {
        self.game_type
    }

    pub fn my_player(&self) -> Player {
        self.my_player
    }

    pub fn my_cards(&self) -> u32 {
        self.my_cards
    }

    pub fn next_player(&self) -> Player {
        self.next_player
    }

    pub fn card_on_table_previous_player(&self) -> u32 {
        self.card_on_table_previous_player
    }

    pub fn card_on_table_next_player(&self) -> u32 {
        self.card_on_table_next_player
    }

    pub fn cards_on_table(&self) -> u32 {
        self.card_on_table_previous_player | self.card_on_table_next_player
    }

    pub fn all_cards(&self) -> u32 {
        self.all_cards
    }

    pub fn active_suit(&self) -> u32 {
        self.active_suit
    }

    pub fn threshold_upper(&self) -> u8 {
        self.threshold_upper
    }

    pub fn facts_declarer(&self) -> Facts {
        self.facts_declarer
    }

    pub fn facts_left(&self) -> Facts {
        self.facts_left
    }

    pub fn facts_right(&self) -> Facts {
        self.facts_right
    }
}

// Setter
impl UncertainProblem {
    pub fn set_game_type(&mut self, game_type: Game) {
        self.game_type = game_type;
    }

    pub fn set_my_player(&mut self, my_player: Player) {
        self.my_player = my_player;
    }

    pub fn set_my_cards(&mut self, my_cards: u32) {
        self.my_cards = my_cards;
    }

    pub fn set_next_player(&mut self, next_player: Player) {
        self.next_player = next_player;
    }

    pub fn set_card_on_table_previous_player(&mut self, card_on_table_previous_player: u32) {
        self.card_on_table_previous_player = card_on_table_previous_player;
    }

    pub fn set_card_on_table_next_player(&mut self, card_on_table_next_player: u32) {
        self.card_on_table_next_player = card_on_table_next_player;
    }

    pub fn set_all_cards(&mut self, all_cards: u32) {
        self.all_cards = all_cards;
    }

    pub fn set_active_suit(&mut self, active_suit: u32) {
        self.active_suit = active_suit;
    }

    pub fn set_threshold_upper(&mut self, threshold_upper: u8) {
        self.threshold_upper = threshold_upper;
    }

    pub fn set_facts_declarer(&mut self, facts_declarer: Facts) {
        self.facts_declarer = facts_declarer;
    }

    pub fn set_facts_left(&mut self, facts_left: Facts) {
        self.facts_left = facts_left;
    }

    pub fn set_facts_right(&mut self, facts_right: Facts) {
        self.facts_right = facts_right;
    }
}

impl UncertainProblem {
    pub fn new() -> Self {
        UncertainProblem {
            game_type: Game::Farbe,
            my_cards: 0u32,
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            all_cards: 0u32,
            card_on_table_previous_player: 0u32,
            card_on_table_next_player: 0u32,
            active_suit: 0u32,
            threshold_upper: 1u8,
            facts_declarer: Facts::zero_fact(),
            facts_left: Facts::zero_fact(),
            facts_right: Facts::zero_fact()
        }
    }

    pub fn generate_concrete_problem(&self) -> Problem {
        
        self.validate();

        let problem = ProblemBuilder::new(self.game_type)
        .cards(Player::Declarer, "")
        .cards(Player::Left, "")
        .cards(Player::Right, "")
        .turn(self.next_player)
        .trick(self.active_suit, self.cards_on_table().__str().as_str())
        .threshold(self.threshold_upper)
        .set_cards_for_problem(self.my_cards, self.my_player)
        .set_cards_for_other_players(self.all_cards, self.card_on_table_previous_player, self.card_on_table_next_player, self.my_cards, self.my_player, self.next_player_facts(), self.previous_player_facts())
        .build();
        
        if verify_card_distribution(&problem) {
            return problem;
        } else {
            panic!("Something went wrong in randomly select cards with given facts.");
        }
    }

    fn next_player_facts(&self) -> Facts {
        match self.my_player {
            Player::Declarer => self.facts_left,
            Player::Left => self.facts_right,
            Player::Right => self.facts_declarer,
        }
    }

    fn previous_player_facts(&self) -> Facts {
        match self.my_player {
            Player::Declarer => self.facts_right,
            Player::Left => self.facts_declarer,
            Player::Right => self.facts_left,
        }
    }

    fn validate(&self) {
        self.validate_all_cards();
    }

    fn validate_all_cards(&self) {
        assert!(self.all_cards & self.my_cards == self.my_cards);
        assert!(self.all_cards & self.cards_on_table() == self.cards_on_table());

        // currently uncertain problems can only be solved before a trick starts:
        assert!(self.all_cards.count_ones() % 3 == 0);
    }

}

fn verify_card_distribution(problem: &Problem) -> bool {
    assert!(problem.declarer_cards() & problem.left_cards() == 0);
    assert!(problem.declarer_cards() & problem.right_cards() == 0);
    assert!(problem.left_cards() & problem.right_cards() == 0);

    return true;
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::UncertainProblem;
    use crate::{
        traits::{BitConverter, StringConverter},
        types::{game::Game, player::Player},
        uncertain::uncertain_problem::Facts, consts::bitboard::TRUMP_FARBE,
    };

    #[test]
    fn test_problem_generation() {

        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            card_on_table_next_player: 0u32,
            card_on_table_previous_player: 0u32,
            active_suit: 0u32,
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            threshold_upper: 1,
            facts_declarer: Facts::zero_fact(),
            facts_left: Facts::one_fact(true, false, false, false, false),
            facts_right: Facts::zero_fact()            
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards().__str());
        println!("Left cards    : {}", problem.left_cards().__str());
        println!("Right cards   : {}", problem.right_cards().__str());
    }

    #[test]
    fn test_inter_trick_problem_generation() {

        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            my_player: Player::Declarer,
            next_player: Player::Declarer,
            threshold_upper: 1,
            card_on_table_previous_player: "ST".__bit(),
            card_on_table_next_player: 0u32,
            active_suit: TRUMP_FARBE,
            facts_declarer: Facts::zero_fact(),
            facts_left: Facts::zero_fact(),
            facts_right: Facts::zero_fact(),
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards().__str());
        println!("Left cards    : {}", problem.left_cards().__str());
        println!("Right cards   : {}", problem.right_cards().__str());
    }
}
