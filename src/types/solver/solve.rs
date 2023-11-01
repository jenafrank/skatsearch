use super::Solver;
use super::retargs::SolveAllCardsRet;
use super::retargs::SolveRet;
use super::retargs::SolveWithSkatRet;
use super::retargs::SolveWithSkatRetLine;
use super::retargs::SolveWinRet;
use crate::traits::Augen;
use crate::traits::Bitboard;
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::state::State;
use crate::types::tt_table::TtTable;

impl Solver {
    
    /// Analyses a twelve-card hand during the task of putting away two cards (Skat) before
    /// game starts. It analyses all 66 cases and calculating the best play for each of them
    /// using the same transposition table for speed-up reasons.
    /// # Variants of arguments
    /// * _false_ _false_: Returns exact values for all 66 games.
    /// * _true_ _false_: Returns best skat by means of alpha-window narrowing. Thus, the
    /// 66-array does also contain wrong values.
    /// * _false_ _true_: Returns some skat for which the game will be won.
    /// The routine always takes into account the value of the skat which is neglected by default
    /// in the basic search routines.
    pub fn solve_with_skat(
        &mut self,
        is_alpha_beta_accelerating: bool,
        is_winning_only: bool,
    ) -> SolveWithSkatRet {
        
        let mut ret: SolveWithSkatRet = SolveWithSkatRet {
            best_skat: None,
            all_skats: Vec::new(),
            counters: Counters::new()
        };

        let state = State::create_initial_state_from_problem(&self.problem);
        let remaining_cards = !state.get_all_unplayed_cards();
        let twelve_bit = remaining_cards | state.declarer_cards;
        let twelve = twelve_bit.__decompose_twelve();

        let mut alpha_with_skat = 0;
        for i in 0..11 {
            for j in i + 1..12 {
                let skat = twelve[i] | twelve[j];
                let declarer_cards = twelve_bit ^ skat;

                self.problem.set_declarer_cards(declarer_cards);
            
                let mut current_initial_state =
                    State::create_initial_state_from_problem(&self.problem);

                if is_alpha_beta_accelerating {
                    if alpha_with_skat > skat.__get_value() {
                        current_initial_state.alpha = alpha_with_skat - skat.__get_value();
                    }
                } else if is_winning_only {
                    if alpha_with_skat >= 61 {
                        return ret;
                    }

                    current_initial_state.alpha = 60 - skat.__get_value();
                    current_initial_state.beta = current_initial_state.alpha + 1;
                }

                let result = self.problem.search(&current_initial_state);
                let value_with_skat = result.1 + skat.__get_value();

                ret.all_skats.push( SolveWithSkatRetLine {
                    skat_card_1: twelve[i],
                    skat_card_2: twelve[j],
                    value: value_with_skat
                });

                if value_with_skat > alpha_with_skat {
                    ret.best_skat = SolveWithSkatRetLine {
                        skat_card_1: twelve[i],
                        skat_card_2: twelve[j],
                        value: value_with_skat
                    }.into();                        

                    alpha_with_skat = value_with_skat;
                }
            }
        }

        ret.counters = Counters::get();

        ret
    }

    /// Investigates all legal moves for a given state and returns an option array
    /// with 0) card under investigation 1) follow-up card from tree search (tree root) and
    /// 2) value of search
    pub fn solve_all_cards(&self, alpha: u8, beta: u8) -> SolveAllCardsRet {
        let initial_state = State::create_initial_state_from_problem(&self.problem);        
        self.get_all_cards(initial_state, alpha, beta)
    }

    pub fn solve_win(&self) -> SolveWinRet {
        let mut alpha = self.problem.points_to_win() - 1;
        let mut beta = self.problem.points_to_win();

        if self.problem.game_type() == Game::Null {
            alpha = 0;
            beta = 1;
        } 

        let state = self.problem.new_state(alpha, beta);
        let (best_card, value) = self.problem.search(&state);
        
        let mut declarer_wins = value > alpha;
        
        if self.problem.game_type() == Game::Null {
            declarer_wins = !declarer_wins;
        }        

        SolveWinRet {
            best_card,
            declarer_wins,
            counters: Counters::get()
        }
    }

    // works currently only with 10 cards, since all cards not part of the full deck
    // are considered as skat and thus as points fot the declarer.
    pub fn solve_win_10tricks(&mut self) -> SolveWinRet {
        let mut state = State::create_initial_state_from_problem(&self.problem);

        let skat_value = self.problem.get_skat().__get_value();

        let threshold_farbe_and_grand = 60 - skat_value;

        match self.problem.game_type() {
            Game::Farbe => {
                state.alpha = threshold_farbe_and_grand;
                state.beta = threshold_farbe_and_grand + 1;
            }
            Game::Grand => {
                state.alpha = threshold_farbe_and_grand;
                state.beta = threshold_farbe_and_grand + 1;
            }
            Game::Null => {
                state.alpha = 0;
                state.beta = 1;
            }
        }

        let result = self.problem.search(&state);
        let val = result.1;

        let declarer_wins = if self.problem.game_type() == Game::Null {
            val == 0
        } else {
            val > threshold_farbe_and_grand
        };

        SolveWinRet {
            best_card: result.0,
            declarer_wins,
            counters: Counters::get()
        }
    }

    // unclear, if the right best card is determined. complicated. in search routine we should
    // identify, if any best card has been detected so far
    pub fn solve_double_dummy(&self) -> SolveRet {
        let mut result = (0u32, 0u8);
        let mdf = 5u8;

        for i in 0..119 {
            let mut state = State::create_initial_state_from_problem(&self.problem);
            state.alpha = mdf * i;
            state.beta = mdf * (i + 1);
            result = self.problem.search(&state);

            if result.1 < state.beta {
                break;
            }
        }

        SolveRet { best_card: result.0, best_value: result.1, counters: Counters::get() }
    }   

    pub fn solve(&mut self) -> SolveRet {
        let state = State::create_initial_state_from_problem(&self.problem);
        let result = self.get(state);

        println!(" Iters: {}, Slots: {}, Writes: {}, Reads: {}, ExactReads: {}, Collisions: {}, Breaks: {}",
        result.counters.iters,
        TtTable::get().get_occupied_slots(),
        result.counters.writes,
        result.counters.reads,
        result.counters.exactreads,
        result.counters.collisions,
        result.counters.breaks);

        result
    }

}

#[cfg(test)]
mod tests {
    use crate::{types::{solver::Solver, player::Player, problem_builder::ProblemBuilder}, consts::bitboard::SPADES};

    #[test]
    fn test_solve_win() {

        let problem = ProblemBuilder::new_farbspiel()
        .cards_all("SA SK", "ST SQ", "S9 S8")
        .turn(Player::Declarer)
        .threshold(14)
        .build();

        let solver = Solver::create(problem);
        let result = solver.solve_win();

        assert_eq!(result.declarer_wins, true);
    }

    #[test]
    fn test_solve_win_intertrick() {
        let problem = ProblemBuilder::new_farbspiel()
        .cards_all("SA SK", "ST SQ", "S9 S8")
        .turn(Player::Left)
        .threshold(3)
        .trick(SPADES, "SA")
        .build();

        let solver = Solver::create(problem);
        let result = solver.solve_win();

        assert_eq!(result.declarer_wins, true);
    }
}