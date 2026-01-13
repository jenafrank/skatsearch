use crate::extensions::solver::{solve_optimum_from_position, OptimumMode};
use crate::skat::context::GameContext;
use crate::skat::defs::{Game, Player, ALLCARDS};
use crate::skat::engine::SkatEngine;
use crate::skat::position::Position;
use crate::traits::{BitConverter, Bitboard, Points, StringConverter};
use serde::{Deserialize, Serialize};
use std::panic;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct GameStateJson {
    pub my_cards: String,
    pub trick_cards: String,
    pub trick_suit: String,
    pub current_value: i32,
    pub last_loss: i32,
    pub game_over: bool,
    pub winner: Option<String>,
    pub declarer_points: u8,
    pub team_points: u8,
    pub current_player: String,
    pub last_trick_cards: Option<String>,
    pub last_trick_winner: Option<String>,
    pub last_trick_points: Option<u8>,
    pub left_cards: String,
    pub right_cards: String,
}

#[derive(Serialize)]
pub struct HintJson {
    pub best_card: String,
    pub value: i32,
}

#[wasm_bindgen]
pub struct SkatGame {
    engine: SkatEngine,
    user_player: Player,
    history: Vec<Position>,
    last_loss: i32,
    last_trick_cards: Option<u32>,
    last_trick_winner: Option<Player>,
    last_trick_points: Option<u8>,
    current_position: Position,
}

#[wasm_bindgen]
impl SkatGame {
    pub fn new_random() -> SkatGame {
        let (mut deck, _count) = ALLCARDS.__decompose();

        // Convert to vec and shuffle
        let mut deck_vec = deck.to_vec();
        deck_vec.truncate(32);

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        deck_vec.shuffle(&mut rng);

        let mut decl_cards = 0u32;
        let mut left_cards = 0u32;
        let mut right_cards = 0u32;
        let mut skat_cards = 0u32;

        for i in 0..10 {
            decl_cards |= deck_vec[i];
        }
        for i in 10..20 {
            left_cards |= deck_vec[i];
        }
        for i in 20..30 {
            right_cards |= deck_vec[i];
        }
        skat_cards |= deck_vec[30];
        skat_cards |= deck_vec[31];

        let mut context = GameContext::create(
            decl_cards,
            left_cards,
            right_cards,
            Game::Suit,       // Clubs default
            Player::Declarer, // Start Player is Human
        );

        let skat_points = skat_cards.points();
        context.set_declarer_start_points(skat_points);
        context.set_threshold_upper(120);

        let engine = SkatEngine::new(context, None);
        let current_position = engine.create_initial_position();

        SkatGame {
            engine,
            user_player: Player::Declarer,
            history: Vec::new(),
            last_loss: 0,
            last_trick_cards: None,
            last_trick_winner: None,
            last_trick_points: None,
            current_position,
        }
    }

    pub fn get_state_json(&self) -> JsValue {
        let pos = self.current_position;

        let display_cards = if self.user_player == Player::Declarer {
            pos.declarer_cards.__str()
        } else {
            "".to_string()
        };

        // Always expose opponent cards for Cheat Mode (frontend decides visibility)
        let left_str = pos.left_cards.__str();
        let right_str = pos.right_cards.__str();

        let trick_str = if pos.trick_cards != 0 {
            pos.trick_cards.__str()
        } else {
            "".to_string()
        };

        let current_value = if !self.is_game_over() {
            self.calculate_theoretical_value()
        } else {
            0
        };

        let game_over = self.is_game_over();

        let state = GameStateJson {
            my_cards: display_cards,
            trick_cards: trick_str,
            trick_suit: format!("{}", pos.trick_suit),
            current_value,
            last_loss: self.last_loss,
            game_over,
            winner: if game_over {
                Some(if pos.declarer_points > 60 {
                    "Declarer".to_string()
                } else {
                    "Opponents".to_string()
                })
            } else {
                None
            },
            declarer_points: pos.declarer_points,
            team_points: pos.team_points,
            current_player: pos.player.str().to_string(), // "D", "L", "R"
            last_trick_cards: self.last_trick_cards.map(|c| c.__str()),
            last_trick_winner: self.last_trick_winner.map(|p| p.str().to_string()),
            last_trick_points: self.last_trick_points,
            left_cards: left_str,
            right_cards: right_str,
        };

        serde_wasm_bindgen::to_value(&state).unwrap()
    }

    pub fn play_card_str(&mut self, card_str: &str) -> bool {
        let card = card_str.trim().__bit();
        if card == 0 {
            return false;
        }

        let pos = self.current_position;
        if pos.player != self.user_player {
            return false;
        }

        self.perform_move(card, &pos)
    }

    pub fn make_ai_move(&mut self) -> bool {
        let pos = self.current_position;
        if pos.player == self.user_player {
            return false; // Human turn
        }
        if self.is_game_over() {
            return false;
        }

        // God Mode Solve
        let (best_card, _) = self.solve_best_move();
        if best_card == 0 {
            return false;
        }

        self.perform_move(best_card, &pos)
    }

    fn perform_move(&mut self, card: u32, pos: &Position) -> bool {
        let legal_moves = pos.get_legal_moves();
        if (card & legal_moves) == 0 {
            return false;
        }

        // Save history (Position)
        self.history.push(*pos);

        let best_val_before = self.calculate_theoretical_value();

        // Check for trick completion
        let trick_will_complete = pos.trick_cards.count_ones() == 2;
        let mut completed_trick_info = None;
        if trick_will_complete {
            let full_trick = pos.trick_cards | card;
            completed_trick_info = Some(full_trick);
        }

        // Make move using Full Context (stored in engine.context)
        let next_pos = pos.make_move(card, &self.engine.context);

        if let Some(trick) = completed_trick_info {
            self.last_trick_cards = Some(trick);
            self.last_trick_winner = Some(next_pos.player); // winner starts next

            let points_gained = if next_pos.player == Player::Declarer {
                next_pos.declarer_points.saturating_sub(pos.declarer_points)
            } else {
                next_pos.team_points.saturating_sub(pos.team_points)
            };

            self.last_trick_points = Some(points_gained);
        }

        // Update current position (GameContext is NOT modulated)
        self.current_position = next_pos;

        let best_val_after = self.calculate_theoretical_value();

        if best_val_after < best_val_before {
            self.last_loss = best_val_before - best_val_after;
        } else {
            // Trick change often results in value jump if heuristic isn't perfect,
            // but in God Mode solve it should be stable or decrease.
            self.last_loss = 0;
        }

        true
    }

    pub fn undo(&mut self) {
        if let Some(prev_pos) = self.history.pop() {
            self.current_position = prev_pos;
            self.last_loss = 0;
            self.last_trick_cards = None;
            self.last_trick_winner = None;
            self.last_trick_points = None;
        }
    }

    pub fn get_hint_json(&self) -> JsValue {
        let (card, val) = self.solve_best_move();
        let hint = HintJson {
            best_card: card.__str(),
            value: val as i32,
        };
        serde_wasm_bindgen::to_value(&hint).unwrap()
    }

    fn is_game_over(&self) -> bool {
        let pos = self.current_position;
        pos.declarer_cards == 0
            && pos.left_cards == 0
            && pos.right_cards == 0
            && pos.trick_cards == 0
    }

    fn solve_best_move(&self) -> (u32, i32) {
        // Create temp engine using Full Context (kept in self.engine.context)
        let mut temp_engine = SkatEngine::new(self.engine.context, None);
        let pos = self.current_position;

        // Solve from current position using temp engine
        let res = solve_optimum_from_position(&mut temp_engine, &pos, OptimumMode::BestValue);

        if let Ok((card, _score, val)) = res {
            (card, val as i32)
        } else {
            (0, 0)
        }
    }

    fn calculate_theoretical_value(&self) -> i32 {
        if self.is_game_over() {
            return 0;
        }
        let (_, val) = self.solve_best_move();
        val
    }
}
