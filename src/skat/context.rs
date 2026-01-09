//! # Game Context
//!
//! Defined the static parameters of a game instance (cards distribution, game type, etc.)
//! Formerly `Problem`.

use crate::skat::defs::{Game, Player, CLUBS, DIAMONDS, HEARTS, SPADES};
use crate::skat::position::Position;
use crate::skat::rules::get_suit_for_card;
use crate::traits::Bitboard; // Need to ensure traits are available or moved

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProblemTransformation {
    SpadesSwitch,
    HeartsSwitch,
    DiamondsSwitch,
}

#[derive(Clone, Copy)]
pub struct GameContext {
    pub declarer_cards: u32,
    pub left_cards: u32,
    pub right_cards: u32,
    pub game_type: Game,
    pub start_player: Player,
    pub threshold_upper: u8,
    pub trick_cards: u32,
    pub trick_suit: u32,
    pub declarer_start_points: u8,
}

impl GameContext {
    pub fn create(
        declarer_cards: u32,
        left_cards: u32,
        right_cards: u32,
        game_type: Game,
        start_player: Player,
    ) -> Self {
        Self {
            declarer_cards,
            left_cards,
            right_cards,
            game_type,
            start_player,
            threshold_upper: 61, // Default
            trick_cards: 0,
            trick_suit: 0,
            declarer_start_points: 0,
        }
    }

    // Default equivalent (since Default trait requires 0 values which might not make sense for Game)
    pub fn default() -> Self {
        Self {
            declarer_cards: 0,
            left_cards: 0,
            right_cards: 0,
            game_type: Game::Grand,
            start_player: Player::Declarer,
            threshold_upper: 61,
            trick_cards: 0,
            trick_suit: 0,
            declarer_start_points: 0,
        }
    }

    pub fn set_declarer_cards(&mut self, declarer_cards: u32) {
        self.declarer_cards = declarer_cards;
    }

    pub fn set_left_cards(&mut self, left_cards: u32) {
        self.left_cards = left_cards;
    }

    pub fn set_right_cards(&mut self, right_cards: u32) {
        self.right_cards = right_cards;
    }

    pub fn set_game_type(&mut self, game_type: Game) {
        self.game_type = game_type;
    }

    pub fn set_start_player(&mut self, start_player: Player) {
        self.start_player = start_player;
    }

    pub fn set_threshold_upper(&mut self, threshold_upper: u8) {
        self.threshold_upper = threshold_upper;
    }

    pub fn set_trick_cards(&mut self, trick_cards: u32) {
        self.trick_cards = trick_cards;
    }

    pub fn set_trick_suit(&mut self, trick_suit: u32) {
        self.trick_suit = trick_suit;
    }

    pub fn set_declarer_start_points(&mut self, points: u8) {
        self.declarer_start_points = points;
    }

    // Getters

    pub fn declarer_cards(&self) -> u32 {
        self.declarer_cards
    }

    pub fn left_cards(&self) -> u32 {
        self.left_cards
    }

    pub fn right_cards(&self) -> u32 {
        self.right_cards
    }

    pub fn game_type(&self) -> Game {
        self.game_type
    }

    pub fn start_player(&self) -> Player {
        self.start_player
    }

    pub fn points_to_win(&self) -> u8 {
        self.threshold_upper
    }

    pub fn trick_cards(&self) -> u32 {
        self.trick_cards
    }

    pub fn trick_suit(&self) -> u32 {
        self.trick_suit
    }

    // Derived logic

    pub fn total_points(&self) -> u8 {
        120
    }

    pub fn get_skat(&self) -> u32 {
        use crate::skat::defs::ALLCARDS;
        ALLCARDS ^ self.declarer_cards ^ self.left_cards ^ self.right_cards ^ self.trick_cards
    }

    pub fn create_initial_position(&self) -> Position {
        Position::create_initial_position(self) // Calls Position::create... which takes &GameContext (renamed Problem)
    }

    // Transformations

    pub fn create_transformation(p: GameContext, switch: ProblemTransformation) -> GameContext {
        let switched_declarer_cards = GameContext::get_switched_cards(p.declarer_cards, switch);
        let switched_left_cards = GameContext::get_switched_cards(p.left_cards, switch);
        let switched_right_cards = GameContext::get_switched_cards(p.right_cards, switch);

        GameContext {
            declarer_cards: switched_declarer_cards,
            left_cards: switched_left_cards,
            right_cards: switched_right_cards,
            game_type: p.game_type,
            start_player: p.start_player,
            threshold_upper: p.threshold_upper,
            trick_cards: p.trick_cards,
            trick_suit: p.trick_suit,
            declarer_start_points: p.declarer_start_points,
        }
    }

    pub fn get_switched_cards(cards: u32, switch: ProblemTransformation) -> u32 {
        let shift = match switch {
            ProblemTransformation::SpadesSwitch => 7usize,
            ProblemTransformation::HeartsSwitch => 14usize,
            ProblemTransformation::DiamondsSwitch => 21usize,
        };

        let switch_suit = match switch {
            ProblemTransformation::SpadesSwitch => SPADES,
            ProblemTransformation::HeartsSwitch => HEARTS,
            ProblemTransformation::DiamondsSwitch => DIAMONDS,
        };

        let mut ret = 0u32;
        let decomposed_cards = cards.__decompose();

        for i in 0..decomposed_cards.1 {
            let current_card = decomposed_cards.0[i];
            let mut target_card = current_card;

            if current_card & CLUBS > 0 {
                target_card = target_card >> shift;
            }

            if current_card & switch_suit > 0 {
                target_card = target_card << shift;
            }

            ret = ret ^ target_card;
        }

        ret
    }

    pub fn validate(&self) -> Result<(), String> {
        self.check_equal_card_count()?;
        // Check trick suit
        if self.trick_cards != 0 {
            if self.trick_suit == 0 {
                return Err("Trick cards present but trick suit is 0".to_string());
            }
            let (cards, n) = self.trick_cards.__decompose();
            let mut valid = false;
            for i in 0..n {
                let s = get_suit_for_card(cards[i], self.game_type);
                if s == self.trick_suit {
                    valid = true;
                    break;
                }
            }
            if !valid {
                return Err(format!(
                    "Trick suit {} is not compatible with any card on the table",
                    self.trick_suit
                ));
            }
        }
        Ok(())
    }

    fn check_equal_card_count(&self) -> Result<(), String> {
        if self.trick_cards == 0 {
            let n_declarer = self.declarer_cards.count_ones();
            let n_left = self.left_cards.count_ones();
            let n_right = self.right_cards.count_ones();

            if n_declarer != n_left || n_declarer != n_right {
                return Err(format!(
                    "Card counts mismatch: Declarer={}, Left={}, Right={}. They must be equal when no trick is active.",
                    n_declarer, n_left, n_right
                ));
            }
        }
        Ok(())
    }
}
