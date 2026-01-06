//! # Position
//!
//! Represents the state of the game at a specific point in time.
//! Formerly known as `State`.

use crate::skat::context::GameContext;
use crate::skat::defs::{Game, Player};
use crate::skat::rules::*;
use crate::traits::{Augen, Bitboard}; // Check if these traits need moving // renaming Problem->GameContext later

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Position {
    // Primary values
    pub player: Player,
    pub played_cards: u32,
    pub trick_cards: u32,
    pub trick_suit: u32,
    pub augen_declarer: u8,

    // Derived (cached) values
    pub declarer_cards: u32,
    pub left_cards: u32,
    pub right_cards: u32,
    pub player_cards: u32,
    pub trick_cards_count: u8,
    pub augen_future: u8,
    pub augen_team: u8,

    // Additional values
    pub is_root_position: bool,

    hash: usize,
}

impl Position {
    pub fn new(
        player: Player,
        played_cards: u32,
        trick_cards: u32,
        trick_suit: u32,
        augen_declarer: u8,
        declarer_cards: u32,
        left_cards: u32,
        right_cards: u32,
        player_cards: u32,
        trick_cards_count: u8,
        augen_future: u8,
        augen_team: u8,
        is_root_position: bool,
    ) -> Self {
        let pos = Self {
            player,
            played_cards,
            trick_cards,
            trick_suit,
            augen_declarer,
            declarer_cards,
            left_cards,
            right_cards,
            player_cards,
            trick_cards_count,
            augen_future,
            augen_team,
            is_root_position,
            hash: 0,
        };
        pos.add_hash()
    }

    pub fn create(
        played_cards: u32,
        trick_cards: u32,
        trick_suit: u32,
        augen_declarer: u8,
        player: Player,
        game_context: &GameContext,
        is_root_position: bool,
    ) -> Position {
        let declarer_cards: u32 = game_context.declarer_cards() & !played_cards;
        let left_cards: u32 = game_context.left_cards() & !played_cards;
        let right_cards: u32 = game_context.right_cards() & !played_cards;
        let augen_future = game_context.augen_total() - (played_cards & !trick_cards).__get_value();
        let player_cards = match player {
            Player::Declarer => declarer_cards,
            Player::Left => left_cards,
            Player::Right => right_cards,
        };
        let trick_cards_count = trick_cards.count_ones() as u8;
        let augen_team = game_context.augen_total() - augen_future - augen_declarer;

        Position::new(
            player,
            played_cards,
            trick_cards,
            trick_suit,
            augen_declarer,
            declarer_cards,
            left_cards,
            right_cards,
            player_cards,
            trick_cards_count,
            augen_future,
            augen_team,
            is_root_position,
        )
    }

    pub fn create_initial_position(game_context: &GameContext) -> Position {
        Position::create(
            0u32,
            0u32,
            0u32,
            0u8,
            game_context.start_player(),
            game_context,
            true,
        )
    }

    pub fn make_move(&self, card: u32, game_context: &GameContext) -> Position {
        // new_player
        let mut new_player = self.player.inc(); // CHECK inc method location

        // new_trick_suit
        let mut new_trick_suit: u32 = if self.trick_suit == 0 {
            get_suit_for_card(card, game_context.game_type())
        } else {
            self.trick_suit
        };

        // new played_cards
        let new_played_cards = self.played_cards ^ card;

        // new trick_cards
        let mut new_trick_cards = self.trick_cards ^ card;
        let mut new_trick_cards_count = self.trick_cards_count + 1;

        // new augen
        let mut new_augen_declarer = self.augen_declarer;
        let mut new_augen_team = self.augen_team;
        let mut new_augen_future = self.augen_future;

        // new cards on hand
        let mut new_declarer_cards = self.declarer_cards;
        let mut new_left_cards = self.left_cards;
        let mut new_right_cards = self.right_cards;

        // Update hand cards according to last player
        match self.player {
            Player::Declarer => new_declarer_cards ^= card,
            Player::Left => new_left_cards ^= card,
            Player::Right => new_right_cards ^= card,
        };

        // evaluate upon trick completion
        if new_trick_cards_count == 3 {
            let augen = if game_context.game_type() == Game::Null {
                1
            } else {
                new_trick_cards.__get_value_of_three_cards()
            };

            let winner = self.calculate_trick_winner(new_trick_cards, new_trick_suit, game_context);

            new_trick_cards = 0;
            new_trick_cards_count = 0;
            new_trick_suit = 0;
            new_player = winner;

            match winner {
                Player::Declarer => new_augen_declarer += augen,
                _ => new_augen_team += augen,
            }

            new_augen_future -= augen;
        }

        // Updating current hand cards cache variable
        let new_player_cards = match new_player {
            Player::Declarer => new_declarer_cards,
            Player::Left => new_left_cards,
            Player::Right => new_right_cards,
        };

        Position::new(
            new_player,
            new_played_cards,
            new_trick_cards,
            new_trick_suit,
            new_augen_declarer,
            new_declarer_cards,
            new_left_cards,
            new_right_cards,
            new_player_cards,
            new_trick_cards_count,
            new_augen_future,
            new_augen_team,
            false,
        )
    }

    pub fn get_legal_moves(&self) -> u32 {
        get_legal_moves(self.trick_suit, self.player_cards)
    }

    pub fn get_all_unplayed_cards(&self) -> u32 {
        get_all_unplayed_cards(self.declarer_cards, self.left_cards, self.right_cards)
    }

    pub fn calculate_trick_winner(
        &self,
        trick_cards: u32,
        trick_suit: u32,
        game_context: &GameContext,
    ) -> Player {
        get_trick_winner(
            trick_cards,
            trick_suit,
            game_context.game_type(),
            game_context.declarer_cards(),
            game_context.left_cards(),
            game_context.right_cards(),
        )
    }

    pub fn get_hash(&self) -> usize {
        self.hash
    }

    fn add_hash(mut self) -> Position {
        self.hash = get_mapped_hash(get_hash(
            self.player,
            self.declarer_cards, // Optimization: Use pointers or references? No, u32 is cheap.
            self.left_cards,
            self.right_cards,
            self.trick_cards,
        ));
        self
    }

    // Move reduction methods
    pub fn get_reduced_moves(&self, game_context: &GameContext) -> u32 {
        let mut moves = self.get_legal_moves();
        match game_context.game_type() {
            Game::Farbe | Game::Grand => {
                moves = self.reduce_unequal(moves, game_context);
                moves = self.reduce_equal(moves, game_context.game_type());
            }
            Game::Null => {
                moves = self.reduce_equal(moves, game_context.game_type());
            }
        }
        moves
    }

    fn reduce_unequal(&self, moves: u32, game_context: &GameContext) -> u32 {
        let mut ret = 0u32;
        let player = self.player;
        let all_moves = self.left_cards | self.right_cards | self.declarer_cards;

        let connections = get_connections(
            moves,
            all_moves,
            game_context.game_type().get_unequal_sequence(),
        );

        let mut i = 1;
        while connections[i].0 > 0 {
            let repr = connections[i].1;
            let winner = self.get_forecasted_winner_of_current_trick(repr, game_context);

            if let Some(x) = winner {
                if x.is_same_team_as(player) {
                    ret |= connections[i].1;
                } else {
                    ret |= connections[i].2;
                }
            } else {
                ret |= connections[i].0;
            }

            i += 1;
        }

        ret | connections[0].1
    }

    fn reduce_equal(&self, moves: u32, game: Game) -> u32 {
        let all_moves = self.declarer_cards | self.left_cards | self.right_cards | self.trick_cards;
        let connection_list = game.get_equal_sequence();
        get_reduced_equal_core(moves, all_moves, connection_list)
    }

    pub fn get_forecasted_winner_of_current_trick(
        &self,
        card: u32,
        game_context: &GameContext,
    ) -> Option<Player> {
        match self.trick_cards_count {
            2 => Some(self.calculate_trick_winner(
                self.trick_cards | card,
                self.trick_suit,
                game_context,
            )),
            1 => self.trick_will_be_won_with_one_card_already_on_table(card, game_context),
            0 => self.trick_will_be_won_with_no_card_already_on_table(card, game_context),
            _ => panic!("Trick card count invalid: {}.", self.trick_cards_count),
        }
    }

    fn trick_will_be_won_with_no_card_already_on_table(
        &self,
        card: u32,
        game_context: &GameContext,
    ) -> Option<Player> {
        let trick_suit = get_suit_for_card(card, game_context.game_type());
        let mut winner_accumulated: Option<Player> = None;

        let next_player_moves = self.get_forecasted_moves(self.player.inc(), trick_suit);
        let next2_player_moves = self.get_forecasted_moves(self.player.inc().inc(), trick_suit);

        let (mov_arr_1, mov_arr_1_n) = next_player_moves.__decompose();
        let (mov_arr_2, mov_arr_2_n) = next2_player_moves.__decompose();

        for mov1 in &mov_arr_1[0..mov_arr_1_n] {
            for mov2 in &mov_arr_2[0..mov_arr_2_n] {
                let winner =
                    self.calculate_trick_winner(card | mov1 | mov2, trick_suit, game_context);

                if winner_accumulated == None {
                    winner_accumulated = Some(winner);
                } else if winner_accumulated != Some(winner) {
                    return None;
                }
            }
        }
        winner_accumulated
    }

    fn trick_will_be_won_with_one_card_already_on_table(
        &self,
        card: u32,
        game_context: &GameContext,
    ) -> Option<Player> {
        let trick_suit = self.trick_suit;
        let mut winner_accumulated: Option<Player> = None;

        let next_player_moves = self.get_forecasted_moves(self.player.inc(), trick_suit);
        let (mov_arr_1, mov_arr_1_n) = next_player_moves.__decompose();

        for mov1 in &mov_arr_1[0..mov_arr_1_n] {
            let winner = self.calculate_trick_winner(
                self.trick_cards | card | mov1,
                trick_suit,
                game_context,
            );

            if winner_accumulated == None {
                winner_accumulated = Some(winner);
            } else if winner_accumulated != Some(winner) {
                return None;
            }
        }

        winner_accumulated
    }

    fn get_forecasted_moves(&self, player: Player, trick_suit: u32) -> u32 {
        // Need to replicate get_cards_for_player logic here or make it a method
        let next_player_cards = match player {
            Player::Declarer => self.declarer_cards,
            Player::Left => self.left_cards,
            Player::Right => self.right_cards,
        };
        get_legal_moves(trick_suit, next_player_cards)
    }
}

// Helper definitions for methods used about the Player enum (inc, is_same_team_as) need to be ensured.
// Assuming they are on the Player impl which we need to make sure is available or replicated.
// In defs.rs we just defined the Enum, but not the methods.
// We must add those methods to defs.rs or a separate impl file.
