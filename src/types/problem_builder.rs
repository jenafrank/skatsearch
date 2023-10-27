use crate::{traits::{BitConverter, Augen}, core_functions::get_all_unplayed_cards::get_all_unplayed_cards};

use super::{game::Game, player::Player, problem::Problem};

pub struct ProblemBuilder {
    declarer_cards: Option<u32>,
    left_cards: Option<u32>,
    right_cards: Option<u32>,
    game_type: Option<Game>,
    start_player: Option<Player>,
    threshold_upper: Option<u8>,
    trick_cards: Option<u32>,
    trick_suit: Option<u32>,
}

impl ProblemBuilder {

    pub fn new(game: Game) -> ProblemBuilder {
        let mut builder = ProblemBuilder::default();
        builder.game_type = Some(game);
        builder
    }

    pub fn new_farbspiel() -> ProblemBuilder {
        ProblemBuilder::new(Game::Farbe)
    }

    pub fn new_grand() -> ProblemBuilder {
        ProblemBuilder::new(Game::Grand)
    }

    pub fn new_null() -> ProblemBuilder {
        let builder = ProblemBuilder::new(Game::Null);
        builder.threshold(1)        
    }

    pub fn cards(mut self, player: Player, cards: &str) -> ProblemBuilder {
        let cards_bit = cards.__bit();
        match player {
            Player::Declarer => self.declarer_cards = Some(cards_bit),
            Player::Left => self.left_cards = Some(cards_bit),
            Player::Right => self.right_cards = Some(cards_bit)
        }
        self
    }

    pub fn cards_all(mut self, declarer_cards: &str, left_cards: &str, right_cards: &str) -> ProblemBuilder {
        self.declarer_cards = Some(declarer_cards.__bit());
        self.left_cards = Some(left_cards.__bit());
        self.right_cards = Some(right_cards.__bit());
        self
    }

    pub fn turn(mut self, player: Player) -> ProblemBuilder {
        self.start_player = Some(player);
        self
    }

    pub fn threshold(mut self, threshold_upper: u8) -> ProblemBuilder {
        self.threshold_upper = Some(threshold_upper);
        self
    }

    pub fn threshold_half(mut self) -> ProblemBuilder {
        let declarer_cards = self.declarer_cards.expect("No declarer cards have been set.");
        let left_cards = self.left_cards.expect("No declarer cards have been set.");
        let right_cards = self.right_cards.expect("No declarer cards have been set.");


        let all_cards = get_all_unplayed_cards(declarer_cards, left_cards, right_cards);
        self.threshold_upper = Some((all_cards.__get_value() as u8 / 2) + 1);
        self
    }

    pub fn trick(mut self, trick_suit: u32, trick_cards: &str) -> ProblemBuilder {
        self.trick_cards = Some(trick_cards.__bit());
        self.trick_suit = Some(trick_suit);
        self
    }

    pub fn build(self) -> Problem {

        self.validate();

        let mut problem = Problem::new();

        if let Some(game_type) = self.game_type {
            problem.set_game_type(game_type);
        }

        if let Some(cards) = self.declarer_cards {
            problem.set_declarer_cards(cards);
        }

        if let Some(cards) = self.left_cards {
            problem.set_left_cards(cards);
        }

        if let Some(cards) = self.right_cards {
            problem.set_right_cards(cards);
        }

        if let Some(start_player) = self.start_player {
            problem.set_start_player(start_player);
        }

        if let Some(threshold_upper) = self.threshold_upper {
            problem.set_threshold_upper(threshold_upper);
        }

        if let Some(trick_cards) = self.trick_cards {
            problem.set_trick_cards(trick_cards);
        }

        if let Some(trick_suit) = self.trick_suit {
            problem.set_trick_suit(trick_suit);
        }

        problem
    }

    fn validate(&self) {
        // TODO:
        // check card numbers with respect to cards_in_trick
        // check threshold lower than total augen value
        //         
    }
}

impl Default for ProblemBuilder {
    fn default() -> ProblemBuilder {
        ProblemBuilder {
            declarer_cards: None,
            left_cards: None,
            right_cards: None,
            game_type: None,
            start_player: None,
            threshold_upper: Some(1),
            trick_cards: Some(0),
            trick_suit: Some(0),
        }
    }
}
