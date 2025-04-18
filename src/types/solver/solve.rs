use super::Solver;
use super::retargs::SolveAllCardsRet;
use super::retargs::SolveRet;
use super::retargs::SolveWinRet;
use crate::traits::Augen;
use crate::types::counter::Counters;
use crate::types::game::Game;
use crate::types::state::State;

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

impl Solver {

    /// Investigates all legal moves for a given state and returns an option array
    /// with 0. card under investigation 1. follow-up card from tree search (tree root) and
    /// 2. value of search
    pub fn solve_all_cards(&mut self, alpha: u8, beta: u8) -> SolveAllCardsRet {
        let initial_state = State::create_initial_state_from_problem(&self.problem);        
        self.get_all_cards(initial_state, alpha, beta)
    }

    pub fn solve_win(&mut self) -> SolveWinRet {
        
        let mut cnt = Counters::new();
        
        let mut alpha = self.problem.points_to_win() - 1;
        let mut beta = self.problem.points_to_win();

        if self.problem.game_type() == Game::Null {
            alpha = 0;
            beta = 1;
        } 

        let state = self.problem.new_state(alpha, beta);
        let (best_card, value) = self.problem.search(&state, &mut self.tt, &mut cnt);
        
        let mut declarer_wins = value > alpha;
        
        if self.problem.game_type() == Game::Null {
            declarer_wins = !declarer_wins;
        }        

        SolveWinRet {
            best_card,
            declarer_wins,
            counters: cnt
        }
    }

    // works currently only with 10 cards, since all cards not part of the full deck
    // are considered as skat and thus as points fot the declarer.
    pub fn solve_win_10tricks(&mut self) -> SolveWinRet {

        let mut cnt = Counters::new();

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

        let result = self.problem.search(&state, &mut self.tt, &mut cnt);
        let val = result.1;

        let declarer_wins = if self.problem.game_type() == Game::Null {
            val == 0
        } else {
            val > threshold_farbe_and_grand
        };

        SolveWinRet {
            best_card: result.0,
            declarer_wins,
            counters: cnt
        }
    }

    // unclear, if the right best card is determined. complicated. in search routine we should
    // identify, if any best card has been detected so far
    pub fn solve_double_dummy(&mut self, alpha: u8, beta: u8, width: u8) -> SolveRet {
        
        let mut cnt = Counters::new();
        let mut result = (0u32, 0u8);        
        
        let mut current_alpha = alpha;
        while current_alpha < beta {
            let current_beta = std::cmp::min(current_alpha + width, beta);
            
            let mut state = State::create_initial_state_from_problem(&self.problem);
            state.alpha = current_alpha;
            state.beta = current_beta;
            result = self.problem.search(&state, &mut self.tt, &mut cnt);

            if result.1 < current_beta {
                break;
            }
            
            current_alpha = current_beta;
        }

        SolveRet { best_card: result.0, best_value: result.1, counters: cnt }
    }   

    pub fn solve(&mut self) -> SolveRet {
        self.solve_double_dummy(0, 120, 1)
    }
    
    /// Berechnet die Doppeldummy-Lösung und addiert den Skat-Wert,
    /// mit speziellem Mapping für Null-Spiele.
    ///
    /// Für Null-Spiele:
    /// - Mappt auf 1, wenn das Spiel verloren wird (solve_double_dummy Ergebnis != 0).
    /// - Mappt auf 0, wenn das Spiel gewonnen wird (solve_double_dummy Ergebnis == 0).
    /// Der Skat-Wert wird für dieses Mapping ignoriert.
    ///
    /// Für alle anderen Spiele:
    /// - Addiert den Skat-Wert zum Ergebnis von solve_double_dummy.
    pub fn solve_and_add_skat(&mut self) -> SolveRet {
        
        // Zuerst das Kernproblem lösen.        
        let mut ret = self.solve_double_dummy(0, 120, 1);

        // Ergebnis von solve_double_dummy speichern, bevor es potenziell verändert wird
        // oder der skat_value addiert wird. Dies macht die Null-Logik klarer.
        let double_dummy_result = ret.best_value;

        let skat_value = self.problem.get_skat().__get_value(); // Wert des Skats holen

        // Spieltyp-spezifische Logik für den finalen Wert anwenden
        match self.problem.game_type() {
            Game::Null => {
                // Für Null-Spiele wird das Ergebnis basierend auf dem Roh-Ergebnis gemappt.
                // 0 Punkte im Nullspiel = Gewinn -> Mapping 0
                // > 0 Punkte im Nullspiel = Verlust -> Mapping 1
                if double_dummy_result == 0 {
                    ret.best_value = 0; // Gewonnen -> 0
                } else {
                    ret.best_value = 1; // Verloren -> 1
                }
                // Der skat_value wird für die 0/1-Bewertung von Null-Spielen ignoriert.
            }
            // TODO: Fügen Sie hier andere Spieltypen hinzu, falls deren Logik vom Standard abweicht.
            // Z.B. Grand, Farbe, etc.
            _ => {
                // Für alle anderen Spieltypen wird der Skat-Wert zum Ergebnis addiert.
                ret.best_value = double_dummy_result + skat_value;
            }
        }

        ret
    }

    pub fn solve_classic(&mut self) -> SolveRet {
        let state = State::create_initial_state_from_problem(&self.problem);
        let result = self.get(state);

        println!(" Iters: {}, Slots: {}, Writes: {}, Reads: {}, ExactReads: {}, Collisions: {}, Breaks: {}",
        result.counters.iters,
        self.tt.get_occupied_slots(),
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
    use crate::{consts::bitboard::SPADES, types::{player::Player, problem_builder::ProblemBuilder, solver::Solver}};

    #[test]
    fn test_solve_win() {

        let problem = ProblemBuilder::new_farbspiel()
        .cards_all("SA SK", "ST SQ", "S9 S8")
        .turn(Player::Declarer)
        .threshold(14)
        .build();

        let mut solver = Solver::new(problem, None);
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

        let mut solver = Solver::new(problem, None);
        let result = solver.solve_win();

        assert_eq!(result.declarer_wins, true);
    }
}