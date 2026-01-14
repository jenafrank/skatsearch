use crate::extensions::solver::{solve_optimum_from_position, OptimumMode};
use crate::skat::context::GameContext;
use crate::skat::defs::{Game, Player, ALLCARDS};
use crate::skat::engine::SkatEngine;
use crate::skat::position::Position;
use crate::traits::{BitConverter, Bitboard, Points, StringConverter};
use serde::{Deserialize, Serialize};
use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Clone)]
pub struct PlayInfo {
    pub card: String,
    pub player: String,
}

#[derive(Serialize)]
pub struct MoveLogEntry {
    pub card: String,
    pub player: String,
    pub value_before: Option<i32>,
    pub value_after: Option<i32>,
    pub delta: Option<i32>,
}

#[derive(Serialize)]
pub struct GameStateJson {
    pub my_cards: String,
    pub trick_cards: String,
    pub trick_plays: Vec<PlayInfo>,
    pub trick_suit: String,
    pub current_value: i32,
    pub last_loss: i32,
    pub game_over: bool,
    pub winner: Option<String>,
    pub declarer_points: u8,
    pub team_points: u8,
    pub max_possible_points: u8, // New Field
    pub current_player: String,
    pub last_trick_cards: Option<String>,
    pub last_trick_plays: Vec<PlayInfo>,
    pub last_trick_winner: Option<String>,
    pub last_trick_points: Option<u8>,
    pub left_cards: String,
    pub right_cards: String,
    pub skat_cards: String,
    pub move_history: Vec<MoveLogEntry>, // New Log
    pub legal_moves: Vec<String>,
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
    history: Vec<(Position, Vec<(u32, Player)>)>,
    last_loss: i32,
    last_trick_cards: Option<u32>,
    last_trick_winner: Option<Player>,
    last_trick_points: Option<u8>,
    last_trick_plays: Vec<(u32, Player)>,
    current_position: Position,
    current_trick_plays: Vec<(u32, Player)>,
    skat_cards: u32,
    max_possible_points: u8,
    current_value: i32, // Cached analysis

    // History Analysis
    move_sequence: Vec<(u32, Player)>,
    analysis_values: Vec<Option<i32>>, // values[i] is value BEFORE move i (or after move i-1).
                                       // values[0] = Start. values[k] = After k moves.
}

#[wasm_bindgen]
impl SkatGame {
    pub fn new_random() -> SkatGame {
        let (mut deck, _count) = ALLCARDS.__decompose();

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
            Game::Suit,
            Player::Declarer,
        );

        let skat_points = skat_cards.points();
        context.set_declarer_start_points(skat_points);
        context.set_threshold_upper(120);

        let engine = SkatEngine::new(context, None);
        let current_position = engine.create_initial_position();

        // Calculate max possible points (BestValue)
        // Initialize max possible points (Expensive!)
        // let mut temp_engine = SkatEngine::new(engine.context, None);
        // let res = solve_optimum_from_position(
        //     &mut temp_engine,
        //     &current_position,
        //     OptimumMode::BestValue,
        // );
        // let max_possible_points = if let Ok((_, _, val)) = res { val } else { 0 };
        let max_possible_points = 120; // Default placeholder to unblock UI

        SkatGame {
            engine,
            user_player: Player::Declarer,
            history: Vec::new(),
            last_loss: 0,
            last_trick_cards: None,
            last_trick_winner: None,
            last_trick_points: None,
            last_trick_plays: Vec::new(),
            current_position,
            current_trick_plays: Vec::new(),
            skat_cards,
            max_possible_points,
            current_value: 0,
            move_sequence: Vec::new(),
            analysis_values: vec![None], // Initial state (index 0) is unknown yet
        }
    }

    pub fn get_state_json(&self) -> JsValue {
        let pos = self.current_position;

        let display_cards = if self.user_player == Player::Declarer {
            pos.declarer_cards.__str()
        } else {
            "".to_string()
        };

        let left_str = pos.left_cards.__str();
        let right_str = pos.right_cards.__str();
        let skat_str = self.skat_cards.__str();

        let trick_str = if pos.trick_cards != 0 {
            pos.trick_cards.__str()
        } else {
            "".to_string()
        };

        let trick_plays_json: Vec<PlayInfo> = self
            .current_trick_plays
            .iter()
            .map(|(c, p)| PlayInfo {
                card: c.__str().trim().to_string(),
                player: p.str().to_string(),
            })
            .collect();

        let last_trick_plays_json: Vec<PlayInfo> = self
            .last_trick_plays
            .iter()
            .map(|(c, p)| PlayInfo {
                card: c.__str().trim().to_string(),
                player: p.str().to_string(),
            })
            .collect();

        let current_value = self.current_value;

        let game_over = self.is_game_over();

        let state = GameStateJson {
            my_cards: display_cards,
            trick_cards: trick_str,
            trick_plays: trick_plays_json,
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
            max_possible_points: self.max_possible_points, // Pass through
            current_player: pos.player.str().to_string(),
            last_trick_cards: self.last_trick_cards.map(|c| c.__str()),
            last_trick_plays: last_trick_plays_json,
            last_trick_winner: self.last_trick_winner.map(|p| p.str().to_string()),
            last_trick_points: self.last_trick_points,
            left_cards: left_str,
            right_cards: right_str,
            skat_cards: skat_str,
            move_history: self.get_move_history_json(),
            legal_moves: self.get_legal_moves_strings(),
        };

        serde_wasm_bindgen::to_value(&state).unwrap()
    }

    fn get_legal_moves_strings(&self) -> Vec<String> {
        let pos = self.current_position;
        // Only relevant if it's user's turn (Declarer)
        if pos.player != self.user_player {
            return Vec::new(); // Or all? Usually UI only cares for user.
        }

        let legal_mask = pos.get_legal_moves();
        let mut legal_strs = Vec::new();
        for i in 0..32 {
            let card_bit = 1 << i;
            if (legal_mask & card_bit) != 0 {
                legal_strs.push(card_bit.__str());
            }
        }
        legal_strs
    }

    fn get_move_history_json(&self) -> Vec<MoveLogEntry> {
        let mut log = Vec::new();
        // move_sequence has N items (moves 1..N).
        // analysis_values has N+1 items (indices 0..N).
        for (i, (card, player)) in self.move_sequence.iter().enumerate() {
            // Move i connects State i -> State i+1.
            let val_before = if i < self.analysis_values.len() {
                self.analysis_values[i]
            } else {
                None
            };
            let val_after = if i + 1 < self.analysis_values.len() {
                self.analysis_values[i + 1]
            } else {
                None
            };

            let delta = if let (Some(v1), Some(v2)) = (val_before, val_after) {
                Some(v2 - v1)
            } else {
                None
            };

            log.push(MoveLogEntry {
                card: card.__str(),
                player: player.str().to_string(),
                value_before: val_before,
                value_after: val_after,
                delta,
            });
        }
        log
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
            return false;
        }
        if self.is_game_over() {
            return false;
        }

        let (best_card, _) = self.solve_best_move();
        if best_card == 0 {
            return false;
        }

        self.perform_move(best_card, &pos)
    }

    pub fn calculate_max_points(&mut self) -> u8 {
        // Recalculate if 0 or dummy? Assuming 120 is dummy.
        // Actually, just run it.
        let mut temp_engine = SkatEngine::new(self.engine.context, None);
        // We need INITIAL position for max points of the DEAL.
        // current_position might be mid-game.
        // But Max Points usually means "Max points achieveable from START with open cards"?
        // Or "Max points achieveable NOW"?
        // The display says "(85 max)". Usually implies theoretical max for the hand.
        // So we should use the initial position?
        // But we don't store initial position explicitly except in engine?
        // `engine.create_initial_position()` creates it from context.
        // YES.
        let initial_pos = self.engine.create_initial_position();

        let res =
            solve_optimum_from_position(&mut temp_engine, &initial_pos, OptimumMode::BestValue);
        let val = if let Ok((_, _, v)) = res { v as u8 } else { 0 };
        self.max_possible_points = val;
        val
    }

    pub fn calculate_analysis(&mut self) -> JsValue {
        // 1. Ensure Analysis History is up to date
        // Iterate through all history states and calculate value if missing.
        // self.history contains previous states in order.
        // self.history[i] is state BEFORE move i+1?
        // Let's verify.
        // perform_move pushes `(*pos, plays)` into `self.history`.
        // Then it updates `self.current_position`.
        // So `self.history[k]` corresponds to the position at step k.

        // We have `analysis_values` which should map 1:1 to `history` + `current`.
        // We want `analysis_values[k]` for state k.

        // Loop through all available states (history + current)
        // Total states = history.len() + 1 (current).

        let total_states = self.history.len() + 1;

        // Extend analysis_values if needed
        while self.analysis_values.len() < total_states {
            self.analysis_values.push(None); // Should match history growth
        }
        if self.analysis_values.len() < total_states {
            self.analysis_values.resize(total_states, None);
        }

        // Now iterate and fill missing
        for i in 0..total_states {
            if self.analysis_values[i].is_none() {
                // Get position for index i
                // If i < history.len(), it's in history[i].0
                // If i == history.len(), it's current_position

                let pos_to_solve = if i < self.history.len() {
                    self.history[i].0
                } else {
                    self.current_position
                };

                // Solve
                let mut temp_engine = SkatEngine::new(self.engine.context, None);
                let res = solve_optimum_from_position(
                    &mut temp_engine,
                    &pos_to_solve,
                    OptimumMode::BestValue,
                );
                if let Ok((_, _, v)) = res {
                    self.analysis_values[i] = Some(v as i32);
                }
            }
        }

        // Update cached current value
        if let Some(Some(v)) = self.analysis_values.last() {
            self.current_value = *v;
        }

        // Return a simple ack, the UI will pull get_state_json
        JsValue::TRUE
    }

    fn calculate_theoretical_value(&self) -> i32 {
        if self.is_game_over() {
            return 0; // Value is actual points?
        }
        let (_, val) = self.solve_best_move();
        val
    }

    // Helper to perform move WITHOUT Analysis
    fn perform_move(&mut self, card: u32, pos: &Position) -> bool {
        let legal_moves = pos.get_legal_moves();
        if (card & legal_moves) == 0 {
            return false;
        }

        self.history.push((*pos, self.current_trick_plays.clone()));
        self.current_trick_plays.push((card, pos.player));

        // Track History
        self.move_sequence.push((card, pos.player));
        self.analysis_values.push(None); // Placeholder for next state

        // REMOVED: best_val_before

        let trick_will_complete = pos.trick_cards.count_ones() == 2;
        let mut completed_trick_info = None;
        if trick_will_complete {
            let full_trick = pos.trick_cards | card;
            completed_trick_info = Some(full_trick);
        }

        let next_pos = pos.make_move(card, &self.engine.context);

        if let Some(trick) = completed_trick_info {
            self.last_trick_cards = Some(trick);
            self.last_trick_winner = Some(next_pos.player);
            self.last_trick_plays = self.current_trick_plays.clone();
            self.current_trick_plays.clear();

            let points_gained = if next_pos.player == Player::Declarer {
                next_pos.declarer_points.saturating_sub(pos.declarer_points)
            } else {
                next_pos.team_points.saturating_sub(pos.team_points)
            };
            self.last_trick_points = Some(points_gained);
        }

        self.current_position = next_pos;

        // REMOVED: best_val_after & last_loss calculation
        self.last_loss = 0; // Reset or keep previous? Reset indicates "Recalculating needed"

        true
    }

    // ... existing ... but need to keep solve_best_move and is_game_over

    pub fn undo(&mut self) {
        if let Some((prev_pos, prev_trick_plays)) = self.history.pop() {
            self.current_position = prev_pos;
            self.current_trick_plays = prev_trick_plays;
            self.last_loss = 0;
            self.last_trick_cards = None;
            self.last_trick_winner = None;
            self.last_trick_points = None;

            // Pop history
            self.move_sequence.pop();
            self.analysis_values.pop();
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
        let mut temp_engine = SkatEngine::new(self.engine.context, None);
        let pos = self.current_position;
        let res = solve_optimum_from_position(&mut temp_engine, &pos, OptimumMode::BestValue);

        if let Ok((card, score, val)) = res {
            (card, val as i32)
        } else {
            (0, 0)
        }
    }

    pub fn get_move_analysis_json(&self) -> JsValue {
        let pos = self.current_position;
        if pos.player != self.user_player || self.is_game_over() {
            return JsValue::NULL;
        }

        // 1. Get Best Value (Global Optimum)
        let (best_card_mask, best_val) = self.solve_best_move();

        let mut results = Vec::new();
        let legal_mask = pos.get_legal_moves();

        let mut temp_engine = SkatEngine::new(self.engine.context, None);

        for i in 0..32 {
            let card_bit = 1 << i;
            if (legal_mask & card_bit) != 0 {
                let next_pos = pos.make_move(card_bit, &self.engine.context);

                let res = solve_optimum_from_position(
                    &mut temp_engine,
                    &next_pos,
                    OptimumMode::BestValue,
                );

                let val = if let Ok((_, _, v)) = res { v as i32 } else { 0 };

                // Delta: best_val (our perspective) vs val (result of move).
                // Usually matching.
                let delta = val - best_val;

                results.push(MoveAnalysis {
                    card: card_bit.__str(),
                    value: val,
                    delta,
                    is_best: delta == 0,
                });
            }
        }

        serde_wasm_bindgen::to_value(&results).unwrap()
    }
}

#[derive(Serialize)]
pub struct MoveAnalysis {
    pub card: String,
    pub value: i32,
    pub delta: i32,
    pub is_best: bool,
}
