use super::{facts::Facts, pimc_problem::PimcProblem};
use crate::{
    consts::bitboard::ALLCARDS,
    skat::defs::{Game, Player},
    traits::{BitConverter, Points},
};

pub struct PimcProblemBuilder {
    game_type: Game,
    my_player: Option<Player>,
    my_cards: Option<u32>,
    next_player: Option<Player>,

    // Primary values
    previous_card: Option<u32>,
    next_card: Option<u32>,
    all_cards: Option<u32>,
    active_suit: Option<u32>,
    threshold: Option<u8>,

    // Facts
    facts_declarer: Option<Facts>,
    facts_left: Option<Facts>,
    facts_right: Option<Facts>,
}

impl PimcProblemBuilder {
    pub fn new(game_type: Game) -> PimcProblemBuilder {
        PimcProblemBuilder {
            game_type,
            ..PimcProblemBuilder::default()
        }
    }

    pub fn new_farbspiel() -> PimcProblemBuilder {
        PimcProblemBuilder::new(Game::Suit)
    }

    pub fn new_grand() -> PimcProblemBuilder {
        PimcProblemBuilder::new(Game::Grand)
    }

    pub fn new_null() -> PimcProblemBuilder {
        PimcProblemBuilder::new(Game::Null).threshold(1)
    }

    pub fn cards(mut self, player: Player, cards: &str) -> PimcProblemBuilder {
        self.my_player = Some(player);
        self.next_player = Some(player);
        self.my_cards = Some(cards.__bit());
        self
    }

    pub fn my_player(mut self, player: Player) -> PimcProblemBuilder {
        self.my_player = Some(player);
        self
    }

    pub fn turn(mut self, player: Player) -> PimcProblemBuilder {
        self.next_player = Some(player);
        self
    }

    pub fn threshold(mut self, threshold: u8) -> PimcProblemBuilder {
        self.threshold = Some(threshold);
        self
    }

    pub fn threshold_half(mut self) -> PimcProblemBuilder {
        let all_cards = self.all_cards.expect("No all cards found.");
        self.threshold = Some((all_cards.points() as u8 / 2) + 1);
        self
    }

    pub fn trick_previous_player(
        mut self,
        active_suit: u32,
        trick_previous_player: u32,
    ) -> PimcProblemBuilder {
        self.previous_card = Some(trick_previous_player);
        self.active_suit = Some(active_suit);
        self
    }

    pub fn facts(mut self, player: Player, facts: Facts) -> PimcProblemBuilder {
        match player {
            Player::Declarer => self.facts_declarer = Some(facts),
            Player::Left => self.facts_left = Some(facts),
            Player::Right => self.facts_right = Some(facts),
        }
        self
    }

    // cards part of the game
    pub fn remaining_cards(mut self, cards: &str) -> PimcProblemBuilder {
        let remaining_cards_bit = cards.__bit();
        let my_cards_bit = self.my_cards.expect("No own cards found.");
        let cards_on_table = self.cards_on_table();

        assert!(remaining_cards_bit & my_cards_bit == 0);
        assert!(remaining_cards_bit & cards_on_table == 0);
        assert!(my_cards_bit & cards_on_table == 0);

        self.all_cards = Some(remaining_cards_bit | my_cards_bit | cards_on_table);
        self
    }

    // cards not part of the game
    pub fn missing_cards(mut self, missing_cards: &str) -> PimcProblemBuilder {
        let missing_cards_bit = missing_cards.__bit();
        let my_cards_bit = self.my_cards.expect("No own cards found.");
        let cards_on_table = self.cards_on_table();

        assert!(missing_cards_bit & my_cards_bit == 0);
        assert!(missing_cards_bit & cards_on_table == 0);
        assert!(my_cards_bit & cards_on_table == 0);

        self.all_cards = Some((ALLCARDS & !missing_cards_bit) | my_cards_bit | cards_on_table);
        self
    }

    pub fn skat_cards(mut self, skat_cards: &str) -> PimcProblemBuilder {
        let skat_cards_bit = skat_cards.__bit();
        assert!(skat_cards_bit.count_ones() == 2);
        self.missing_cards(skat_cards)
    }

    pub fn build(self) -> PimcProblem {
        self.validate();

        let mut uproblem = PimcProblem::new();

        uproblem.set_game_type(self.game_type);

        if let Some(my_player) = self.my_player {
            uproblem.set_my_player(my_player);
        }

        if let Some(my_cards) = self.my_cards {
            uproblem.set_my_cards(my_cards);
        }

        if let Some(previous_card) = self.previous_card {
            uproblem.set_previous_card(previous_card);
        }

        if let Some(next_card) = self.next_card {
            uproblem.set_next_card(next_card);
        }

        if let Some(all_cards) = self.all_cards {
            uproblem.set_all_cards(self.cards_on_table() | all_cards);
        }

        if let Some(upper_bound_of_null_window) = self.threshold {
            uproblem.set_threshold(upper_bound_of_null_window);
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
        if self.my_player.is_none()
            || self.my_cards.is_none()
            || self.next_player.is_none()
            || self.next_card.is_none()
            || self.previous_card.is_none()
            || self.all_cards.is_none()
            || self.active_suit.is_none()
            || self.threshold.is_none()
            || self.facts_declarer.is_none()
            || self.facts_left.is_none()
            || self.facts_right.is_none()
        {
            if self.my_player.is_none() {
                println!("My player missing.");
            }
            if self.my_cards.is_none() {
                println!("My cards missing.");
            }
            if self.next_player.is_none() {
                println!("Next player missing.");
            }
            if self.next_card.is_none() {
                println!("Card on table next player missing.");
            }
            if self.previous_card.is_none() {
                println!("Card on table previous player missing.");
            }
            if self.all_cards.is_none() {
                println!("All cards missing.");
            }
            if self.active_suit.is_none() {
                println!("Active suit missing.");
            }
            if self.threshold.is_none() {
                println!("Upper Threshold missing.");
            }
            if self.facts_declarer.is_none() {
                println!("Facts Declarer missing.");
            }
            if self.facts_left.is_none() {
                println!("Facts Left missing.");
            }
            if self.facts_right.is_none() {
                println!("Facts Right missing.");
            }

            panic!("Incomplete build. Can not create uproblem from builder.");
        }
    }

    fn validate_number_of_cards(&self) {
        let nr_own_cards = self.my_cards.unwrap().count_ones();
        let nr_all_cards = self.all_cards.unwrap().count_ones();

        assert!(nr_all_cards % 3 == 0);
        assert!(nr_all_cards == 3 * nr_own_cards);
    }

    fn cards_on_table(&self) -> u32 {
        self.next_card.unwrap_or(0u32) | self.previous_card.unwrap_or(0u32)
    }
}

impl Default for PimcProblemBuilder {
    fn default() -> Self {
        PimcProblemBuilder {
            game_type: Game::Suit, // Default to a game type, as it's no longer Option
            my_player: None,
            my_cards: None,
            next_player: None,
            next_card: Some(0u32),
            previous_card: Some(0u32),
            all_cards: None,
            active_suit: Some(0),
            threshold: None,
            facts_declarer: Some(Facts::zero_fact()),
            facts_left: Some(Facts::zero_fact()),
            facts_right: Some(Facts::zero_fact()),
        }
    }
}
