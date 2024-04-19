use crate::{types::{game::Game, player::Player}, traits::{BitConverter, Augen}, consts::bitboard::ALLCARDS};
use super::{uncertain_problem::UncertainProblem, facts::Facts};

pub struct UProblemBuilder {
    game_type: Option<Game>,
    my_player: Option<Player>,
    my_cards: Option<u32>,
    next_player: Option<Player>,

    // Primary values
    card_on_table_previous_player: Option<u32>,
    card_on_table_next_player: Option<u32>,
    all_cards: Option<u32>,
    active_suit: Option<u32>,
    threshold_upper: Option<u8>,

    // Facts
    facts_declarer: Option<Facts>,
    facts_left: Option<Facts>,
    facts_right: Option<Facts>,
}

impl UProblemBuilder {
    pub fn new(game: Game) -> UProblemBuilder {
        let mut builder = UProblemBuilder::default();
        builder.game_type = Some(game);
        builder
    }

    pub fn new_farbspiel() -> UProblemBuilder {
        UProblemBuilder::new(Game::Farbe)
    }

    pub fn new_grand() -> UProblemBuilder {
        UProblemBuilder::new(Game::Grand)
    }

    pub fn new_null() -> UProblemBuilder {
        let builder = UProblemBuilder::new(Game::Null);
        builder.threshold(1)        
    }

    pub fn cards(mut self, player: Player, cards: &str) -> UProblemBuilder {
        self.my_player = Some(player);
        self.next_player = Some(player);
        self.my_cards = Some(cards.__bit());
        self
    }

    pub fn turn(mut self, player: Player) -> UProblemBuilder {
        self.next_player = Some(player);
        self
    }

    pub fn threshold(mut self, threshold_upper: u8) -> UProblemBuilder {
        self.threshold_upper = Some(threshold_upper);
        self
    }

    pub fn threshold_half(mut self) -> UProblemBuilder {
        let all_cards = self.all_cards.expect("No all cards found.");
        self.threshold_upper = Some((all_cards.__get_value() as u8 / 2) + 1);
        self
    }

    pub fn trick_previous_player(mut self, active_suit: u32, trick_previous_player: u32) -> UProblemBuilder {
        self.card_on_table_previous_player = Some(trick_previous_player);
        self.active_suit = Some(active_suit);
        self
    }

    pub fn facts(mut self, player: Player, facts: Facts) -> UProblemBuilder {
        match player {
            Player::Declarer => self.facts_declarer = Some(facts),
            Player::Left => self.facts_left = Some(facts),
            Player::Right => self.facts_right = Some(facts),
        }
        self
    }

    // cards part of the game
    pub fn remaining_cards(mut self, remaining_cards: &str) -> UProblemBuilder{
        let remaining_cards_bit = remaining_cards.__bit();
        let my_cards_bit = self.my_cards.expect("No own cards found.");
        let cards_on_table = self.cards_on_table();
        
        assert!(remaining_cards_bit & my_cards_bit == 0);
        assert!(remaining_cards_bit & cards_on_table == 0);
        assert!(my_cards_bit & cards_on_table == 0);
        
        self.all_cards = Some(
            remaining_cards_bit | 
            my_cards_bit |
            cards_on_table
        );
        self
    }

    // cards not part of the game
    pub fn missing_cards(mut self, missing_cards: &str) -> UProblemBuilder {
        let missing_cards_bit = missing_cards.__bit();
        let my_cards_bit = self.my_cards.expect("No own cards found.");
        let cards_on_table = self.cards_on_table();
        
        assert!(missing_cards_bit & my_cards_bit == 0);
        assert!(missing_cards_bit & cards_on_table == 0);
        assert!(my_cards_bit & cards_on_table == 0);
        
        self.all_cards = Some(
            (ALLCARDS & !missing_cards_bit) | 
            my_cards_bit |
            cards_on_table
        );
        self
    }

    pub fn skat_cards(self, skat_cards: &str) -> UProblemBuilder {
        let skat_cards_bit = skat_cards.__bit();
        assert!(skat_cards_bit.count_ones() == 2);
        self.missing_cards(skat_cards)
    }

    pub fn build(self) -> UncertainProblem {

        self.validate();

        let mut uproblem = UncertainProblem::new();

        if let Some(game_type) = self.game_type {
            uproblem.set_game_type(game_type);
        }

        if let Some(my_player) = self.my_player {
            uproblem.set_my_player(my_player);
        }

        if let Some(my_cards) = self.my_cards {
            uproblem.set_my_cards(my_cards);
        }     

        if let Some(card_on_table_previous_player) = self.card_on_table_previous_player {
            uproblem.set_card_on_table_previous_player(card_on_table_previous_player);
        }

        if let Some(card_on_table_next_player) = self.card_on_table_next_player {
            uproblem.set_card_on_table_next_player(card_on_table_next_player);
        }

        if let Some(all_cards) = self.all_cards {
            uproblem.set_all_cards(self.cards_on_table() | all_cards);         
        }       

        if let Some(upper_bound_of_null_window) = self.threshold_upper {
            uproblem.set_threshold_upper(upper_bound_of_null_window);
        }       

        if let Some(facts) = self.facts_left {
            uproblem.set_facts_left(facts);
        }

        if let Some(facts) = self.facts_right {
            uproblem.set_facts_right(facts);
        }

        uproblem
    }

    fn validate(&self) {
        self.validate_nothing_none();
        self.validate_number_of_cards();
    }

    fn validate_nothing_none(&self) {
        if self.game_type.is_none()
            || self.my_player.is_none()
            || self.my_cards.is_none()
            || self.next_player.is_none()
            || self.card_on_table_next_player.is_none()
            || self.card_on_table_previous_player.is_none()
            || self.all_cards.is_none()
            || self.active_suit.is_none()
            || self.threshold_upper.is_none()
            || self.facts_declarer.is_none()
            || self.facts_left.is_none()
            || self.facts_right.is_none()
        {
            if self.game_type.is_none() { println!("Game Type missing."); }
            if self.my_player.is_none() { println!("My player missing."); }
            if self.my_cards.is_none() { println!("My cards missing."); }
            if self.next_player.is_none() { println!("Next player missing."); }
            if self.card_on_table_next_player.is_none() { println!("Card on table next player missing."); }
            if self.card_on_table_previous_player.is_none() { println!("Card on table previous player missing."); }
            if self.all_cards.is_none() { println!("All cards missing."); }
            if self.active_suit.is_none() { println!("Active suit missing."); }
            if self.threshold_upper.is_none() { println!("Upper Threshold missing."); }
            if self.facts_declarer.is_none() { println!("Facts Declarer missing."); }
            if self.facts_left.is_none() { println!("Facts Left missing."); }
            if self.facts_right.is_none() { println!("Facts Right missing."); }

            panic!("Incomplete build. Can not create uproblem from builder.");
        }
    }

    fn validate_number_of_cards(&self) {
        let nr_own_cards = self.my_cards.unwrap().count_ones();
        let nr_all_cards = self.all_cards.unwrap().count_ones();

        assert!(nr_all_cards %3 == 0);
        assert!(nr_all_cards == 3 * nr_own_cards);
    }

    fn cards_on_table(&self) -> u32 {
        self.card_on_table_next_player.unwrap_or(0u32) | self.card_on_table_previous_player.unwrap_or(0u32)
    }
}

impl Default for UProblemBuilder {
    fn default() -> Self {
        UProblemBuilder {
            game_type: None,
            my_player: None,
            my_cards: None,
            next_player: None,
            card_on_table_next_player: Some(0u32),
            card_on_table_previous_player: Some(0u32),
            all_cards: None,
            active_suit: Some(0),
            threshold_upper: None,
            facts_declarer: Some(Facts::zero_fact()),
            facts_left: Some(Facts::zero_fact()),
            facts_right: Some(Facts::zero_fact()),
        }
    }
}
