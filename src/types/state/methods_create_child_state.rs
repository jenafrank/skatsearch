use crate::core_functions::get_suit_for_card::get_suit_for_card;

use crate::traits::Augen;

use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::types::state::State;

impl State {

    pub fn create_child_state(
        &self, 
        card: u32, 
        problem: &Problem,
        alpha_start: u8,
        beta_start: u8) -> State {

        // new_player, may be overwritten after trick calculation !!
        let mut new_player = self.player.inc();

        // new_trick_suit
        let mut new_trick_suit: u32 = if self.trick_suit == 0 {
            get_suit_for_card(card, problem.game_type)
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

        // evaluate upon trick completion. Overwrites four variables. Accumulates augen vars.
        if new_trick_cards_count == 3 {
            let augen = new_trick_cards.__get_value_of_three_cards();
            let winner = self.get_trick_winner(new_trick_cards, new_trick_suit, problem);

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

        // Updating current hand cards cache variable according to current player
        let new_player_cards = match new_player {
            Player::Declarer => new_declarer_cards,
            Player::Left => new_left_cards,
            Player::Right => new_right_cards,
        };

        State {
            played_cards: new_played_cards,
            declarer_cards: new_declarer_cards,
            left_cards: new_left_cards,
            right_cards: new_right_cards,
            trick_cards: new_trick_cards,
            trick_suit: new_trick_suit,
            trick_cards_count: new_trick_cards_count,
            augen_declarer: new_augen_declarer,
            augen_team: new_augen_team,
            augen_future: new_augen_future,
            player: new_player,
            player_cards: new_player_cards,
            alpha: alpha_start,
            beta: beta_start,
            mapped_hash: 0,
            is_root_state: false
        }.add_hash()

    }

}
