mod search;
mod constructors;
mod methods;
mod traits;

use super::game::Game;
use super::player::Player;
use super::state::{State, StatePayload};

pub struct Problem {
    declarer_cards: u32,
    left_cards: u32,
    right_cards: u32,
    game_type: Game,
    start_player: Player,
    threshold_upper: u8,
    trick_cards: u32,
    trick_suit: u32 
}

// Setters

impl Problem {

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

}

// Getters

impl Problem {
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

}

impl Problem {

    pub fn new_state(&self, alpha: u8, beta: u8) -> State {

        let player_cards = match self.start_player {
            Player::Declarer => self.declarer_cards,
            Player::Left => self.left_cards,
            Player::Right => self.right_cards,
        };

        let trick_cards_count = self.trick_cards.count_ones() as u8;       

        State::new(StatePayload{
            player: self.start_player,
            played_cards: self.trick_cards,
            trick_cards: self.trick_cards,
            trick_suit: self.trick_suit,
            augen_declarer: 0,
            declarer_cards: self.declarer_cards,
            left_cards: self.left_cards,
            right_cards: self.right_cards,
            player_cards,
            trick_cards_count,
            augen_future: self.augen_total(),
            augen_team: 0,
            alpha,
            beta,
            is_root_state: true,
        })
    }
}