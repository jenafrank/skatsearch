use super::*;

use crate::types::player::Player;
use crate::types::problem::Problem;
use crate::core_functions::get_legal_moves::get_legal_moves;
use crate::core_functions::get_suit_for_card::get_suit_for_card;
use crate::traits::Bitboard;

impl State {
    pub fn get_forecasted_winner_of_current_trick(&self, card: u32, problem: &Problem) -> Option<Player> {
        return match self.trick_cards_count {
            2 => self.trick_will_be_won_with_two_cards_already_on_table(card, problem),
            1 => self.trick_will_be_won_with_one_card_already_on_table(card, problem),
            0 => self.trick_will_be_won_with_no_card_already_on_table(card, problem),
            _ => panic!("Trick card count. Not a legal value: {}.", self.trick_cards_count)
        }
    }

    fn trick_will_be_won_with_no_card_already_on_table(&self, card: u32, problem: &Problem) -> Option<Player> {
        let trick_suit = get_suit_for_card(card, problem.game_type);
        let mut winner_accumulated: Option<Player> = None;

        let next_player_moves = self.get_forecasted_moves(self.player.inc(), trick_suit);
        let next2_player_moves = self.get_forecasted_moves(self.player.inc().inc(), trick_suit);

        let (mov_arr_1, mov_arr_1_n) = next_player_moves.__decompose();
        let (mov_arr_2, mov_arr_2_n) = next2_player_moves.__decompose();

        for mov1 in &mov_arr_1[0..mov_arr_1_n] {
            for mov2 in &mov_arr_2[0..mov_arr_2_n] {
                let winner = self.get_trick_winner(card | mov1 | mov2, trick_suit, problem);

                if winner_accumulated == None {
                    winner_accumulated = Some(winner);
                } else if winner_accumulated != Some(winner) {
                    return None;
                }
            }
        }
        winner_accumulated
    }

    fn trick_will_be_won_with_one_card_already_on_table(&self, card: u32, problem: &Problem) -> Option<Player> {
        let trick_suit = self.trick_suit;
        let mut winner_accumulated: Option<Player> = None;

        let next_player_moves = self.get_forecasted_moves(self.player.inc(), trick_suit);

        let (mov_arr_1, mov_arr_1_n) = next_player_moves.__decompose();

        for mov1 in &mov_arr_1[0..mov_arr_1_n] {
            let winner = self.get_trick_winner(self.trick_cards | card | mov1, trick_suit, problem);

            if winner_accumulated == None {
                winner_accumulated = Some(winner);
            } else if winner_accumulated != Some(winner) {
                return None;
            }
        }

        winner_accumulated
    }

    fn trick_will_be_won_with_two_cards_already_on_table(&self, card: u32, problem: &Problem) -> Option<Player> {
        let winner = self.get_trick_winner(self.trick_cards | card, self.trick_suit, problem);
        Some(winner)
    }

    fn get_forecasted_moves(&self, player: Player, trick_suit: u32) -> u32 {
        let next_player_cards = self.get_cards_for_player(player);
        get_legal_moves(trick_suit, next_player_cards)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::types::problem::Problem;
    use crate::types::player::Player;
    use crate::traits::BitConverter;

    #[test]
    fn test_forecast_trivial() {
        let p = Problem::create_farbe_declarer_problem("DJ CA", "CT SA", "HA HT");
        let s = State::create_initial_state_from_problem(&p);
        let w1 = s.get_forecasted_winner_of_current_trick("DJ".__bit(), &p);
        let w2 = s.get_forecasted_winner_of_current_trick("CA".__bit(), &p);
        assert_eq!(w1.unwrap(), Player::Declarer);
        assert_eq!(w2.unwrap(), Player::Declarer);
    }

}