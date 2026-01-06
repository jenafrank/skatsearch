use crate::skat::builder::GameContextBuilder;
use crate::skat::context::GameContext;
use crate::skat::defs::Game;
use crate::skat::defs::Player;
use crate::traits::{BitConverter, StringConverter};

use super::facts::Facts;

#[derive(Clone, Copy)]
pub struct UncertainProblem {
    game_type: Game,
    my_player: Player,
    my_cards: u32,

    // Primary values
    card_on_table_previous_player: u32,
    card_on_table_next_player: u32,
    all_cards: u32,
    threshold_upper: u8,

    // Facts
    facts_previous_player: Facts,
    facts_next_player: Facts,
}

impl UncertainProblem {
    pub fn advance(&self, played_card: String) -> UncertainProblem {
        let mut ret = self.clone();

        ret.print_object();

        // Check if facts are added
        // todo!() assuming no for now

        // Check if trick completed => Remove trick from table and adjust points
        // todo!() assuming no for now

        // Add card to trick card
        ret.card_on_table_previous_player = played_card.__bit();

        ret.print_object();

        ret
    }
    fn print_object(&self) {
        println!("");
        println!("DUMPING UNCERTAIN PROBLEM: ");
        println!("-------------------------- ");
        println!("game type = {}", self.game_type.convert_to_string());
        println!("my player = {}", self.my_player.str());
        println!("my cards = {}", self.my_cards.__str());
        println!(
            "trick_card_previous_player = {}",
            self.card_on_table_previous_player.__str()
        );
        println!(
            "trick_card_next_player = {}",
            self.card_on_table_next_player.__str()
        );
        println!("all cards = {}", self.all_cards.__str());
        println!("threshold = {}", self.threshold_upper);
        println!(
            "facts left = {}",
            self.facts_previous_player.convert_to_string()
        );
        println!(
            "facts right = {}",
            self.facts_next_player.convert_to_string()
        );
    }
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

    pub fn threshold_upper(&self) -> u8 {
        self.threshold_upper
    }

    pub fn facts_previous_player(&self) -> Facts {
        self.facts_previous_player
    }

    pub fn facts_next_player(&self) -> Facts {
        self.facts_next_player
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

    pub fn set_card_on_table_previous_player(&mut self, card_on_table_previous_player: u32) {
        self.card_on_table_previous_player = card_on_table_previous_player;
    }

    pub fn set_card_on_table_next_player(&mut self, card_on_table_next_player: u32) {
        self.card_on_table_next_player = card_on_table_next_player;
    }

    pub fn set_all_cards(&mut self, all_cards: u32) {
        self.all_cards = all_cards;
    }

    pub fn set_threshold_upper(&mut self, threshold_upper: u8) {
        self.threshold_upper = threshold_upper;
    }

    pub fn set_facts_left(&mut self, facts_left: Facts) {
        self.facts_previous_player = facts_left;
    }

    pub fn set_facts_right(&mut self, facts_right: Facts) {
        self.facts_next_player = facts_right;
    }
}

impl UncertainProblem {
    pub fn new() -> Self {
        UncertainProblem {
            game_type: Game::Farbe,
            my_cards: 0u32,
            my_player: Player::Declarer,
            all_cards: 0u32,
            card_on_table_previous_player: 0u32,
            card_on_table_next_player: 0u32,
            threshold_upper: 1u8,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::zero_fact(),
        }
    }

    pub fn generate_concrete_problem(&self) -> GameContext {
        self.validate();

        let problem = GameContextBuilder::new(self.game_type)
            .cards(Player::Declarer, "")
            .cards(Player::Left, "")
            .cards(Player::Right, "")
            .turn(self.my_player)
            .trick_from_uproblem(
                self.card_on_table_previous_player,
                self.card_on_table_next_player,
            )
            .threshold(self.threshold_upper)
            .set_cards_for_problem(self.my_cards, self.my_player)
            .set_cards_for_other_players(
                self.all_cards,
                self.card_on_table_previous_player,
                self.card_on_table_next_player,
                self.my_cards,
                self.my_player,
                self.next_player_facts(),
                self.previous_player_facts(),
            )
            .build();

        if verify_card_distribution(&problem) {
            return problem;
        } else {
            panic!("Something went wrong in randomly select cards with given facts.");
        }
    }

    fn next_player_facts(&self) -> Facts {
        self.facts_next_player
    }

    fn previous_player_facts(&self) -> Facts {
        self.facts_previous_player
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

fn verify_card_distribution(problem: &GameContext) -> bool {
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
        skat::defs::{Game, Player},
        traits::{BitConverter, StringConverter},
        uncertain::uncertain_problem::Facts,
    };

    #[test]
    fn test_problem_generation() {
        let uproblem = UncertainProblem {
            game_type: Game::Farbe,
            all_cards: "CA CT SA ST HA HT DA DT D9".__bit(),
            my_cards: "CA CT SA".__bit(),
            card_on_table_next_player: 0u32,
            card_on_table_previous_player: 0u32,
            my_player: Player::Declarer,
            threshold_upper: 1,
            facts_previous_player: Facts::one_fact(true, false, false, false, false),
            facts_next_player: Facts::zero_fact(),
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
            threshold_upper: 1,
            card_on_table_previous_player: "ST".__bit(),
            card_on_table_next_player: 0u32,
            facts_previous_player: Facts::zero_fact(),
            facts_next_player: Facts::zero_fact(),
        };

        let problem = uproblem.generate_concrete_problem();

        println!("Declarer cards: {}", problem.declarer_cards().__str());
        println!("Left cards    : {}", problem.left_cards().__str());
        println!("Right cards   : {}", problem.right_cards().__str());
    }
}
